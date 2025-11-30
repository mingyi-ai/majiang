use crate::tile::Tile;

/// A specialized Bit Array for Mahjong.
///
/// Layout:
/// - Row 0: Characters (1-9). 9 tiles * 4 bits = 36 bits.
/// - Row 1: Dots (1-9). 9 tiles * 4 bits = 36 bits.
/// - Row 2: Bamboos (1-9). 9 tiles * 4 bits = 36 bits.
/// - Row 3: Honors (7 types) + Flowers (8 types).
///   - Honors: 7 * 4 bits = 28 bits.
///   - Flowers: 8 * 1 bit = 8 bits.
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
pub trait MahjongBitArray: Copy + AsRef<[u64]> + AsMut<[u64]> {
    /// Adds a tile to a row value at the specified shift.
    /// Returns true if successful (count < 4), or false if full.
    #[inline(always)]
    fn add_to_row(row: &mut u64, shift: usize) -> bool {
        let slot_val = (*row >> shift) & 0xF;
        if slot_val == 0xF {
            return false;
        }
        let bit_to_add = slot_val + 1;
        *row |= bit_to_add << shift;
        true
    }

    /// Adds a tile to the hand.
    /// Returns true if successful (count < 4).
    #[inline(always)]
    fn add_tile(&mut self, id: u8) -> bool {
        let id = id as usize;
        let row = id >> 6; // id / 64
        let shift = id & 0x3F; // id % 64
        let hand = self.as_mut();

        if id >= 220 {
            let slot_val = (hand[row] >> shift) & 0xF;
            if slot_val != 0 {
                return false;
            }
        }

        Self::add_to_row(&mut hand[row], shift)
    }

    /// Removes a tile from a row value at the specified shift.
    /// Returns true if successful (count > 0), false if empty.
    #[inline(always)]
    fn remove_from_row(row: &mut u64, shift: usize) -> bool {
        let slot_val = (*row >> shift) & 0xF;
        if slot_val == 0 {
            return false;
        }
        let bit_to_remove = slot_val ^ (slot_val >> 1);
        *row &= !(bit_to_remove << shift);
        true
    }

    /// Removes a tile from the hand.
    /// Returns true if successful (count > 0), false if empty.
    #[inline(always)]
    fn remove_tile(&mut self, id: u8) -> bool {
        let id = id as usize;
        let row = id >> 6;
        let shift = id & 0x3F;
        let hand = self.as_mut();

        Self::remove_from_row(&mut hand[row], shift)
    }

    /// Counts the number of copies of a tile.
    #[inline(always)]
    fn count_tile(&self, id: u8) -> u32 {
        let id = id as usize;
        let row = id >> 6;
        let shift = id & 0x3F;
        let hand = self.as_ref();

        let slot_val = (hand[row] >> shift) & 0xF;
        slot_val.count_ones()
    }

    /// Checks if there are at least `count` copies of a tile.
    #[inline(always)]
    fn has_tile_count_geq(&self, id: u8, count: u32) -> bool {
        if count == 0 {
            return true;
        }
        if count > 4 {
            return false;
        }
        let id = id as usize;
        let row = id >> 6;
        let shift = id & 0x3F;
        let hand = self.as_ref();

        let bit_idx = count - 1;
        let mask = 1 << bit_idx;
        let slot_val = (hand[row] >> shift) & 0xF;
        (slot_val & mask) != 0
    }

    /// Adds multiple copies of a tile to a row.
    #[inline(always)]
    fn add_tiles_to_row(row: &mut u64, shift: usize, count: u32) -> bool {
        if count == 0 {
            return true;
        }
        if count > 4 {
            return false;
        }
        let slot_val = (*row >> shift) & 0xF;
        let current_count = slot_val.count_ones();

        if current_count + count > 4 {
            return false;
        }

        let mask = ((1 << count) - 1) << current_count;
        *row |= mask << shift;
        true
    }

    /// Adds multiple copies of a tile.
    #[inline(always)]
    fn add_tiles(&mut self, id: u8, count: u32) -> bool {
        let id = id as usize;
        let row = id >> 6;
        let shift = id & 0x3F;
        let hand = self.as_mut();

        Self::add_tiles_to_row(&mut hand[row], shift, count)
    }

    /// Removes multiple copies of a tile from a row.
    #[inline(always)]
    fn remove_tiles_from_row(row: &mut u64, shift: usize, count: u32) -> bool {
        if count == 0 {
            return true;
        }
        if count > 4 {
            return false;
        }
        let slot_val = (*row >> shift) & 0xF;
        if slot_val.count_ones() < count {
            return false;
        }

        let mask = slot_val ^ (slot_val >> count);
        *row &= !(mask << shift);
        true
    }

    /// Removes multiple copies of a tile.
    #[inline(always)]
    fn remove_tiles(&mut self, id: u8, count: u32) -> bool {
        let id = id as usize;
        let row = id >> 6;
        let shift = id & 0x3F;
        let hand = self.as_mut();

        Self::remove_tiles_from_row(&mut hand[row], shift, count)
    }

    /// Finds sequences (Chows) in all rows.
    /// Returns a BitArray where each row contains bitmasks of sequences found.
    #[inline(always)]
    fn find_sequences_for_row(row: &u64) -> u64 {
        row & (row >> 4) & (row >> 8)
    }

    /// Finds sequences (Chows) in a specific row.
    /// Returns a bitmask where bit `i` set means a sequence of (i, i+4, i+8)
    /// starts at tile index `i`
    #[inline(always)]
    fn find_sequences(&self, row_idx: usize) -> u64 {
        if row_idx >= 3 {
            return 0;
        } // Honors/Flowers don't form sequences

        let hand = self.as_ref();

        Self::find_sequences_for_row(&hand[row_idx])
    }

    /// Removes a sequence (Chow) from a row starting at the specified shift.
    /// Repects LSB fill.
    #[inline(always)]
    fn remove_sequences_from_row(row: &mut u64, shift: usize) -> bool {
        Self::remove_from_row(row, shift)
            && Self::remove_from_row(row, shift + 4)
            && Self::remove_from_row(row, shift + 8)
    }

    /// Bitwise AND with another MahjongBitArray.
    #[inline(always)]
    fn bit_and(&mut self, other: Self) -> Self {
        let hand = self.as_mut();
        for (i, row) in hand.iter_mut().enumerate() {
            *row &= other.as_ref()[i];
        }
        *self
    }
    /// Bitwise OR with another MahjongBitArray.
    #[inline(always)]
    fn bit_or(&mut self, other: Self) -> Self {
        let hand = self.as_mut();
        for (i, row) in hand.iter_mut().enumerate() {
            *row |= other.as_ref()[i];
        }
        *self
    }

    fn to_tiles(&self, capacity: usize) -> Vec<Tile> {
        let mut tiles = Vec::with_capacity(capacity);
        let mask = self.as_ref();

        for (idx, &row) in mask.iter().enumerate() {
            let mut row_mut = row;
            while row_mut != 0 {
                let shift = row_mut.trailing_zeros() as usize;
                tiles.push(Tile::from_indices(idx, shift).unwrap());
                BitArray::remove_from_row(&mut row_mut, shift);
            }
        }

        tiles
    }

    const NON_FLOWER_MASK: Self;
    const FLOWER_MASK: Self;
    const UNIQUE_MASK: Self;

    fn to_non_flower_tiles(&self) -> Vec<Tile> {
        let mut masked = *self;
        masked.bit_and(Self::NON_FLOWER_MASK).to_tiles(14)
    }

    fn to_flower_tiles(&self) -> Vec<Tile> {
        let mut masked = *self;
        masked.bit_and(Self::FLOWER_MASK).to_tiles(4)
    }

    fn to_unique_tiles(&self) -> Vec<Tile> {
        let mut masked = *self;
        masked.bit_and(Self::UNIQUE_MASK).to_tiles(14)
    }
}

pub type BitArray = [u64; 4];

impl MahjongBitArray for BitArray {
    const NON_FLOWER_MASK: Self =
        [0xFFFFFFFFF, 0xFFFFFFFFF, 0xFFFFFFFFF, 0xFFFFFFF];
    const FLOWER_MASK: Self = [0x0, 0x0, 0x0, 0xFFFFFFFF0000000];
    const UNIQUE_MASK: Self =
        [0x111111111, 0x111111111, 0x111111111, 0x111111111111111];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    // Helper to get a clean hand
    fn new_hand() -> [u64; 4] {
        [0; 4]
    }

    #[test]
    fn test_add_to_row() {
        let mut row = 0u64;
        // Shift 0 (Tile 1)
        assert!(<[u64; 4] as MahjongBitArray>::add_to_row(&mut row, 0));
        assert_eq!(row, 0b0001);
        assert!(<[u64; 4] as MahjongBitArray>::add_to_row(&mut row, 0));
        assert_eq!(row, 0b0011);
        assert!(<[u64; 4] as MahjongBitArray>::add_to_row(&mut row, 0));
        assert_eq!(row, 0b0111);
        assert!(<[u64; 4] as MahjongBitArray>::add_to_row(&mut row, 0));
        assert_eq!(row, 0b1111);
        // Max 4
        assert!(!<[u64; 4] as MahjongBitArray>::add_to_row(&mut row, 0));
        assert_eq!(row, 0b1111);

        // Shift 4 (Tile 2)
        assert!(<[u64; 4] as MahjongBitArray>::add_to_row(&mut row, 4));
        assert_eq!(row, 0b0001_1111);
    }

    #[test]
    fn test_remove_from_row() {
        let mut row = 0b0011_0111; // Tile 2 count 2, Tile 1 count 3

        // Remove from shift 0 (Tile 1)
        assert!(<[u64; 4] as MahjongBitArray>::remove_from_row(&mut row, 0));
        // 3 (0111) -> 2 (0011). Row: 0011_0011
        assert_eq!(row, 0b0011_0011);

        // Remove from shift 4 (Tile 2)
        assert!(<[u64; 4] as MahjongBitArray>::remove_from_row(&mut row, 4));
        // 2 (0011) -> 1 (0001). Row: 0001_0011
        assert_eq!(row, 0b0001_0011);

        // Empty
        let mut empty_row = 0u64;
        assert!(!<[u64; 4] as MahjongBitArray>::remove_from_row(
            &mut empty_row,
            0
        ));
    }

    #[test]
    fn test_add_remove_tile_state() {
        let mut hand = new_hand();
        let t1 = Tile::Character1.id(); // Row 0, Shift 0
        let t2 = Tile::Character2.id(); // Row 0, Shift 4

        hand.add_tile(t1);
        assert_eq!(hand[0], 0x1);

        hand.add_tile(t1);
        assert_eq!(hand[0], 0x3);

        hand.add_tile(t2);
        assert_eq!(hand[0], 0x13); // 0001_0011

        hand.remove_tile(t1);
        assert_eq!(hand[0], 0x11); // 0001_0001
    }

    #[test]
    fn test_add_remove_tiles_bulk() {
        let mut hand = new_hand();
        let t = Tile::Dot1.id(); // Row 1, Shift 0

        // Add 3
        assert!(hand.add_tiles(t, 3));
        assert_eq!(hand[1], 0x7); // 0111

        // Add 2 more (should fail, max 4)
        assert!(!hand.add_tiles(t, 2));
        assert_eq!(hand[1], 0x7);

        // Add 1 more (should succeed -> 4)
        assert!(hand.add_tiles(t, 1));
        assert_eq!(hand[1], 0xF); // 1111

        // Remove 2
        assert!(hand.remove_tiles(t, 2));
        // 4 (1111) -> 2 (0011)
        assert_eq!(hand[1], 0x3);
    }

    #[test]
    fn test_sequences() {
        let mut hand = new_hand();
        // Sequence 1-2-3 in Row 0 (Characters)
        // Char1 (shift 0): 1 (0001)
        // Char2 (shift 4): 1 (0001)
        // Char3 (shift 8): 1 (0001)
        // Row 0: 0001_0001_0001 = 0x111
        hand[0] = 0x111;

        let seqs = hand.find_sequences(0);
        // Expect bit 0 set.
        assert_eq!(seqs, 1);

        // Sequence 2-3-4
        // Char2 (shift 4): 1
        // Char3 (shift 8): 1
        // Char4 (shift 12): 1
        // Row 0: 0001_0001_0001_0000 = 0x1110
        hand[0] = 0x1110;
        let seqs = hand.find_sequences(0);
        // Expect bit 4 set.
        assert_eq!(seqs, 1 << 4);

        // Mixed: 1-2-3 and 3-4-5
        // Char1: 1
        // Char2: 1
        // Char3: 2 (0011)
        // Char4: 1
        // Char5: 1
        // Row 0: 0001_0001_0011_0001_0001
        //        5    4    3    2    1
        // Hex: 1 1 3 1 1
        hand[0] = 0x11311;
        let seqs = hand.find_sequences(0);
        // 1-2-3 starts at 0.
        // 2-3-4 starts at 4. (Char2, Char3, Char4 all have count > 0)
        // 3-4-5 starts at 8.
        // Expected: (1<<0) | (1<<4) | (1<<8) = 1 | 16 | 256 = 273
        assert_eq!(seqs, 1 | (1 << 4) | (1 << 8));

        BitArray::remove_sequences_from_row(&mut hand[0], 0);
        assert_eq!(hand[0], 0x11100); // 123 removed
    }

    #[test]
    fn test_flowers_state() {
        let mut hand = new_hand();
        let f = Tile::Plum.id(); // Row 3, Shift 28 (220 % 64 = 28)

        assert!(hand.add_tile(f));
        // Bit at 28 set to 1.
        assert_eq!(hand[3], 1 << 28);

        // Try add again (should fail for flower)
        assert!(!hand.add_tile(f));
        assert_eq!(hand[3], 1 << 28);

        assert!(hand.remove_tile(f));
        assert_eq!(hand[3], 0);
    }

    #[test]
    fn test_to_tiles() {
        let mut hand = new_hand();
        // Add Character 1 x2, Character 9x1, Dot 1 x1, Dot 5 x1,
        // Dot 9 x1, Bamboo 1 x1, Bamboo 9 x3, East x1, White x1, Plum x1
        hand.add_tiles(Tile::Character1.id(), 2);
        hand.add_tile(Tile::Character9.id());
        hand.add_tile(Tile::Dot1.id());
        hand.add_tiles(Tile::Dot5.id(), 1);
        hand.add_tile(Tile::Dot9.id());
        hand.add_tile(Tile::Bamboo1.id());
        hand.add_tiles(Tile::Bamboo9.id(), 3);
        hand.add_tile(Tile::East.id()); // Honor
        hand.add_tile(Tile::White.id()); // Honor
        hand.add_tiles(Tile::Plum.id(), 1); // Flower
        let tiles = hand.to_tiles(14);
        let mut expected_tiles = vec![
            Tile::Character1,
            Tile::Character1,
            Tile::Character9,
            Tile::Dot1,
            Tile::Dot5,
            Tile::Dot9,
            Tile::Bamboo1,
            Tile::Bamboo9,
            Tile::Bamboo9,
            Tile::Bamboo9,
            Tile::East,
            Tile::White,
            Tile::Plum,
        ];
        tiles.iter().for_each(|t| {
            if let Some(pos) = expected_tiles.iter().position(|&et| et == *t) {
                expected_tiles.remove(pos);
            } else {
                panic!("Unexpected tile {:?}", t);
            }
        });
        assert!(expected_tiles.is_empty());
    }

    #[test]
    fn test_masks() {
        let hand = [0xFFFFFFFFF, 0xFFFFFFFFF, 0xFFFFFFFFF, 0x11111111FFFFFFF];

        let mut hand_copy = hand;
        let non_flower_mask = hand_copy.bit_and(BitArray::NON_FLOWER_MASK);
        assert_eq!(
            non_flower_mask,
            [0xFFFFFFFFF, 0xFFFFFFFFF, 0xFFFFFFFFF, 0xFFFFFFF]
        );

        let mut hand_copy = hand;
        let flower_mask = hand_copy.bit_and(BitArray::FLOWER_MASK);
        assert_eq!(flower_mask, [0x0, 0x0, 0x0, 0x111111110000000]);

        let mut hand_copy = hand;
        let unique_mask = hand_copy.bit_and(BitArray::UNIQUE_MASK);
        assert_eq!(
            unique_mask,
            [0x111111111, 0x111111111, 0x111111111, 0x111111111111111]
        );
    }
}
