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
    fn tile_to_position(tile: Tile) -> (usize, usize) {
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

    fn remove_from_row(&mut self, row: usize, shift: usize, count: u8) {
        let slot_val = (self.rows[row] >> shift) & TILE_COUNT_MASK;
        debug_assert!(
            slot_val.count_ones() >= count as u32,
            "Cannot remove more tiles than currently present"
        );

        let mask = slot_val ^ (slot_val >> count);
        self.rows[row] &= !(mask << shift);
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
}

#[derive(Default)]
pub struct Hand {
    pub(crate) concealed: BitTileCounts,
    pub(crate) melds: [Option<Meld>; 4],
    pub(crate) flower_count: u8,
}
