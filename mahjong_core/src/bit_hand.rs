use crate::bits::{BitArray, MahjongBitArray};
use crate::hu_solver;
use crate::tile::Tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandError {
    DrawError(&'static str),
    DiscardError(&'static str),
    ActionError(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Meld {
    #[default]
    None,
    Chow(Tile),
    Pong(Tile),
    Kong(Tile),
    ConcealedKong(Tile),
}

#[derive(Debug, Clone, Default)]
pub struct Hand {
    pub tiles: BitArray,
    pub melds: [Meld; 4],
    pub meld_count: usize,
}

impl Hand {
    /// Returns a BitArray where the bit corresponding to
    /// the *start tile* of a valid chow involving `target` is set.
    #[inline]
    pub fn possible_chows(&self, target: Tile) -> BitArray {
        let id = target.id();

        let mut result = [0u64; 4];
        if id >= 192 {
            return result;
        }

        let row_idx = (id >> 6) as usize;
        let shift = (id & 0x3F) as usize;

        let mut row_val = self.tiles[row_idx];
        if !BitArray::add_to_row(&mut row_val, shift) {
            return result;
        }

        let seqs = BitArray::find_sequences_for_row(&row_val);
        // mask to avoid usize overflow
        let mask = 0xFFF000000u64 >> (32 - shift);
        result[row_idx] = seqs & mask;
        result
    }

    #[inline]
    pub fn can_pong(&self, target: Tile) -> bool {
        self.tiles.has_tile_count_geq(target.id(), 2)
    }

    #[inline]
    pub fn can_kong(&self, target: Tile) -> bool {
        self.tiles.has_tile_count_geq(target.id(), 3)
    }

    /// Returns a BitArray where the bit corresponding to
    /// tiles that can form a concealed kong is set.
    #[inline]
    pub fn possible_concealed_kong(&self) -> BitArray {
        const MASK_HIGH: u64 = 0x8888888888888888;

        let mut result = [0u64; 4];

        self.tiles.iter().zip(result.iter_mut()).for_each(
            |(&row_val, result_ref)| {
                *result_ref = (row_val & MASK_HIGH) >> 3;
            },
        );

        result
    }

    /// Returns a BitArray where the bit corresponding to
    /// tiles that can form a kong by adding to an existing pong is set.
    #[inline]
    pub fn possible_kong_from_pong(&self) -> BitArray {
        let mut result = [0u64; 4];
        let hand_tiles = self.tiles.as_ref();

        for meld in self.melds.iter() {
            let tile = match meld {
                Meld::Pong(t) => t,
                _ => continue,
            };

            let id = tile.id() as usize;
            let row = id >> 6;
            let shift = id & 0x3F;

            let is_present = ((hand_tiles[row] >> shift) & 0x1) != 0;

            if !is_present {
                continue;
            }

            result[row] |= 1 << shift;
        }
        result
    }

    #[inline]
    pub fn chow(&mut self, target: Tile, start_tile: Tile) -> bool {
        debug_assert!(
            self.tiles.add_tile(target.id()),
            "Failed to add target tile before chow removal."
        );

        let id = start_tile.id() as usize;
        let row = id >> 6;
        let shift = id & 0x3F;

        let mut success = true;
        let row_ref = &mut self.tiles[row];

        const CHOW_OFFSETS: [usize; 3] = [0, 4, 8];

        for offset in CHOW_OFFSETS.iter() {
            let shifted = shift + offset;
            success &= BitArray::remove_from_row(row_ref, shifted);
        }

        success &= self.add_meld(Meld::Chow(start_tile));

        success
    }

    #[inline(always)]
    pub fn pong(&mut self, target: Tile) -> bool {
        self.tiles.remove_tiles(target.id(), 2)
            && self.add_meld(Meld::Pong(target))
    }

    #[inline(always)]
    pub fn kong(&mut self, target: Tile) -> bool {
        self.tiles.remove_tiles(target.id(), 3)
            && self.add_meld(Meld::Kong(target))
    }

    #[inline(always)]
    pub fn concealed_kong(&mut self, target: Tile) -> bool {
        self.tiles.remove_tiles(target.id(), 4)
            && self.add_meld(Meld::ConcealedKong(target))
    }

    #[inline]
    pub fn added_kong(&mut self, target: Tile) -> bool {
        let meld_to_upgrade = self
            .melds
            .iter_mut()
            .find(|m| matches!(m, Meld::Pong(t) if *t == target));

        if let Some(meld) = meld_to_upgrade
            && self.tiles.remove_tile(target.id())
        {
            *meld = Meld::Kong(target);
            return true;
        }

        false
    }

    #[inline(always)]
    fn add_meld(&mut self, meld: Meld) -> bool {
        if self.meld_count >= 4 {
            return false;
        }
        self.melds[self.meld_count] = meld;
        self.meld_count += 1;
        true
    }

    /// Converts a BitArray mask to a Vec<Tile>.
    /// The capacity parameter is a hint for pre-allocation.
    pub fn mask_to_tiles(
        mask: BitArray,
        capacity: usize,
    ) -> Result<Vec<Tile>, HandError> {
        let mut tiles = Vec::with_capacity(capacity);

        for (idx, &row) in mask.iter().enumerate() {
            let mut row_mut = row;
            while row_mut != 0 {
                let shift = row_mut.trailing_zeros() as usize;
                tiles.push(Tile::from_indices(idx, shift).map_err(|_| {
                    HandError::ActionError("Invalid tile indices in mask.")
                })?);
                BitArray::remove_from_row(&mut row_mut, shift);
            }
        }

        Ok(tiles)
    }

    pub fn can_hu_self(&self) -> bool {
        hu_solver::HuSolver::is_standard_hu(&self.tiles)
    }

    pub fn can_hu(&self, _target: Tile) -> bool {
        let mut temp_hand = self.tiles;
        temp_hand.add_tile(_target.id());
        hu_solver::HuSolver::is_standard_hu(&temp_hand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    #[test]
    fn test_possible_chows() {
        let mut hand = Hand::default();
        // 2, 3 Character
        hand.tiles.add_tile(Tile::Character2.id());
        hand.tiles.add_tile(Tile::Character3.id());

        // Target 1 Character -> 1-2-3
        let mask = hand.possible_chows(Tile::Character1);
        let mut expected = [0u64; 4];
        expected.add_tile(Tile::Character1.id());
        assert_eq!(mask, expected);

        // Target 4 Character -> 2-3-4. Sequence starts at 2.
        let mask = hand.possible_chows(Tile::Character4);
        let mut expected = [0u64; 4];
        expected.add_tile(Tile::Character2.id());
        assert_eq!(mask, expected);

        // Target 5 Character -> No valid chow
        let mask = hand.possible_chows(Tile::Character5);
        assert_eq!(mask, [0u64; 4]);

        // Add 4 Character to hand
        hand.tiles.add_tile(Tile::Character4.id());
        // Now Target 5 Character -> 3-4-5. Sequence starts at 3.
        let mask = hand.possible_chows(Tile::Character5);
        let mut expected = [0u64; 4];
        expected.add_tile(Tile::Character3.id());
        assert_eq!(mask, expected);

        // Target 2 Character -> sequence starts at 2
        let mask = hand.possible_chows(Tile::Character2);
        let mut expected = [0u64; 4];
        expected.add_tile(Tile::Character2.id());
        assert_eq!(mask, expected);

        // Add 2, 3, 4 Character to hand
        // Now 2, 2, 3, 3, 4, 4 Character in hand
        hand.tiles.add_tile(Tile::Character2.id());
        hand.tiles.add_tile(Tile::Character3.id());
        hand.tiles.add_tile(Tile::Character4.id());
        // Target 2 Character -> sequences starting at character 2 with count 2
        let mask = hand.possible_chows(Tile::Character2);
        let mut expected = [0u64; 4];
        expected.add_tiles(Tile::Character2.id(), 2);
        assert_eq!(mask, expected);
    }

    #[test]
    fn test_pong_kong() {
        let mut hand = Hand::default();
        hand.tiles.add_tiles(Tile::Dot1.id(), 2);

        assert!(hand.can_pong(Tile::Dot1));
        assert!(!hand.can_kong(Tile::Dot1));

        hand.tiles.add_tile(Tile::Dot1.id()); // Now 3
        assert!(hand.can_pong(Tile::Dot1));
        assert!(hand.can_kong(Tile::Dot1));
    }

    #[test]
    fn test_chow_execution() {
        let mut hand = Hand::default();
        // 2, 3 Character
        hand.tiles.add_tile(Tile::Character2.id());
        hand.tiles.add_tile(Tile::Character3.id());

        // Chow 1 Character (using 2, 3 from hand)
        // Target is 1. Start tile is 1.
        assert!(hand.chow(Tile::Character1, Tile::Character1));

        // Hand should now be empty of 2 and 3
        assert_eq!(hand.tiles.count_tile(Tile::Character2.id()), 0);
        assert_eq!(hand.tiles.count_tile(Tile::Character3.id()), 0);

        // Meld should be recorded
        assert!(
            matches!(hand.melds[0], Meld::Chow(t) if t == Tile::Character1)
        );
    }

    #[test]
    fn test_pong_execution() {
        let mut hand = Hand::default();
        hand.tiles.add_tiles(Tile::Dot1.id(), 2);

        assert!(hand.pong(Tile::Dot1));
        assert_eq!(hand.tiles.count_tile(Tile::Dot1.id()), 0);
        assert!(matches!(hand.melds[0], Meld::Pong(t) if t == Tile::Dot1));
    }

    #[test]
    fn test_kong_execution() {
        let mut hand = Hand::default();
        hand.tiles.add_tiles(Tile::Dot1.id(), 3);

        assert!(hand.kong(Tile::Dot1));
        assert_eq!(hand.tiles.count_tile(Tile::Dot1.id()), 0);
        assert!(matches!(hand.melds[0], Meld::Kong(t) if t == Tile::Dot1));
    }

    #[test]
    fn test_concealed_kong_execution() {
        let mut hand = Hand::default();
        hand.tiles.add_tiles(Tile::Dot1.id(), 4);

        assert!(hand.concealed_kong(Tile::Dot1));
        assert_eq!(hand.tiles.count_tile(Tile::Dot1.id()), 0);
        assert!(
            matches!(hand.melds[0], Meld::ConcealedKong(t) if t == Tile::Dot1)
        );
    }

    #[test]
    fn test_added_kong_execution() {
        let mut hand = Hand::default();
        // Setup a Pong first
        hand.tiles.add_tiles(Tile::Dot1.id(), 2);
        hand.pong(Tile::Dot1);

        // Add the 4th tile to hand
        hand.tiles.add_tile(Tile::Dot1.id());

        assert!(hand.added_kong(Tile::Dot1));
        assert_eq!(hand.tiles.count_tile(Tile::Dot1.id()), 0);
        assert!(matches!(hand.melds[0], Meld::Kong(t) if t == Tile::Dot1));
    }

    #[test]
    fn test_possible_concealed_kong() {
        let mut hand = Hand::default();
        hand.tiles.add_tiles(Tile::Dot1.id(), 4);
        hand.tiles.add_tiles(Tile::Dot2.id(), 3);

        let mask = hand.possible_concealed_kong();
        assert_eq!(mask.count_tile(Tile::Dot1.id()), 1);
        assert_eq!(mask.count_tile(Tile::Dot2.id()), 0);
    }

    #[test]
    fn test_possible_kong_from_pong() {
        let mut hand = Hand::default();
        // Pong Dot1
        hand.tiles.add_tiles(Tile::Dot1.id(), 2);
        hand.pong(Tile::Dot1);

        // Have 4th Dot1 in hand
        hand.tiles.add_tile(Tile::Dot1.id());

        let mask = hand.possible_kong_from_pong();
        assert_eq!(mask.count_tile(Tile::Dot1.id()), 1);
    }
    #[test]
    fn test_mask_to_tiles() {
        let mut mask = [0u64; 4];
        mask.add_tile(Tile::Character1.id());
        mask.add_tile(Tile::Dot5.id());
        mask.add_tile(Tile::Dot5.id());
        mask.add_tile(Tile::Bamboo9.id());
        let tiles = Hand::mask_to_tiles(mask, 4).unwrap();
        dbg!(&tiles);
        assert_eq!(tiles.len(), 4);
        assert!(tiles.contains(&Tile::Character1));
        assert!(tiles.contains(&Tile::Dot5));
        assert!(tiles.contains(&Tile::Bamboo9));
    }
}
