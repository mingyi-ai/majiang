use super::*;

/// A specialized Bit Array for Mahjong.
///
/// Layout:
/// - Row 0: Characters (1-9). 9 tiles * 4 bits = 36 bits.
/// - Row 1: Dots (1-9). 9 tiles * 4 bits = 36 bits.
/// - Row 2: Bamboos (1-9). 9 tiles * 4 bits = 36 bits.
/// - Row 3: Honors (7 types).
///   - Honors: 7 * 4 bits = 28 bits.
///   - Total: 36 bits.
///
/// Each tile (including flowers) uses 4 bits
/// (LSB fill) to represent count.
/// Count representation (LSB fill):
/// 0: 0000
/// 1: 0001
/// 2: 0011
/// 3: 0111
/// 4: 1111
///
#[derive(Clone, Copy, Default)]
pub(crate) struct BitTileCounts {
    pub(crate) rows: [u64; 4],
}

const TILE_COUNT_MASK: u64 = 0xF; // Mask for 4 bits

// Basic operations for BitTileCounts that are internal knowledge
impl BitTileCounts {
    pub(crate) fn tile_to_position(tile: Tile) -> (usize, usize) {
        debug_assert!(
            !matches!(tile.get_type(), TileType::Flower),
            "Flower tiles should not be represented in BitTileCounts"
        );
        let id = tile as usize;
        let row = id >> 6; // id / 64
        let shift = id & 0x3F; // id % 64
        (row, shift)
    }

    pub(crate) fn insert(&mut self, tile: Tile) {
        let (row, shift) = Self::tile_to_position(tile);

        let slot_val = (self.rows[row] >> shift) & TILE_COUNT_MASK;
        debug_assert_ne!(
            slot_val, TILE_COUNT_MASK,
            "Tile count cannot exceed 4"
        );

        let bit_to_add = slot_val + 1;
        self.rows[row] |= bit_to_add << shift;
    }

    pub(crate) fn remove_from_row(
        &mut self,
        row: usize,
        shift: usize,
        count: u8,
    ) {
        Self::remove_nibble(&mut self.rows[row], shift, count);
    }
    pub(crate) fn remove(&mut self, tile: Tile, count: u8) {
        let (row, shift) = Self::tile_to_position(tile);
        self.remove_from_row(row, shift, count);
    }

    pub(crate) fn count(&self, tile: Tile) -> u8 {
        let (row, shift) = Self::tile_to_position(tile);
        let slot_val = (self.rows[row] >> shift) & TILE_COUNT_MASK;
        slot_val.count_ones() as u8
    }

    pub(crate) fn position_to_tile(row: usize, shift: usize) -> Tile {
        debug_assert!(row < 4, "Row index out of bounds");
        Tile::from_repr(((row as u8) << 6) | (shift as u8))
    }

    /// Finds sequences (Chows) in a row.
    /// Returns a bitmask where bit `p` is set iff tiles at
    /// nibbles p, p+4, p+8 all have count >= 1.
    pub(crate) fn find_sequences_for_row(row: u64) -> u64 {
        row & (row >> 4) & (row >> 8)
    }

    // ── Static helpers operating on raw u64 rows ──
    // (Used by the hu solver which works on row copies.)

    pub(crate) fn add_to_row(row: &mut u64, shift: usize) {
        let slot_val = (*row >> shift) & TILE_COUNT_MASK;
        let bit_to_add = slot_val + 1;
        *row |= bit_to_add << shift;
    }

    /// Remove `count` tiles from a single nibble in a raw u64 row.
    pub(crate) fn remove_nibble(row: &mut u64, shift: usize, count: u8) {
        debug_assert!(
            ((*row >> shift) & TILE_COUNT_MASK).count_ones() >= count as u32,
            "Cannot remove more tiles than currently present"
        );
        let slot_val = (*row >> shift) & TILE_COUNT_MASK;
        let mask = slot_val ^ (slot_val >> count);
        *row &= !(mask << shift);
    }

    pub(crate) fn remove_sequence_from_row(row: &mut u64, shift: usize) {
        Self::remove_nibble(row, shift, 1);
        Self::remove_nibble(row, shift + 4, 1);
        Self::remove_nibble(row, shift + 8, 1);
    }
}

#[derive(Default, Clone, Copy)]
pub struct Hand {
    pub(crate) concealed: BitTileCounts,
    pub(crate) melds: [Option<Meld>; 4],
    pub(crate) flower_count: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reuse tile groupings from the domain — source of truth is the variant name.
    const SUIT_TILES: [Tile; 27] = [
        Tile::Character1, Tile::Character2, Tile::Character3,
        Tile::Character4, Tile::Character5, Tile::Character6,
        Tile::Character7, Tile::Character8, Tile::Character9,
        Tile::Dot1, Tile::Dot2, Tile::Dot3,
        Tile::Dot4, Tile::Dot5, Tile::Dot6,
        Tile::Dot7, Tile::Dot8, Tile::Dot9,
        Tile::Bamboo1, Tile::Bamboo2, Tile::Bamboo3,
        Tile::Bamboo4, Tile::Bamboo5, Tile::Bamboo6,
        Tile::Bamboo7, Tile::Bamboo8, Tile::Bamboo9,
    ];
    const HONOR_TILES: [Tile; 7] = [
        Tile::East, Tile::South, Tile::West, Tile::North,
        Tile::Red, Tile::Green, Tile::White,
    ];

    // ── tile_to_position / position_to_tile ──

    #[test]
    fn tile_to_position_characters_row_0() {
        // Character1 = 0  → (0, 0)
        // Character2 = 4  → (0, 4)
        // Character9 = 32 → (0, 32)
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character1), (0, 0));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character2), (0, 4));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character3), (0, 8));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character4), (0, 12));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character5), (0, 16));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character6), (0, 20));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character7), (0, 24));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character8), (0, 28));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Character9), (0, 32));
    }

    #[test]
    fn tile_to_position_dots_row_1() {
        assert_eq!(BitTileCounts::tile_to_position(Tile::Dot1), (1, 0));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Dot5), (1, 16));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Dot9), (1, 32));
    }

    #[test]
    fn tile_to_position_bamboos_row_2() {
        assert_eq!(BitTileCounts::tile_to_position(Tile::Bamboo1), (2, 0));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Bamboo5), (2, 16));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Bamboo9), (2, 32));
    }

    #[test]
    fn tile_to_position_honors_row_3() {
        assert_eq!(BitTileCounts::tile_to_position(Tile::East), (3, 0));
        assert_eq!(BitTileCounts::tile_to_position(Tile::South), (3, 4));
        assert_eq!(BitTileCounts::tile_to_position(Tile::West), (3, 8));
        assert_eq!(BitTileCounts::tile_to_position(Tile::North), (3, 12));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Red), (3, 16));
        assert_eq!(BitTileCounts::tile_to_position(Tile::Green), (3, 20));
        assert_eq!(BitTileCounts::tile_to_position(Tile::White), (3, 24));
    }

    #[test]
    fn position_to_tile_round_trips() {
        for &tile in &SUIT_TILES {
            let (row, shift) = BitTileCounts::tile_to_position(tile);
            let recovered = BitTileCounts::position_to_tile(row, shift);
            assert_eq!(tile, recovered, "round-trip failed for {:?}", tile);
        }
        for &tile in &HONOR_TILES {
            let (row, shift) = BitTileCounts::tile_to_position(tile);
            let recovered = BitTileCounts::position_to_tile(row, shift);
            assert_eq!(tile, recovered, "round-trip failed for {:?}", tile);
        }
    }

    // ── LSB-fill insert / count ──

    #[test]
    fn insert_increments_count_to_four() {
        let mut b = BitTileCounts::default();
        assert_eq!(b.count(Tile::Character1), 0);

        b.insert(Tile::Character1);
        assert_eq!(b.count(Tile::Character1), 1);

        b.insert(Tile::Character1);
        assert_eq!(b.count(Tile::Character1), 2);

        b.insert(Tile::Character1);
        assert_eq!(b.count(Tile::Character1), 3);

        b.insert(Tile::Character1);
        assert_eq!(b.count(Tile::Character1), 4);
    }

    #[test]
    fn insert_works_for_all_suit_tiles() {
        let mut b = BitTileCounts::default();
        for &tile in &SUIT_TILES {
            b.insert(tile);
            assert_eq!(b.count(tile), 1, "{:?}", tile);
        }
    }

    #[test]
    fn insert_works_for_all_honor_tiles() {
        let mut b = BitTileCounts::default();
        for &tile in &HONOR_TILES {
            b.insert(tile);
            assert_eq!(b.count(tile), 1, "{:?}", tile);
        }
    }

    #[test]
    #[should_panic(expected = "Tile count cannot exceed 4")]
    fn insert_panics_on_fifth_tile() {
        let mut b = BitTileCounts::default();
        for _ in 0..4 {
            b.insert(Tile::Dot5);
        }
        // Fifth insert should trigger debug_assert
        b.insert(Tile::Dot5);
    }

    #[test]
    fn insert_multiple_tiles_are_independent() {
        let mut b = BitTileCounts::default();
        b.insert(Tile::Character1);
        b.insert(Tile::Character1);
        b.insert(Tile::Character2);
        b.insert(Tile::Dot1);
        b.insert(Tile::East);

        assert_eq!(b.count(Tile::Character1), 2);
        assert_eq!(b.count(Tile::Character2), 1);
        assert_eq!(b.count(Tile::Character3), 0);
        assert_eq!(b.count(Tile::Dot1), 1);
        assert_eq!(b.count(Tile::East), 1);
    }

    #[test]
    fn insert_different_rows_dont_interfere() {
        let mut b = BitTileCounts::default();
        // Fill row 0, leave others empty
        for _ in 0..4 {
            b.insert(Tile::Character9);
        }
        // Row 1 and 2 should be untouched
        assert_eq!(b.count(Tile::Dot1), 0);
        assert_eq!(b.count(Tile::Bamboo1), 0);
        // Row 3 should be untouched
        assert_eq!(b.count(Tile::East), 0);
    }

    // ── Remove (and remove_from_row) ──

    #[test]
    fn remove_decrements_count() {
        let mut b = BitTileCounts::default();
        for _ in 0..4 {
            b.insert(Tile::Bamboo3);
        }
        assert_eq!(b.count(Tile::Bamboo3), 4);

        b.remove(Tile::Bamboo3, 1);
        assert_eq!(b.count(Tile::Bamboo3), 3);

        b.remove(Tile::Bamboo3, 1);
        assert_eq!(b.count(Tile::Bamboo3), 2);

        b.remove(Tile::Bamboo3, 2);
        assert_eq!(b.count(Tile::Bamboo3), 0);
    }

    #[test]
    fn remove_partial_count() {
        let mut b = BitTileCounts::default();
        for _ in 0..4 {
            b.insert(Tile::White);
        }
        b.remove(Tile::White, 3);
        assert_eq!(b.count(Tile::White), 1);
    }

    #[test]
    fn remove_from_row_equivalent_to_remove() {
        let mut b = BitTileCounts::default();
        for _ in 0..3 {
            b.insert(Tile::Character5);
        }

        let mut b2 = b;
        let (row, shift) = BitTileCounts::tile_to_position(Tile::Character5);
        b2.remove_from_row(row, shift, 2);
        b.remove(Tile::Character5, 2);
        assert_eq!(b.count(Tile::Character5), b2.count(Tile::Character5));
    }

    #[test]
    #[should_panic(expected = "Cannot remove more tiles than currently present")]
    fn remove_panics_on_underflow() {
        let mut b = BitTileCounts::default();
        b.insert(Tile::Red);
        b.remove(Tile::Red, 2); // only 1 present
    }

    #[test]
    #[should_panic(expected = "Cannot remove more tiles than currently present")]
    fn remove_panics_from_empty() {
        let mut b = BitTileCounts::default();
        b.remove(Tile::Green, 1); // 0 present
    }

    // ── add_to_row (static helper) ──

    #[test]
    fn add_to_row_matches_insert() {
        let mut row = 0u64;
        BitTileCounts::add_to_row(&mut row, 0);
        assert_eq!((row >> 0) & TILE_COUNT_MASK, 0b0001);

        BitTileCounts::add_to_row(&mut row, 0);
        assert_eq!((row >> 0) & TILE_COUNT_MASK, 0b0011);

        BitTileCounts::add_to_row(&mut row, 0);
        assert_eq!((row >> 0) & TILE_COUNT_MASK, 0b0111);

        BitTileCounts::add_to_row(&mut row, 0);
        assert_eq!((row >> 0) & TILE_COUNT_MASK, 0b1111);
    }

    #[test]
    fn add_to_row_at_different_shifts() {
        let mut row = 0u64;
        BitTileCounts::add_to_row(&mut row, 4);
        BitTileCounts::add_to_row(&mut row, 4);
        assert_eq!((row >> 4) & TILE_COUNT_MASK, 0b0011);
        // shift 0 should still be empty
        assert_eq!((row >> 0) & TILE_COUNT_MASK, 0b0000);
    }

    // ── remove_nibble (static helper) ──

    #[test]
    fn remove_nibble_partial() {
        let mut row = 0u64;
        // Set nibble at shift 8 to count 4 (0b1111)
        row |= 0b1111u64 << 8;
        BitTileCounts::remove_nibble(&mut row, 8, 2);
        assert_eq!((row >> 8) & TILE_COUNT_MASK, 0b0011); // count 2

        BitTileCounts::remove_nibble(&mut row, 8, 1);
        assert_eq!((row >> 8) & TILE_COUNT_MASK, 0b0001); // count 1

        BitTileCounts::remove_nibble(&mut row, 8, 1);
        assert_eq!((row >> 8) & TILE_COUNT_MASK, 0b0000); // count 0
    }

    // ── find_sequences_for_row ──

    #[test]
    fn find_sequences_adjacent_triple() {
        // Three consecutive tiles each count >= 1
        let mut row: u64 = 0;
        BitTileCounts::add_to_row(&mut row, 0);  // Character1
        BitTileCounts::add_to_row(&mut row, 4);  // Character2
        BitTileCounts::add_to_row(&mut row, 8);  // Character3

        let seqs = BitTileCounts::find_sequences_for_row(row);
        // Bit 0 should be set (sequence starting at Character1)
        assert_ne!(seqs & (1 << 0), 0, "expected sequence at shift 0");
        // No other sequence bits should be set
        assert_eq!(seqs & !(1 << 0), 0, "unexpected sequence bits: {:#b}", seqs);
    }

    #[test]
    fn find_sequences_no_full_triple() {
        // Only two consecutive tiles — no sequence
        let mut row: u64 = 0;
        BitTileCounts::add_to_row(&mut row, 0);  // Character1
        BitTileCounts::add_to_row(&mut row, 4);  // Character2
        // Missing Character3

        let seqs = BitTileCounts::find_sequences_for_row(row);
        assert_eq!(seqs, 0, "expected no sequences");
    }

    #[test]
    fn find_sequences_multiple_overlapping() {
        // Tiles at positions 0,4,8 and 4,8,12 and 8,12,16
        let mut row: u64 = 0;
        BitTileCounts::add_to_row(&mut row, 0);   // 1
        BitTileCounts::add_to_row(&mut row, 4);   // 2
        BitTileCounts::add_to_row(&mut row, 8);   // 3
        BitTileCounts::add_to_row(&mut row, 12);  // 4
        BitTileCounts::add_to_row(&mut row, 16);  // 5

        let seqs = BitTileCounts::find_sequences_for_row(row);
        // Bit 0: 1-2-3 ✓
        assert_ne!(seqs & (1 << 0), 0, "expected sequence at shift 0");
        // Bit 4: 2-3-4 ✓
        assert_ne!(seqs & (1 << 4), 0, "expected sequence at shift 4");
        // Bit 8: 3-4-5 ✓
        assert_ne!(seqs & (1 << 8), 0, "expected sequence at shift 8");
        // No other bits
        assert_eq!(seqs & !((1 << 0) | (1 << 4) | (1 << 8)), 0);
    }

    #[test]
    fn find_sequences_count_two_qualifies() {
        // Having count 2 at a position should still register as >= 1
        let mut row: u64 = 0;
        BitTileCounts::add_to_row(&mut row, 0);  // Character1
        BitTileCounts::add_to_row(&mut row, 0);  // Character1 again
        BitTileCounts::add_to_row(&mut row, 4);  // Character2
        BitTileCounts::add_to_row(&mut row, 8);  // Character3

        let seqs = BitTileCounts::find_sequences_for_row(row);
        assert_ne!(seqs & (1 << 0), 0, "count 2 should still register as present");
    }

    // ── remove_sequence_from_row ──

    #[test]
    fn remove_sequence_removes_one_from_each() {
        let mut row: u64 = 0;
        BitTileCounts::add_to_row(&mut row, 0);  // count 1
        BitTileCounts::add_to_row(&mut row, 4);  // count 1
        BitTileCounts::add_to_row(&mut row, 8);  // count 1

        BitTileCounts::remove_sequence_from_row(&mut row, 0);

        assert_eq!((row >> 0) & TILE_COUNT_MASK, 0b0000);
        assert_eq!((row >> 4) & TILE_COUNT_MASK, 0b0000);
        assert_eq!((row >> 8) & TILE_COUNT_MASK, 0b0000);
    }

    #[test]
    fn remove_sequence_handles_multiple_counts() {
        let mut row: u64 = 0;
        // Character1 x2, Character2 x1, Character3 x3
        BitTileCounts::add_to_row(&mut row, 0);
        BitTileCounts::add_to_row(&mut row, 0);
        BitTileCounts::add_to_row(&mut row, 4);
        BitTileCounts::add_to_row(&mut row, 8);
        BitTileCounts::add_to_row(&mut row, 8);
        BitTileCounts::add_to_row(&mut row, 8);

        BitTileCounts::remove_sequence_from_row(&mut row, 0);

        assert_eq!(((row >> 0) & TILE_COUNT_MASK).count_ones(), 1, "Ch1 should have 1 remaining");
        assert_eq!(((row >> 4) & TILE_COUNT_MASK).count_ones(), 0, "Ch2 should have 0 remaining");
        assert_eq!(((row >> 8) & TILE_COUNT_MASK).count_ones(), 2, "Ch3 should have 2 remaining");
    }

    // ── LSB-fill encoding invariants ──

    #[test]
    fn lsb_fill_pattern_is_monotonic() {
        // The four LSB-fill patterns for counts 0..4
        let expected = [0b0000u64, 0b0001, 0b0011, 0b0111, 0b1111];
        for count in 0..=4usize {
            // Build it by inserting `count` times
            let mut b = BitTileCounts::default();
            for _ in 0..count {
                b.insert(Tile::Character3);
            }
            let nibble = (b.rows[0] >> 8) & TILE_COUNT_MASK;
            assert_eq!(
                nibble, expected[count],
                "LSB-fill for count {} should be {:#06b}, got {:#06b}",
                count, expected[count], nibble
            );
            assert_eq!(nibble.count_ones() as usize, count);
        }
    }

    #[test]
    fn lsb_fill_remove_returns_to_previous_pattern() {
        let mut b = BitTileCounts::default();
        for _ in 0..4 {
            b.insert(Tile::Dot5);
        }
        assert_eq!(b.count(Tile::Dot5), 4);

        b.remove(Tile::Dot5, 1);
        assert_eq!(b.count(Tile::Dot5), 3);

        b.remove(Tile::Dot5, 1);
        assert_eq!(b.count(Tile::Dot5), 2);

        b.remove(Tile::Dot5, 1);
        assert_eq!(b.count(Tile::Dot5), 1);

        b.remove(Tile::Dot5, 1);
        assert_eq!(b.count(Tile::Dot5), 0);
    }

    // ── Default ──

    #[test]
    fn default_bit_tile_counts_is_all_zeros() {
        let b = BitTileCounts::default();
        for &tile in &SUIT_TILES {
            assert_eq!(b.count(tile), 0, "{:?}", tile);
        }
        for &tile in &HONOR_TILES {
            assert_eq!(b.count(tile), 0, "{:?}", tile);
        }
    }

    #[test]
    fn default_hand_is_empty() {
        let h = Hand::default();
        assert_eq!(h.flower_count, 0);
        assert!(h.melds.iter().all(|m| m.is_none()));
    }
}

