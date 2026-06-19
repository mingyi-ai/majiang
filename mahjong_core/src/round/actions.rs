use crate::structs::{
    BitTileCounts, Hand, Meld, Quad, Sequence, Tile, Triplet,
};

// Basic actions
impl Hand {
    pub(crate) fn add_tile(&mut self, tile: Tile) {
        if tile.is_flower() {
            self.flower_count += 1;
        } else {
            self.concealed.insert(tile);
        }
    }

    pub(crate) fn remove_tile(&mut self, tile: Tile, count: u8) {
        if tile.is_flower() {
            debug_assert!(
                self.flower_count >= count,
                "Cannot remove more flower tiles than currently present"
            );
            self.flower_count -= count;
        } else {
            self.concealed.remove(tile, count);
        }
    }
}

// Queries related to the hand's state, such as checking if a tile can be used for a specific action (Pong, Chow, Kong) based on the current concealed hand and melds
// for pong, kong, chow, query tiles are not supposed to be in the hand
impl Hand {
    /// Checks if the player can declare a Pong reaction with `tile` (i.e., has at least 2 copies in the concealed hand).
    pub(crate) fn can_pong(&self, discard: Tile) -> bool {
        self.concealed.count(discard) >= 2
    }

    /// Checks if the player can declare a Kong reaction with `tile` (i.e., has at least 3 copies in the concealed hand).
    pub(crate) fn can_kong(&self, discard: Tile) -> bool {
        self.concealed.count(discard) >= 3
    }

    /// Returns the start tiles of all possible chows that include `tile`
    /// as one of the three sequence tiles.
    pub(crate) fn chow_start_options(&self, discard: Tile) -> Vec<Tile> {
        if !discard.is_suit() {
            return vec![];
        }

        let mut concealed = self.concealed; // Copy (BitTileCounts is Copy)
        concealed.insert(discard); // simulate having the discard

        let (row, shift) = BitTileCounts::tile_to_position(discard);
        let seqs = BitTileCounts::find_sequences_for_row(concealed.rows[row]);

        let mut results = Vec::new();
        // The discard can be the 1st, 2nd, or 3rd tile of a sequence.
        // For position i (0=1st, 1=2nd, 2=3rd), the sequence starts at shift - i*4.
        for offset in [0, 4, 8] {
            if offset <= shift && shift - offset <= 28 {
                let start = shift - offset;
                if seqs & (1 << start) != 0 {
                    results.push(BitTileCounts::position_to_tile(row, start));
                }
            }
        }

        results
    }

    /// Check if the hand can declare hu with `tile` as the winning tile
    /// (claimed from a discard).
    pub(crate) fn can_hu_on(&self, discard: Tile) -> bool {
        let mut simulated_hand = *self; // Copy (Hand is Copy)
        simulated_hand.add_tile(discard);
        simulated_hand.concealed_tiles_are_hu()
    }

    /// Check if the hand can declare hu with the current concealed hand
    /// (i.e., self-draw winnig tile).
    pub(crate) fn can_hu(&self) -> bool {
        self.concealed_tiles_are_hu()
    }

    /// Delegates to `HuSolver::is_hu` for the concealed tiles.
    /// Extracted so the solver type isn't imported at every call site.
    fn concealed_tiles_are_hu(&self) -> bool {
        crate::solver::HuSolver::is_hu(&self.concealed)
    }

    /// Returns tiles where the player has an exposed Pong meld
    /// AND at least 1 more of that tile in the concealed hand,
    /// allowing an upgrade to an exposed Kong.
    pub(crate) fn added_kong_options(&self) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for meld in &self.melds {
            if let Some(Meld::Pung(t)) = meld {
                let t = *t;
                if !t.is_concealed() && self.concealed.count(t.tile()) >= 1 {
                    tiles.push(t.tile());
                }
            }
        }
        tiles
    }

    /// Returns tiles where the concealed hand has exactly 4 copies,
    /// allowing a concealed Kong declaration.
    pub(crate) fn concealed_kong_options(&self) -> Vec<Tile> {
        Tile::iter()
            .filter(|t| !t.is_flower() && self.concealed.count(*t) >= 4)
            .collect()
    }
}

// Actions related to modifying the hand based on player actions (Pong, Chow, Kong) and their effects on the concealed hand and melds
impl Hand {
    fn push_meld(&mut self, meld: Meld) {
        // The action path never constructs concealed Pungs or Chows —
        // concealed melds only exist in the fan/hu solver path.
        match meld {
            Meld::Pung(t) if t.is_concealed() => {
                panic!(
                    "Concealed Pung must not be created via the action path"
                );
            }
            Meld::Chow(s) if s.is_concealed() => {
                panic!(
                    "Concealed Chow must not be created via the action path"
                );
            }
            _ => {}
        }
        let slot = self
            .melds
            .iter_mut()
            .find(|m| m.is_none())
            .expect("No empty slot available for new meld");
        *slot = Some(meld);
    }

    /// Creates a Pong meld from discarded `tile`.
    pub(crate) fn pong(&mut self, tile: Tile) {
        self.push_meld(Meld::Pung(Triplet::new(tile, false)));
        self.concealed.remove(tile, 2);
    }

    /// Creates a Chow meld from `start_tile` using the 2 tiles from the player's hand.
    /// `tile_from_discard` is the tile claimed from the discard, which should be one of the three tiles in the chow sequence.
    pub(crate) fn chow(&mut self, start: Tile, discard: Tile) {
        let seq = Sequence::new(start, false);
        self.push_meld(Meld::Chow(seq));

        for tile in seq.tiles() {
            if tile == discard {
                continue; // Skip the tile that came from the discard
            }
            self.concealed.remove(tile, 1);
        }
    }

    /// Creates a Kong meld from `tile`.
    ///
    /// - `is_concealed == true`: all 4 tiles came from the concealed hand, remove 4.
    /// - `is_concealed == false`: 3 tiles from concealed + 1 from discard, remove 3.
    pub(crate) fn kong(&mut self, tile: Tile, is_concealed: bool) {
        self.push_meld(Meld::Kong(Quad::new(tile, is_concealed)));

        let remove_count = if is_concealed { 4 } else { 3 };
        self.concealed.remove(tile, remove_count);
    }

    pub(crate) fn kong_from_pong(&mut self, tile: Tile) {
        let pong_index = self
            .melds
            .iter()
            .position(|m| matches!(m, Some(Meld::Pung(t)) if t.tile() == tile))
            .expect("No existing Pong meld found for the specified tile");

        // The upgraded kong is always considered exposed (concealed = false)
        self.melds[pong_index] = Some(Meld::Kong(Quad::new(tile, false)));

        self.concealed.remove(tile, 1); // Remove the additional tile needed to upgrade the Pong to a Kong
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──

    /// Create a Hand containing the given non-flower tiles (each with count 1).
    fn hand_with(tiles: &[Tile]) -> Hand {
        let mut h = Hand::default();
        for &t in tiles {
            h.concealed.insert(t);
        }
        h
    }

    // ── add_tile ──

    #[test]
    fn add_tile_non_flower_inserts_into_concealed() {
        let mut h = Hand::default();
        h.add_tile(Tile::Character1);
        assert_eq!(h.concealed.count(Tile::Character1), 1);
        assert_eq!(h.flower_count, 0);
    }

    #[test]
    fn add_tile_flower_increments_flower_count() {
        let mut h = Hand::default();
        h.add_tile(Tile::Plum);
        assert_eq!(h.flower_count, 1);
        // Flower tiles don't go into concealed
        assert!(h.concealed.rows.iter().all(|&r| r == 0));
    }

    #[test]
    fn add_tile_multiple_flowers() {
        let mut h = Hand::default();
        h.add_tile(Tile::Plum);
        h.add_tile(Tile::Orchid);
        h.add_tile(Tile::BambooF);
        assert_eq!(h.flower_count, 3);
    }

    // ── remove_tile ──

    #[test]
    fn remove_tile_non_flower_removes_from_concealed() {
        let mut h = hand_with(&[Tile::Dot3, Tile::Dot3]);
        h.remove_tile(Tile::Dot3, 1);
        assert_eq!(h.concealed.count(Tile::Dot3), 1);
    }

    #[test]
    fn remove_tile_flower_decrements_flower_count() {
        let mut h = Hand::default();
        h.add_tile(Tile::Spring);
        h.add_tile(Tile::Summer);
        h.remove_tile(Tile::Spring, 1);
        assert_eq!(h.flower_count, 1);
    }

    #[test]
    #[should_panic(
        expected = "Cannot remove more flower tiles than currently present"
    )]
    fn remove_tile_flower_panics_on_underflow() {
        let mut h = Hand::default();
        h.remove_tile(Tile::Plum, 1);
    }

    // --- push meld ---

    #[test]
    fn push_meld_uses_next_empty_slot() {
        let mut h = Hand::default();
        h.push_meld(Meld::Pung(Triplet::new(Tile::East, false)));
        h.push_meld(Meld::Chow(Sequence::new(Tile::Character3, false)));

        assert!(h.melds[0].is_some());
        assert!(h.melds[1].is_some());
        assert!(h.melds[2].is_none());
    }

    // ── can_pong ──

    #[test]
    fn can_pong_true_with_two_copies() {
        let h = hand_with(&[Tile::Bamboo5, Tile::Bamboo5]);
        assert!(h.can_pong(Tile::Bamboo5));
    }

    #[test]
    fn can_pong_false_with_one_copy() {
        let h = hand_with(&[Tile::Bamboo5]);
        assert!(!h.can_pong(Tile::Bamboo5));
    }

    #[test]
    fn can_pong_false_with_zero_copies() {
        let h = hand_with(&[Tile::Character1]);
        assert!(!h.can_pong(Tile::Bamboo5));
    }

    #[test]
    fn can_pong_true_with_three_copies() {
        let h = hand_with(&[Tile::East, Tile::East, Tile::East]);
        assert!(h.can_pong(Tile::East));
    }

    // ── can_kong ──

    #[test]
    fn can_kong_true_with_three_copies() {
        let h = hand_with(&[Tile::Red, Tile::Red, Tile::Red]);
        assert!(h.can_kong(Tile::Red));
    }

    #[test]
    fn can_kong_false_with_two_copies() {
        let h = hand_with(&[Tile::Red, Tile::Red]);
        assert!(!h.can_kong(Tile::Red));
    }

    #[test]
    fn can_kong_false_with_zero_copies() {
        let h = Hand::default();
        assert!(!h.can_kong(Tile::Dot1));
    }

    // ── chow_start_options ──

    #[test]
    fn chow_start_options_returns_empty_for_non_suit() {
        let h = hand_with(&[Tile::East, Tile::South, Tile::West]);
        assert!(h.chow_start_options(Tile::East).is_empty());
        assert!(h.chow_start_options(Tile::Red).is_empty());
        assert!(h.chow_start_options(Tile::Plum).is_empty());
    }

    #[test]
    fn chow_start_options_discard_is_middle() {
        // Hand has: Character1, Character2, Character3 (each 1 copy)
        // Discard is Character2 (middle of sequence)
        let h =
            hand_with(&[Tile::Character1, Tile::Character2, Tile::Character3]);
        let options = h.chow_start_options(Tile::Character2);
        assert_eq!(options, vec![Tile::Character1]);
    }

    #[test]
    fn chow_start_options_discard_is_first() {
        // Hand has: Character1, Character2, Character3 (each 1 copy)
        // Discard is Character1 (first of sequence)
        let h =
            hand_with(&[Tile::Character1, Tile::Character2, Tile::Character3]);
        let options = h.chow_start_options(Tile::Character1);
        assert_eq!(options, vec![Tile::Character1]);
    }

    #[test]
    fn chow_start_options_discard_is_last() {
        // Hand has: Character1, Character2, Character3 (each 1 copy)
        // Discard is Character3 (last of sequence)
        let h =
            hand_with(&[Tile::Character1, Tile::Character2, Tile::Character3]);
        let options = h.chow_start_options(Tile::Character3);
        assert_eq!(options, vec![Tile::Character1]);
    }

    #[test]
    fn chow_start_options_discard_completes_sequence() {
        // Hand has only: Character1, Character2
        // Discard is Character3 — now they form a sequence
        let h = hand_with(&[Tile::Character1, Tile::Character2]);
        let options = h.chow_start_options(Tile::Character3);
        assert_eq!(options, vec![Tile::Character1]);
    }

    #[test]
    fn chow_start_options_no_sequence_possible() {
        // Hand has: Character1, Character4, Character5
        // Discard is Character5, simulated adds 5, now we have 1,4,5 which is not a sequence.
        let h =
            hand_with(&[Tile::Character1, Tile::Character4, Tile::Character5]);
        let options = h.chow_start_options(Tile::Character5);
        assert!(options.is_empty());
    }

    #[test]
    fn chow_start_options_discard_at_boundary() {
        // Character1 at shift 0 — no chow can start at negative offset
        let h =
            hand_with(&[Tile::Character1, Tile::Character2, Tile::Character3]);
        let options = h.chow_start_options(Tile::Character1);
        // Discard at shift 0: only possible as first tile of a chow starting at 0
        assert_eq!(options, vec![Tile::Character1]);
    }

    #[test]
    fn chow_start_options_multiple_options() {
        // Hand: 2,3,4,5 (each 1 copy). Discard: 3.
        // Discard=3 can be: middle of 2-3-4 (start=2), or first of 3-4-5 (start=3)
        let h = hand_with(&[
            Tile::Character2,
            Tile::Character3,
            Tile::Character4,
            Tile::Character5,
        ]);
        let mut options = h.chow_start_options(Tile::Character3);
        options.sort();
        assert_eq!(options, vec![Tile::Character2, Tile::Character3]);
    }

    // ── pong ──

    #[test]
    fn pong_creates_meld_and_removes_two_tiles() {
        let mut h =
            hand_with(&[Tile::East, Tile::East, Tile::East, Tile::Character1]);
        h.pong(Tile::East);

        // Melds: 1 slot filled with Pung(East, exposed)
        assert!(h.melds[0].is_some());
        let meld = h.melds[0].unwrap();
        if let Meld::Pung(t) = meld {
            assert_eq!(t.tile(), Tile::East);
            assert!(!t.is_concealed());
        } else {
            panic!("expected Pung");
        }

        // 2 copies of East removed from concealed; 1 should remain
        assert_eq!(h.concealed.count(Tile::East), 1);
        // Other tiles untouched
        assert_eq!(h.concealed.count(Tile::Character1), 1);
    }

    // ── chow ──

    #[test]
    fn chow_creates_meld_and_removes_two_concealed_tiles() {
        // Hand: 3,4 (each 1). Discard: 5 (from outside). Start: 3.
        // Tiles 3 and 4 are removed from concealed (they came from the hand).
        // Tile 5 came from the discard pile and is not in concealed.
        let mut h = hand_with(&[Tile::Character3, Tile::Character4]);
        h.chow(Tile::Character3, Tile::Character5);

        // Melds: 1 slot filled with Chow(3, exposed)
        assert!(h.melds[0].is_some());
        let meld = h.melds[0].unwrap();
        if let Meld::Chow(s) = meld {
            assert_eq!(s.start(), Tile::Character3);
            assert!(!s.is_concealed());
        } else {
            panic!("expected Chow");
        }

        // Tiles 3 and 4 removed from concealed (the two that came from the hand)
        assert_eq!(h.concealed.count(Tile::Character3), 0);
        assert_eq!(h.concealed.count(Tile::Character4), 0);
    }

    #[test]
    fn chow_keeps_discard_tile_in_hand() {
        // Hand: 3,4 (each 1). Discard: 5 (from outside). Start: 3.
        // Only tiles 3 and 4 should be removed from concealed (discard 5 came from outside)
        let mut h = hand_with(&[Tile::Character3, Tile::Character4]);
        h.chow(Tile::Character3, Tile::Character5);

        assert_eq!(h.concealed.count(Tile::Character3), 0);
        assert_eq!(h.concealed.count(Tile::Character4), 0);
    }

    // ── kong ──

    #[test]
    fn kong_exposed_removes_three_tiles() {
        let mut h = hand_with(&[
            Tile::Red,
            Tile::Red,
            Tile::Red,
            Tile::Red,
            Tile::Character1,
        ]);
        h.kong(Tile::Red, false); // exposed: 3 from hand + 1 from discard

        assert!(h.melds[0].is_some());
        let meld = h.melds[0].unwrap();
        if let Meld::Kong(q) = meld {
            assert_eq!(q.tile(), Tile::Red);
            assert!(!q.is_concealed());
        } else {
            panic!("expected Kong");
        }

        // 3 copies removed from concealed; 1 should remain
        assert_eq!(h.concealed.count(Tile::Red), 1);
        assert_eq!(h.concealed.count(Tile::Character1), 1);
    }

    #[test]
    fn kong_concealed_removes_four_tiles() {
        let mut h =
            hand_with(&[Tile::White, Tile::White, Tile::White, Tile::White]);
        h.kong(Tile::White, true); // concealed: all 4 from hand

        assert!(h.melds[0].is_some());
        let meld = h.melds[0].unwrap();
        if let Meld::Kong(q) = meld {
            assert_eq!(q.tile(), Tile::White);
            assert!(q.is_concealed());
        } else {
            panic!("expected concealed Kong");
        }

        assert_eq!(h.concealed.count(Tile::White), 0);
    }

    // ── kong_from_pong ──

    #[test]
    fn kong_from_pong_upgrades_pung_to_kong() {
        let mut h = hand_with(&[
            Tile::Bamboo5,
            Tile::Bamboo5,
            Tile::Bamboo5,
            Tile::Bamboo5, // extra copy for upgrade
        ]);
        h.pong(Tile::Bamboo5); // creates exposed Pung, removes 2, leaves 2

        assert_eq!(h.concealed.count(Tile::Bamboo5), 2);

        h.kong_from_pong(Tile::Bamboo5);

        // Meld slot 0 should now be a Kong (not Pung)
        let meld = h.melds[0].unwrap();
        if let Meld::Kong(q) = meld {
            assert_eq!(q.tile(), Tile::Bamboo5);
            assert!(!q.is_concealed()); // upgraded kong is exposed
        } else {
            panic!("expected Kong after upgrade");
        }

        // 1 more tile removed for the upgrade
        assert_eq!(h.concealed.count(Tile::Bamboo5), 1);
    }

    #[test]
    #[should_panic(expected = "No existing Pong meld found")]
    fn kong_from_pong_panics_without_existing_pung() {
        let mut h = Hand::default();
        h.kong_from_pong(Tile::East);
    }

    // ── push_meld overflow ──

    #[test]
    #[should_panic(expected = "No empty slot available for new meld")]
    fn push_meld_panics_when_all_slots_full() {
        let mut h = Hand::default();
        h.push_meld(Meld::Pung(Triplet::new(Tile::East, false)));
        h.push_meld(Meld::Pung(Triplet::new(Tile::South, false)));
        h.push_meld(Meld::Pung(Triplet::new(Tile::West, false)));
        h.push_meld(Meld::Pung(Triplet::new(Tile::North, false)));
        h.push_meld(Meld::Pung(Triplet::new(Tile::Red, false))); // 5th
    }

    // ── added_kong_options ──

    #[test]
    fn added_kong_options_finds_upgradeable_pungs() {
        let mut h =
            hand_with(&[Tile::East, Tile::East, Tile::East, Tile::East]);
        h.pong(Tile::East); // exposed Pung, removes 2, leaves 2

        let options = h.added_kong_options();
        assert_eq!(options, vec![Tile::East]);
    }

    #[test]
    #[should_panic(
        expected = "Concealed Pung must not be created via the action path"
    )]
    fn push_meld_panics_on_concealed_pung() {
        let mut h = Hand::default();
        h.push_meld(Meld::Pung(Triplet::new(Tile::Red, true)));
    }

    #[test]
    #[should_panic(
        expected = "Concealed Chow must not be created via the action path"
    )]
    fn push_meld_panics_on_concealed_chow() {
        let mut h = Hand::default();
        h.push_meld(Meld::Chow(Sequence::new(Tile::Character3, true)));
    }

    #[test]
    fn added_kong_options_skips_without_extra_tile() {
        let mut h = hand_with(&[Tile::Dot1, Tile::Dot1]);
        h.pong(Tile::Dot1); // removes 2, now 0 concealed

        let options = h.added_kong_options();
        assert!(options.is_empty(), "no extra tile for upgrade");
    }

    #[test]
    fn added_kong_options_multiple_options() {
        let mut h = hand_with(&[
            Tile::East,
            Tile::East,
            Tile::East,
            Tile::East,
            Tile::South,
            Tile::South,
            Tile::South,
            Tile::South,
        ]);
        h.pong(Tile::East);
        h.pong(Tile::South);

        let mut options = h.added_kong_options();
        options.sort();
        assert_eq!(options, vec![Tile::East, Tile::South]);
    }

    // ── concealed_kong_options ──

    #[test]
    fn concealed_kong_options_finds_quadruple_tiles() {
        let h = hand_with(&[
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
        ]);
        let options = h.concealed_kong_options();
        assert_eq!(options, vec![Tile::Character1]);
    }

    #[test]
    fn concealed_kong_options_excludes_flowers() {
        // Flowers shouldn't appear in concealed_kong_options
        // (they are not stored in concealed at all, but just in case)
        let h = Hand::default();
        let options = h.concealed_kong_options();
        assert!(options.iter().all(|t| !t.is_flower()));
    }

    #[test]
    fn concealed_kong_options_returns_none_for_counts_below_4() {
        let h = hand_with(&[Tile::Dot5, Tile::Dot5, Tile::Dot5]);
        let options = h.concealed_kong_options();
        assert!(options.is_empty());
    }

    #[test]
    fn concealed_kong_options_multiple_tiles() {
        let h = hand_with(&[
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Dot2,
            Tile::Dot2,
            Tile::Dot2,
            Tile::Dot2,
            Tile::East,
            Tile::East,
            Tile::East,
            Tile::East,
        ]);
        let mut options = h.concealed_kong_options();
        options.sort();
        assert_eq!(options, vec![Tile::Character1, Tile::Dot2, Tile::East]);
    }

    // ── can_hu / can_hu_on (smoke tests via HuSolver delegation) ──

    #[test]
    fn can_hu_returns_true_for_complete_hand() {
        // A complete winning hand: 4 triplets + 1 pair = 14 tiles
        let tiles = [
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Character2,
            Tile::Character2,
            Tile::Character2,
            Tile::Character3,
            Tile::Character3,
            Tile::Character3,
            Tile::Character5,
            Tile::Character5,
            Tile::Character5,
            Tile::Character7,
            Tile::Character7,
        ];
        let h = hand_with(&tiles);
        assert!(h.can_hu());
    }

    #[test]
    fn can_hu_returns_false_for_incomplete_hand() {
        // 14 tiles,
        let tiles = [
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Character2,
            Tile::Character2,
            Tile::Character2,
            Tile::Character3,
            Tile::Character3,
            Tile::Character3,
            Tile::Character5,
            Tile::Character5,
            Tile::Character5,
            Tile::Character7,
            Tile::Character8, // No pair, so not a winning hand
        ];
        let h = hand_with(&tiles);
        assert!(!h.can_hu());
    }

    #[test]
    fn can_hu_on_returns_true_with_winning_discard() {
        // Hand missing one tile to complete the pair
        let tiles = [
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Character2,
            Tile::Character2,
            Tile::Character2,
            Tile::Character3,
            Tile::Character3,
            Tile::Character3,
            Tile::Character5,
            Tile::Character5,
            Tile::Character5,
            Tile::Character7,
        ];
        let h = hand_with(&tiles);
        // Adding Character7 completes the pair
        assert!(h.can_hu_on(Tile::Character7));
    }

    #[test]
    fn can_hu_on_returns_false_with_non_winning_discard() {
        let tiles = [
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Character2,
            Tile::Character2,
            Tile::Character2,
            Tile::Character3,
            Tile::Character3,
            Tile::Character3,
            Tile::Character5,
            Tile::Character5,
            Tile::Character5,
            Tile::Character7,
        ];
        let h = hand_with(&tiles);
        // Adding an unrelated tile doesn't complete the hand
        assert!(!h.can_hu_on(Tile::Character9));
    }
}
