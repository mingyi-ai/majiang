use crate::tile::{Tile, TileType, Wall};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandError {
    DrawError(&'static str),
    DiscardError(&'static str),
    ActionError(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub enum Meld {
    Chow(Tile, Tile, Tile),
    Pong(Tile),
    Kong(Tile),
    ConcealedKong(Tile),
}

#[derive(Debug, Clone, Default)]
pub struct Hand {
    pub tiles: Vec<Tile>,
    pub melds: Vec<Meld>,
    pub flowers: Vec<Tile>,
}

impl Hand {
    const INITIAL_HAND_SIZE: usize = 13;

    pub fn draw(&mut self, wall: &mut Wall) -> Result<Vec<Tile>, HandError> {
        let mut drawn_tiles: Vec<Tile> = Vec::new();

        let tile: Tile =
            wall.draw().ok_or(HandError::DrawError("Wall is empty"))?;
        drawn_tiles.push(tile);

        if !tile.is_flower() {
            self.tiles.push(tile);
            return Ok(drawn_tiles);
        }

        self.flowers.push(tile);

        loop {
            let repl: Tile =
                wall.draw().ok_or(HandError::DrawError("Wall is empty"))?;
            drawn_tiles.push(repl);
            if repl.is_flower() {
                self.flowers.push(repl);
                continue;
            } else {
                self.tiles.push(repl);
                break;
            }
        }

        Ok(drawn_tiles)
    }

    pub fn draw_initial_hand(
        &mut self,
        wall: &mut Wall,
    ) -> Result<Vec<Tile>, HandError> {
        let mut drawn_tiles: Vec<Tile> = Vec::new();
        while self.tiles.len() < Self::INITIAL_HAND_SIZE {
            let drawn = self.draw(wall)?;
            drawn_tiles.extend(drawn.clone());
        }

        Ok(drawn_tiles)
    }

    pub fn discard(&mut self, target: Tile) -> Result<Tile, HandError> {
        self.tiles
            .iter()
            .position(|&t| t == target)
            .map(|i| self.tiles.swap_remove(i))
            .ok_or(HandError::DiscardError("Tile not found in hand"))
    }

    pub fn possible_chows(&self, target: Tile) -> Vec<[Tile; 2]> {
        match target.get_type() {
            TileType::Dot | TileType::Bamboo | TileType::Character => {}
            _ => return Vec::new(),
        }

        let t_id = target.id();

        let mut candidates: Vec<Tile> = self
            .tiles
            .iter()
            .copied()
            .filter(|t| t.get_type() == target.get_type())
            .filter(|t| {
                let id = t.id();
                id >= t_id.saturating_sub(2) && id <= t_id + 2
            })
            .collect();

        candidates.sort();
        candidates.dedup();

        if candidates.len() < 2 {
            return Vec::new();
        }

        let mut result = Vec::new();

        for w in candidates.windows(2) {
            let a = w[0].id();
            let b = w[1].id();

            let mut trio = [a, b, t_id];
            trio.sort_unstable();

            if trio[0] + 1 == trio[1] && trio[1] + 1 == trio[2] {
                result.push([w[0], w[1]]);
            }
        }

        result
    }

    pub fn can_pong(&self, target: Tile) -> bool {
        self.tiles.iter().filter(|&&t| t == target).count() >= 2
    }

    pub fn can_kong(&self, target: Tile) -> bool {
        self.tiles.iter().filter(|&&t| t == target).count() >= 3
    }
    pub fn possible_concealed_kong(&self) -> Vec<Tile> {
        let mut counts = [0; Tile::COUNT];

        for &tile in &self.tiles {
            counts[tile.id() as usize] += 1;
        }

        counts
            .iter()
            .enumerate()
            .filter_map(|(i, &count)| {
                if count >= 4 {
                    Tile::from_usize(i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn possible_kong_from_pong(&self) -> Vec<Tile> {
        self.melds
            .iter()
            .filter_map(|meld| match meld {
                Meld::Pong(tile) => Some(*tile),
                _ => None,
            })
            .filter(|tile| self.tiles.contains(tile))
            .collect()
    }

    pub fn chow(&mut self, target: Tile, tiles: [Tile; 2]) {
        let mut to_remove = tiles.to_vec();

        self.tiles.retain(|&t| {
            if let Some(pos) = to_remove.iter().position(|&x| x == t) {
                to_remove.remove(pos);
                false
            } else {
                true
            }
        });

        self.melds.push(Meld::Chow(target, tiles[0], tiles[1]));
    }

    pub fn pong(&mut self, target: Tile) {
        let mut to_remove = 2;

        // when pong, the target tile is given from other player
        // so only need to remove the two tiles from hand
        self.tiles.retain(|&t| {
            if t == target && to_remove > 0 {
                to_remove -= 1;
                false
            } else {
                true
            }
        });

        self.melds.push(Meld::Pong(target));
    }

    pub fn kong(&mut self, target: Tile) {
        let mut to_remove = 3;

        // when kong, the target tile is given from other player
        // so only need to remove the three tiles from hand
        self.tiles.retain(|&t| {
            if t == target && to_remove > 0 {
                to_remove -= 1;
                false
            } else {
                true
            }
        });

        self.melds.push(Meld::Kong(target));
    }

    pub fn added_kong(&mut self, target: Tile) {
        // when added kong, one tile is from hand
        self.tiles.retain(|&t| t != target);

        // find the existing pong meld and upgrade it to kong
        if let Some(pos) = self
            .melds
            .iter()
            .position(|meld| matches!(meld, Meld::Pong(t) if *t == target))
        {
            self.melds[pos] = Meld::Kong(target);
        }
    }

    pub fn concealed_kong(&mut self, target: Tile) {
        let mut to_remove = 4;

        // when concealed kong, all four tiles are from hand
        self.tiles.retain(|&t| {
            if t == target && to_remove > 0 {
                to_remove -= 1;
                false
            } else {
                true
            }
        });

        self.melds.push(Meld::ConcealedKong(target));
    }

    pub fn can_hu_self(hand: &Hand) -> Result<bool, HandError> {
        // TODO: separate of concerns, this function is too long
        // Naive Hu check: exhaust by the following order: pair first, then pongs, then chows
        // count tiles in hand
        let mut counts: [u8; 42] = [0; Tile::COUNT];
        for &tile in &hand.tiles {
            counts[tile.id() as usize] += 1;
        }

        for count in counts.iter() {
            if *count > 4 {
                return Err(HandError::ActionError(
                    "Invalid hand: more than 4 of the same tile",
                ));
            }
        }

        // sanity check
        if hand.tiles.len() % 3 != 2 {
            return Err(HandError::ActionError("Invalid hand size for Hu"));
        }

        // try each tile as the pair
        for i in 0..Tile::COUNT {
            // skip if less than 2 tiles
            if counts[i] < 2 {
                continue;
            }

            // reset sets count
            let mut sets_count = hand.melds.len();

            // make a trail copy
            let mut trail = counts;
            // remove the pair
            trail[i] -= 2;

            // check if the rest can form pongs
            for count in trail.iter_mut() {
                if *count >= 3 {
                    // Check for pongs
                    *count -= 3;
                    sets_count += 1;
                }
            }

            // skip if there are honor or flower tiles remaining
            if trail
                .iter()
                .enumerate()
                .filter(|(_, c)| **c >= 1)
                .any(|(id, _)| id >= 27)
            {
                continue;
            }

            // Check for chows
            for t in Tile::iter_all() {
                let t_id = t.id();

                // only suited tiles between 1 and 7 can form chows
                if t_id % 9 >= 7 || t_id >= 27 {
                    continue;
                }

                while trail[t_id as usize] > 0
                    && trail[(t_id + 1) as usize] > 0
                    && trail[(t_id + 2) as usize] > 0
                {
                    trail[t_id as usize] -= 1;
                    trail[(t_id + 1) as usize] -= 1;
                    trail[(t_id + 2) as usize] -= 1;
                    sets_count += 1;
                }
            }

            if trail.iter().all(|&c| c == 0) && sets_count == 4 {
                return Ok(true);
            }

            // for the same pair, try exhausting chows first then pongs
            // reset sets count
            sets_count = hand.melds.len();

            // make a trail copy
            let mut trail = counts;
            // remove the pair
            trail[i] -= 2;

            // Check for chows
            for t in Tile::iter_all() {
                let t_id = t.id();

                // only suited tiles between 1 and 7 can form chows
                if t_id % 9 >= 7 || t_id >= 27 {
                    continue;
                }

                while trail[t_id as usize] > 0
                    && trail[(t_id + 1) as usize] > 0
                    && trail[(t_id + 2) as usize] > 0
                {
                    trail[t_id as usize] -= 1;
                    trail[(t_id + 1) as usize] -= 1;
                    trail[(t_id + 2) as usize] -= 1;
                    sets_count += 1;
                }
            }

            // check if the rest can form pongs
            for count in trail.iter_mut() {
                if *count >= 3 {
                    // Check for pongs
                    *count -= 3;
                    sets_count += 1;
                }
            }

            if trail.iter().all(|&c| c == 0) && sets_count == 4 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn can_hu(&self, target: Tile) -> Result<bool, HandError> {
        let mut temp_hand = self.clone();
        temp_hand.tiles.push(target);
        Self::can_hu_self(&temp_hand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile::*;

    macro_rules! hu_case {
        ($name:ident : ($tiles:expr, $melds:expr, $flowers:expr) => $expected:expr) => {
            #[test]
            fn $name() {
                let hand = Hand {
                    tiles: $tiles,
                    melds: $melds,
                    flowers: $flowers,
                };
                assert_eq!(Hand::can_hu_self(&hand), $expected);
            }
        };
    }
    hu_case!(test_hu_all_chow: (
        vec![
            Dot1,Dot2,Dot3, 
            Dot4,Dot5,Dot6, 
            Dot7,Dot8,Dot9, 
            Bamboo1,Bamboo2,Bamboo3,
            Red,Red // pair
        ],
        vec![],
        vec![]
    ) => Ok(true));
    hu_case!(test_hu_one_chow_in_melds: (
        vec![
            Dot1,Dot2,Dot3, 
            Dot4,Dot5,Dot6, 
            Bamboo1,Bamboo2,Bamboo3,
            Red,Red // pair
        ],
        vec![
            Meld::Chow(Dot7,Dot8,Dot9)
        ],
        vec![]
    ) => Ok(true));
    hu_case!(test_hu_all_chow_in_melds: (
        vec![
            Red,Red // pair
        ],
        vec![
            Meld::Chow(Dot1,Dot2,Dot3), 
            Meld::Chow(Dot4,Dot5,Dot6), 
            Meld::Chow(Dot7,Dot8,Dot9), 
            Meld::Chow(Bamboo1,Bamboo2,Bamboo3)
        ],
        vec![]
    ) => Ok(true));
    hu_case!(test_hu_all_pong: (
        vec![
            Dot1,Dot1,Dot1, // pong
            Bamboo2,Bamboo2,Bamboo2, // pong
            Character3,Character3,Character3, // pong
            East,East,East, // pong
            Red,Red // pair
        ],
        vec![],
        vec![]
    ) => Ok(true));
    hu_case!(test_hu_one_pong_in_melds: (
        vec![
            Dot1,Dot1,Dot1, // pong
            Bamboo2,Bamboo2,Bamboo2, // pong
            East,East,East, // pong
            Red,Red // pair
        ],
        vec![
            Meld::Pong(Character3),
        ],
        vec![]
    ) => Ok(true));
    hu_case!(test_hu_all_pong_in_melds: (
        vec![
            East,East // pair
        ],
        vec![
            Meld::Pong(Dot1), 
            Meld::Pong(Bamboo2), 
            Meld::Pong(Character3), 
            Meld::Pong(East)
        ],
        vec![]
    ) => Ok(true));
    hu_case!(test_hu_mixed: (
        vec![
            Dot1,Dot2,Dot3, // chow
            Bamboo2,Bamboo2,Bamboo2, // pong
            Character3,Character3,Character3, // pong
            East,East,East, // pong
            Red,Red // pair
        ],
        vec![],
        vec![]
    ) => Ok(true));
    hu_case!(test_hu_shifted_mixed:
            (
            vec![
                Dot1, Dot2, Dot3, // chow
                Dot2, Dot3, Dot4, // chow
                Dot3, Dot4, Dot5, // chow
                Red, Red // pair
            ],
            vec![
                Meld::Chow(Bamboo4,Bamboo5,Bamboo6)
            ],
            vec![]
        ) => Ok(true)
    );
    hu_case!(test_hu_invalid_size_1: (
        vec![
            Dot1,Dot2,Dot3, // chow
            Bamboo2,Bamboo2,Bamboo2, // pong
            Character3,Character3,Character3, // pong
            East,East,East, // pong
        ],
        vec![],
        vec![]
    ) => Err(HandError::ActionError("Invalid hand size for Hu")));
    hu_case!(test_hu_invalid_size_2: (
        vec![
            Dot1,Dot2,Dot3, // chow
            Bamboo2,Bamboo2,Bamboo2, // pong
            Character3,Character3,Character3, // pong
            East,East,East, // pong
            Red // single tile
        ],
        vec![],
        vec![]
    ) => Err(HandError::ActionError("Invalid hand size for Hu")));
    hu_case!(test_not_hu: (
        vec![
            Dot1,Dot2,Dot3, // chow
            Bamboo2,Bamboo2,Bamboo2, // pong
            Character3,Character3,Character3, // pong
            East,East,East, // pong
            Red,Green // waiting for pair
        ],
        vec![],
        vec![]
    ) => Ok(false));
    hu_case!(test_invalid_more_than_four: (
        vec![
            Dot1,Dot1,Dot1,
            Bamboo2,Bamboo2,Bamboo2,
            Character3,Character3,Character3,
            East,East,East,
            Dot1,Dot1, // five of a kind, invalid
        ],
        vec![],
        vec![]
    ) => Err(HandError::ActionError("Invalid hand: more than 4 of the same tile")));
    #[test]
    fn test_possible_chows() {
        let hand = Hand {
            tiles: vec![
                Dot2, Dot3, Dot4, Dot6, Dot7, Dot8, Bamboo2, Bamboo3, Bamboo5,
                Bamboo6,
            ],
            melds: vec![],
            flowers: vec![],
        };
        let chows_for_dot5 = hand.possible_chows(Dot5);
        println!("{:?}", chows_for_dot5);
        assert_eq!(chows_for_dot5.len(), 3);
        assert!(chows_for_dot5.contains(&[Dot3, Dot4]));
        assert!(chows_for_dot5.contains(&[Dot6, Dot7]));
        assert!(chows_for_dot5.contains(&[Dot4, Dot6]));

        let chows_for_dot1 = hand.possible_chows(Dot1);
        assert_eq!(chows_for_dot1.len(), 1);
        assert!(chows_for_dot1.contains(&[Dot2, Dot3]));

        let chows_for_honor = hand.possible_chows(East);
        assert_eq!(chows_for_honor.len(), 0);
    }
}
