use crate::bits::{BitArray, MahjongBitArray};

pub struct HuSolver;

impl HuSolver {
    /// Main entry point for checking a Standard Hu (MCR).
    pub fn is_standard_hu(hand: &BitArray) -> bool {
        let possible_eyes = Self::possible_eyes_for_hu(hand);
        if possible_eyes != [0; 4] {
            return true;
        }
        false
    }

    /// Returns a BitArray of possible eyes (pairs) that lead to a valid Hu.
    /// Since there must be an eye in a Standard Hu, 0 indicates no Hu possible.
    /// Multiple bits may be set if multiple pairs lead to valid Hu.
    pub fn possible_eyes_for_hu(hand: &BitArray) -> BitArray {
        let mut possible_eyes = [0; 4];

        let honor_result = Self::check_honors(&hand[3]);
        if honor_result.is_none() {
            // Invalid Honors partition → no Hu possible
            return possible_eyes;
        }
        let eyes_from_honors = honor_result.unwrap();

        // Analyze Honors Partition (Row 3)
        // Case A: The Pair is in Honors.
        // Check Chars (Row 0), Dots (Row 1), Bams (Row 2)
        if eyes_from_honors != 0 {
            let mut temp_hand = *hand;
            if Self::solve_suits_partition(&mut temp_hand[0])
                && Self::solve_suits_partition(&mut temp_hand[1])
                && Self::solve_suits_partition(&mut temp_hand[2])
            {
                possible_eyes.bit_or([0, 0, 0, eyes_from_honors]);
            }
        }
        // Case B: The Pair maybe in Suits.
        if eyes_from_honors == 0 {
            let possible_eyes_from_suits = Self::check_suits_for_pair(hand);
            possible_eyes.bit_or(possible_eyes_from_suits);
        }

        possible_eyes
    }

    /// Checks the Honors partition (Row index 3) for validity.
    /// Valid means all tiles are in sets of 2 (at most one pair) or 3.
    /// Returns Option<BitArray>:
    /// - Some<BitArray>: if valid, with BitArray containing the pair tile if exists.
    /// - None: if invalid configuration.
    #[inline]
    fn check_honors(row: &u64) -> Option<u64> {
        const HONOR_MASK: u64 = 0x0FFFFFFF;
        let mut honor_row = row & HONOR_MASK;

        let mut eye_in_honor_row = 0;

        while honor_row != 0 {
            let shift = honor_row.trailing_zeros() as usize;
            let nibble = (honor_row >> shift) & 0xF;

            if nibble == 0b0011 {
                if eye_in_honor_row != 0 {
                    return None;
                }
                BitArray::add_to_row(&mut eye_in_honor_row, shift);
            }
            if nibble == 0b1111 {
                return None; // Invalid nibble
            }
            honor_row &= !(nibble << shift);
        }

        Some(eye_in_honor_row)
    }

    /// Iterates over potential pairs in the suit partitions.
    /// Returns a BitArray of possible pair tiles that lead to a valid decomposition.
    #[inline]
    fn check_suits_for_pair(hand: &BitArray) -> BitArray {
        let mut possible_eyes = [0; 4];
        // Iterate over Row 0, 1, 2
        for i in 0..3 {
            let row = hand[i];
            let mut row_ref = row; // Copy of the row
            while row_ref != 0 {
                let shift = row_ref.trailing_zeros() as usize;
                let nibble = (row_ref >> shift) & 0xF;
                if nibble & 0b1110 == 0 {
                    row_ref &= !(0xF << shift);
                    continue;
                }

                let mut temp_hand = *hand;

                let _ = BitArray::remove_tiles_from_row(
                    &mut temp_hand[i],
                    shift,
                    2,
                );

                if Self::solve_suits_partition(&mut temp_hand[0])
                    && Self::solve_suits_partition(&mut temp_hand[1])
                    && Self::solve_suits_partition(&mut temp_hand[2])
                {
                    possible_eyes.add_tile((64 * i + shift) as u8);
                }
                row_ref &= !(0xF << shift);
            }
        }

        possible_eyes
    }

    /// Solves a suit partition (Row index 0, 1, or 2).
    /// Returns true if any branch of the binary tree leads to empty row.
    #[inline]
    fn solve_suits_partition(row: &mut u64) -> bool {
        if *row == 0 {
            return true;
        }

        // Stack of (row_state, branch_state)
        // branch_state: 0 = try pong, 1 = try chow, 2 = done
        // depth max 5: initial node + 4 sets
        let mut stack: [(u64, u8); 5] = [(0, 0); 5];
        let mut sp: usize = 0;

        stack[sp] = (*row, 0);
        sp += 1;

        while sp > 0 {
            let (cur, branch) = stack[sp - 1];

            // If row empty → found a valid complete solution.
            if cur == 0 {
                return true;
            }

            let shift = cur.trailing_zeros() as usize;
            let nibble = (cur >> shift) & 0xF;

            if branch == 0 {
                // Try Pong branch
                stack[sp - 1].1 = 1; // Next time: try Chow

                if nibble & 0b0100 != 0 {
                    let mut next = cur;
                    let _ =
                        BitArray::remove_tiles_from_row(&mut next, shift, 3);

                    // Push Pong child state
                    stack[sp] = (next, 0);
                    sp += 1;
                }
            } else if branch == 1 {
                stack[sp - 1].1 = 2; // Next time: mark dead

                let sequences = BitArray::find_sequences_for_row(&cur);

                if ((sequences >> shift) & 0xF) != 0 {
                    let mut next = cur;
                    let _ =
                        BitArray::remove_sequences_from_row(&mut next, shift);

                    // Push Chow child state
                    stack[sp] = (next, 0);
                    sp += 1;
                }
            } else {
                // Safety: branch can only be 2 here
                sp -= 1;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    #[allow(dead_code)]
    fn to_bit_hand(tiles: Vec<Tile>) -> BitArray {
        let mut hand = [0; 4];

        for t in tiles {
            hand.add_tile(t.id());
        }
        hand
    }

    #[test]
    fn test_pure_hand_hu() {
        let mut hand: BitArray = [0; 4];
        // 111 234 678 999 55 (Pair)

        hand.add_tiles(Tile::Character1.id(), 3);
        hand.add_tile(Tile::Character2.id());
        hand.add_tile(Tile::Character3.id());
        hand.add_tile(Tile::Character4.id());

        hand.add_tile(Tile::Character6.id());
        hand.add_tile(Tile::Character7.id());
        hand.add_tile(Tile::Character8.id());

        hand.add_tiles(Tile::Character9.id(), 3);
        hand.add_tiles(Tile::Character5.id(), 2);

        assert!(HuSolver::is_standard_hu(&hand));
    }
    #[test]
    fn test_eyes_in_wind() {
        let mut row = 0u64;

        BitArray::add_tiles_to_row(&mut row, Tile::East.id() as usize % 64, 2);
        println!("Row with East pair: {:064b}", row);

        let mut expected_eyes = 0u64;
        BitArray::add_to_row(
            &mut expected_eyes,
            Tile::East.id() as usize % 64,
        );
        println!("Expected eyes: {:064b}", expected_eyes);
        assert_eq!(HuSolver::check_honors(&row).unwrap(), expected_eyes);

        BitArray::add_tiles_to_row(
            &mut row,
            Tile::South.id() as usize % 64,
            3,
        );
        println!("Row with East pair + South pong: {:064b}", row);
        assert_eq!(HuSolver::check_honors(&row).unwrap(), expected_eyes);
    }

    #[test]
    fn test_dfs_solver() {
        let mut row = 0u64;

        // 111 234
        BitArray::add_tiles_to_row(&mut row, 0, 3); // 111
        println!("After adding 111: {:064b}", row);
        assert!(HuSolver::solve_suits_partition(&mut row));

        BitArray::add_to_row(&mut row, Tile::Character4.id() as usize % 64); // 4
        BitArray::add_to_row(&mut row, Tile::Character5.id() as usize % 64); // 5
        BitArray::add_to_row(&mut row, Tile::Character6.id() as usize % 64); // 6
        println!("After adding 456: {:064b}", row);
        assert!(HuSolver::solve_suits_partition(&mut row));

        let mut row = 0u64;
        BitArray::add_tiles_to_row(&mut row, 0, 4); // 1111
        BitArray::add_to_row(&mut row, Tile::Character2.id() as usize % 64); // 2
        BitArray::add_to_row(&mut row, Tile::Character3.id() as usize % 64); // 3
        println!("After adding 1111 23: {:064b}", row);
        assert!(HuSolver::solve_suits_partition(&mut row));
    }

    #[test]
    fn test_mixed_suit_hu() {
        let mut hand: BitArray = [0; 4];

        // Pong 1-Wan
        hand.add_tiles(Tile::Character1.id(), 3);

        // Chow 1-2-3 Dot
        hand.add_tile(Tile::Dot1.id());
        hand.add_tile(Tile::Dot2.id());
        hand.add_tile(Tile::Dot3.id());

        // Pong Red Dragon
        hand.add_tiles(Tile::Red.id(), 3);

        // Chow 7-8-9 Bam
        hand.add_tile(Tile::Bamboo7.id());
        hand.add_tile(Tile::Bamboo8.id());
        hand.add_tile(Tile::Bamboo9.id());

        // Pair West Wind
        hand.add_tiles(Tile::West.id(), 2);

        assert!(HuSolver::is_standard_hu(&hand));
    }

    #[test]
    fn test_invalid_hand() {
        let mut hand: BitArray = [0; 4];
        // 11 (Pair?)
        hand.add_tiles(Tile::Character1.id(), 2);
        // 23 (Incomplete Chow)
        hand.add_tile(Tile::Character2.id());
        hand.add_tile(Tile::Character3.id());

        assert!(!HuSolver::is_standard_hu(&hand));
    }
}
