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
        unimplemented!("Chow logic not implemented yet");
    }

    pub(crate) fn possible_kong_from_pong(&self) -> bool {
        unimplemented!("Kong from Pong logic not implemented yet");
    }

    pub(crate) fn possible_concealed_kong(&self) -> Vec<Tile> {
        unimplemented!("Concealed Kong logic not implemented yet");
    }
}

// Actions related to modifying the hand based on player actions (Pong, Chow, Kong) and their effects on the concealed hand and melds
impl Hand {
    pub(crate) fn pong(&mut self, tile: Tile, is_concealed: bool) {
        self.melds
            .iter_mut()
            .find(|m| m.is_none())
            .map(|slot| {
                *slot = Some(Meld::Pung(Triplet::new(tile, is_concealed)));
            })
            .expect("No empty slot available for new meld");

        self.concealed.remove(tile, 3); // Remove the three tiles used for the Pong from the concealed hand
    }

    pub(crate) fn chow(&mut self, start_tile: Tile, is_concealed: bool) {
        self.melds
            .iter_mut()
            .find(|m| m.is_none())
            .map(|slot| {
                *slot =
                    Some(Meld::Chow(Sequence::new(start_tile, is_concealed)));
            })
            .expect("No empty slot available for new meld");

        // self.concealed.remove(start_tile, 1);
        unimplemented!("Remove sequence not implemented yet");
    }

    pub(crate) fn kong(&mut self, tile: Tile, is_concealed: bool) {
        self.melds
            .iter_mut()
            .find(|m| m.is_none())
            .map(|slot| {
                *slot = Some(Meld::Kong(Quad::new(tile, is_concealed)));
            })
            .expect("No empty slot available for new meld");

        self.concealed.remove(tile, 4); // Remove the four tiles used for the Kong from the concealed hand
    }

    pub(crate) fn kong_from_pong(&mut self, tile: Tile) {
        let pong_index = self
            .melds
            .iter()
            .position(|m| matches!(m, Some(Meld::Pung(Triplet { tile: t, .. })) if *t == tile))
            .expect("No existing Pong meld found for the specified tile");

        self.melds[pong_index] = Some(Meld::Kong(Quad::new(tile, true)));

        self.concealed.remove(tile, 1); // Remove the additional tile needed to upgrade the Pong to a Kong
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
