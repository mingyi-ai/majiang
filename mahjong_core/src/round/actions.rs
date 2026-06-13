use crate::structs::{Hand, Meld, Quad, Sequence, Tile, TileType, Triplet};

// Basic actions
impl Hand {
    pub(crate) fn add_tile(&mut self, tile: Tile) {
        if matches!(tile.get_type(), TileType::Flower) {
            self.flower_count += 1;
        } else {
            self.concealed.insert(tile);
        }
    }

    pub(crate) fn remove_tile(&mut self, tile: Tile, count: u8) {
        if matches!(tile.get_type(), TileType::Flower) {
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
    pub(crate) fn can_pong(&self, tile: Tile) -> bool {
        self.concealed.count(tile) >= 2
    }

    pub(crate) fn can_kong(&self, tile: Tile) -> bool {
        self.concealed.count(tile) >= 3
    }

    pub(crate) fn possible_chow_starts(&self, tile: Tile) -> Vec<Tile> {
        let _ = tile;
        unimplemented!("Chow logic not implemented yet");
    }

    /// Returns tiles where the player has an exposed Pong meld
    /// AND at least 1 more of that tile in the concealed hand,
    /// allowing an upgrade to an exposed Kong.
    pub(crate) fn possible_kong_from_pong(&self) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for meld in &self.melds {
            if let Some(Meld::Pung(Triplet { tile, is_concealed: false })) = meld {
                if self.concealed.count(*tile) >= 1 {
                    tiles.push(*tile);
                }
            }
        }
        tiles
    }

    /// Returns tiles where the concealed hand has exactly 4 copies,
    /// allowing a concealed Kong declaration.
    pub(crate) fn possible_concealed_kong(&self) -> Vec<Tile> {
        Tile::ALL
            .iter()
            .filter(|t| !t.is_flower() && self.concealed.count(**t) >= 4)
            .copied()
            .collect()
    }
}

// Actions related to modifying the hand based on player actions (Pong, Chow, Kong) and their effects on the concealed hand and melds
impl Hand {
    /// Creates a Pong meld from `tile`.
    ///
    /// - `is_concealed == true`: all 3 tiles came from the concealed hand, remove 3.
    /// - `is_concealed == false`: 2 tiles from concealed + 1 from discard, remove 2.
    pub(crate) fn pong(&mut self, tile: Tile, is_concealed: bool) {
        self.melds
            .iter_mut()
            .find(|m| m.is_none())
            .map(|slot| {
                *slot = Some(Meld::Pung(Triplet::new(tile, is_concealed)));
            })
            .expect("No empty slot available for new meld");

        let remove_count = if is_concealed { 3 } else { 2 };
        self.concealed.remove(tile, remove_count);
    }

    /// Creates a Chow meld from `start_tile` using the 2 tiles from the player's hand.
    ///
    /// - `is_concealed == true`: all 3 tiles came from the concealed hand, remove all 3.
    /// - `is_concealed == false`: 2 tiles from concealed + 1 from discard, remove `hand_tiles`.
    pub(crate) fn chow(&mut self, start_tile: Tile, hand_tiles: [Tile; 2], is_concealed: bool) {
        self.melds
            .iter_mut()
            .find(|m| m.is_none())
            .map(|slot| {
                *slot =
                    Some(Meld::Chow(Sequence::new(start_tile, is_concealed)));
            })
            .expect("No empty slot available for new meld");

        if is_concealed {
            // Remove all 3 tiles of the sequence from concealed
            // start_tile, start_tile+4, start_tile+8
            self.concealed.remove(start_tile, 1);
            let dt = start_tile as u8;
            let tile2 = Tile::from_repr(dt + 4).expect("Invalid sequence tile");
            let tile3 = Tile::from_repr(dt + 8).expect("Invalid sequence tile");
            self.concealed.remove(tile2, 1);
            self.concealed.remove(tile3, 1);
        } else {
            // Only remove the 2 tiles from the hand (the discard provides the 3rd)
            self.concealed.remove(hand_tiles[0], 1);
            self.concealed.remove(hand_tiles[1], 1);
        }
    }

    /// Creates a Kong meld from `tile`.
    ///
    /// - `is_concealed == true`: all 4 tiles came from the concealed hand, remove 4.
    /// - `is_concealed == false`: 3 tiles from concealed + 1 from discard, remove 3.
    pub(crate) fn kong(&mut self, tile: Tile, is_concealed: bool) {
        self.melds
            .iter_mut()
            .find(|m| m.is_none())
            .map(|slot| {
                *slot = Some(Meld::Kong(Quad::new(tile, is_concealed)));
            })
            .expect("No empty slot available for new meld");

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
