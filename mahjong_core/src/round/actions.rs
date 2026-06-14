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
    ///
    /// TODO: integrate hu solver — currently always returns false.
    pub(crate) fn can_hu_on(&self, discard: Tile) -> bool {
        let mut simulated_hand = *self; // Copy (Hand is Copy)
        simulated_hand.add_tile(discard);
        simulated_hand.can_hu()
    }

    /// Check if the hand can declare hu with the current concealed hand
    /// (i.e., self-draw winnig tile).
    pub(crate) fn can_hu(&self) -> bool {
        unimplemented!()
    }

    /// Returns tiles where the player has an exposed Pong meld
    /// AND at least 1 more of that tile in the concealed hand,
    /// allowing an upgrade to an exposed Kong.
    pub(crate) fn added_kong_options(&self) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for meld in &self.melds {
            if let Some(Meld::Pung(Triplet {
                tile,
                is_concealed: false,
            })) = meld
                && self.concealed.count(*tile) >= 1
            {
                tiles.push(*tile);
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
        self.push_meld(Meld::Chow(Sequence::new(start, false)));

        for tile in [start, start.next(), start.next().next()] {
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
            .position(|m| matches!(m, Some(Meld::Pung(Triplet { tile: t, .. })) if *t == tile))
            .expect("No existing Pong meld found for the specified tile");

        // The upgraded kong is always considered exposed (concealed = false)
        self.melds[pong_index] = Some(Meld::Kong(Quad::new(tile, false)));

        self.concealed.remove(tile, 1); // Remove the additional tile needed to upgrade the Pong to a Kong
    }
}

#[cfg(test)]
mod tests {}
