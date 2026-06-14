use crate::structs::BitTileCounts;

/// Standard MCR Hu solver using DFS over suit partitions.
impl BitTileCounts {
    pub fn is_hu(&self) -> bool {
        let eyes = self.possible_eyes_for_hu();
        eyes.iter().any(|&row| row != 0)
    }

    /// Returns possible eye tiles for each row.
    /// A non-zero entry means a pair at that position leads to a valid
    /// decomposition of the remaining tiles.
    fn possible_eyes_for_hu(&self) -> [u64; 4] {
        let mut possible_eyes = [0u64; 4];

        // ── Check honors (row 3) ──
        let honor_eye = match Self::check_honors(&self.rows[3]) {
            None => return possible_eyes,
            Some(e) => e,
        };

        // Case A: Pair is in honors → solve suits without a pair
        if honor_eye != 0 {
            let mut temp = *self;
            if Self::solve_suit(&mut temp.rows[0])
                && Self::solve_suit(&mut temp.rows[1])
                && Self::solve_suit(&mut temp.rows[2])
            {
                possible_eyes[3] = honor_eye;
            }
        }

        // Case B: No pair in honors → try each possible suit pair
        if honor_eye == 0 {
            let suit_eyes = Self::check_suits_for_pair(self);
            possible_eyes[0] = suit_eyes[0];
            possible_eyes[1] = suit_eyes[1];
            possible_eyes[2] = suit_eyes[2];
        }

        possible_eyes
    }

    /// Check honors row: must be all pung (3) or exactly one pair (2).
    /// Returns the eye bitmask, or None if invalid.
    fn check_honors(row: &u64) -> Option<u64> {
        const HONOR_MASK: u64 = 0x0FFFFFFF;
        let mut r = row & HONOR_MASK;
        let mut eye = 0u64;

        while r != 0 {
            let shift = r.trailing_zeros() as usize;
            let nibble = (r >> shift) & 0xF;

            if nibble == 0b0011 {
                // pair
                if eye != 0 {
                    return None; // more than one pair
                }
                Self::add_to_row(&mut eye, shift);
            } else if nibble == 0b1111 {
                return None; // kong not allowed in hu hand
            }
            // 0b0001 (single) or 0b0111 (pung) → valid, just clear
            r &= !(nibble << shift);
        }

        Some(eye)
    }

    /// Try each possible pair in the suit rows, return eye bitmasks.
    fn check_suits_for_pair(&self) -> [u64; 4] {
        let mut eyes = [0u64; 4];
        for row_idx in 0..3 {
            let mut row = self.rows[row_idx];
            while row != 0 {
                let shift = row.trailing_zeros() as usize;
                let nibble = (row >> shift) & 0xF;

                // Skip tiles that can't form a pair (count < 2)
                if nibble & 0b0010 == 0 {
                    row &= !(0xF << shift);
                    continue;
                }

                // Try removing this pair and solving the rest
                let mut temp = *self;
                Self::remove_nibble(
                    &mut temp.rows[row_idx],
                    shift,
                    2,
                );
                if Self::solve_suit(&mut temp.rows[0])
                    && Self::solve_suit(&mut temp.rows[1])
                    && Self::solve_suit(&mut temp.rows[2])
                {
                    Self::add_to_row(&mut eyes[row_idx], shift);
                }

                row &= !(0xF << shift);
            }
        }
        eyes
    }

    /// DFS over a single suit row: try pong then chow at the first
    /// non-zero nibble, backtracking via explicit stack.
    fn solve_suit(row: &mut u64) -> bool {
        if *row == 0 {
            return true;
        }

        // Stack: (row_state, branch)
        // branch 0 = try pong, 1 = try chow, 2 = done (pop)
        let mut stack: [(u64, u8); 5] = [(0, 0); 5];
        let mut sp: usize = 0;
        stack[sp] = (*row, 0);
        sp += 1;

        while sp > 0 {
            let (cur, branch) = stack[sp - 1];

            if cur == 0 {
                return true;
            }

            let shift = cur.trailing_zeros() as usize;
            let nibble = (cur >> shift) & 0xF;

            if branch == 0 {
                // Try pong
                stack[sp - 1].1 = 1;

                if nibble & 0b0100 != 0 {
                    // has at least 3 copies
                    let mut next = cur;
                    Self::remove_nibble(&mut next, shift, 3);
                    stack[sp] = (next, 0);
                    sp += 1;
                }
            } else if branch == 1 {
                // Try chow
                stack[sp - 1].1 = 2;

                let seqs = Self::find_sequences_for_row(cur);
                if (seqs >> shift) & 0xF != 0 {
                    let mut next = cur;
                    Self::remove_sequence_from_row(&mut next, shift);
                    stack[sp] = (next, 0);
                    sp += 1;
                }
            } else {
                // branch == 2 → dead end, pop
                sp -= 1;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::Tile;

    fn hand_from(tiles: &[(Tile, u8)]) -> BitTileCounts {
        let mut h = BitTileCounts::default();
        for &(t, n) in tiles {
            for _ in 0..n {
                h.insert(t);
            }
        }
        h
    }

    #[test]
    fn test_pure_hand_hu() {
        let hand = hand_from(&[
            (Tile::Character1, 3), // pong
            (Tile::Character2, 1),
            (Tile::Character3, 1),
            (Tile::Character4, 1), // chow 234
            (Tile::Character6, 1),
            (Tile::Character7, 1),
            (Tile::Character8, 1), // chow 678
            (Tile::Character9, 3), // pong
            (Tile::Character5, 2), // pair
        ]);
        assert!(hand.is_hu());
    }

    #[test]
    fn test_mixed_suit_hu() {
        let hand = hand_from(&[
            (Tile::Character1, 1),
            (Tile::Character2, 1),
            (Tile::Character3, 1), // chow 123 Wan
            (Tile::Dot1, 1),
            (Tile::Dot2, 1),
            (Tile::Dot3, 1), // chow 123 Tong
            (Tile::Red, 3),  // pong Red
            (Tile::Bamboo7, 1),
            (Tile::Bamboo8, 1),
            (Tile::Bamboo9, 1), // chow 789 Tiao
            (Tile::West, 2),    // pair West
        ]);
        assert!(hand.is_hu());
    }

    #[test]
    fn test_invalid_hand() {
        let hand = hand_from(&[
            (Tile::Character1, 2), // pair only
            (Tile::Character2, 1),
            (Tile::Character3, 1), // incomplete
        ]);
        assert!(!hand.is_hu());
    }

    #[test]
    fn test_eyes_in_honors() {
        let mut hand = BitTileCounts::default();
        hand.insert(Tile::East);
        hand.insert(Tile::East); // pair East
        assert_eq!(
            BitTileCounts::check_honors(&hand.rows[3]),
            Some(0b0001u64 << 0) // shift 0 = East
        );

        hand.insert(Tile::South);
        hand.insert(Tile::South);
        hand.insert(Tile::South); // pong South
        assert_eq!(
            BitTileCounts::check_honors(&hand.rows[3]),
            Some(0b0001u64 << 0) // East pair still the only eye
        );
    }

    #[test]
    fn test_dfs_solver() {
        // 111 (pong)
        let mut row = 0u64;
        for _ in 0..3 {
            BitTileCounts::add_to_row(&mut row, 0);
        }
        assert!(BitTileCounts::solve_suit(&mut row));

        // 111 234 (pong + chow)
        BitTileCounts::add_to_row(&mut row, 12); // Char4
        BitTileCounts::add_to_row(&mut row, 16); // Char5
        BitTileCounts::add_to_row(&mut row, 20); // Char6
        assert!(BitTileCounts::solve_suit(&mut row));

        // 1111 23 (kong_split: 111 + 123 with one from the kong)
        let mut row = 0u64;
        for _ in 0..4 {
            BitTileCounts::add_to_row(&mut row, 0); // 4x Char1
        }
        BitTileCounts::add_to_row(&mut row, 4); // Char2
        BitTileCounts::add_to_row(&mut row, 8); // Char3
        assert!(BitTileCounts::solve_suit(&mut row));
    }
}
