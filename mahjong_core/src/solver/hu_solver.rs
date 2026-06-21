use crate::structs::{BitTileCounts, Meld, Sequence, Tile, Triplet};

use super::decomposition::ConcealedDecomp;

// ═══════════════════════════════════════════════════════════════════════
// SuitDecomp — stack-allocated meld sequence
// ═══════════════════════════════════════════════════════════════════════

/// One valid decomposition of a single suit row into up to 4 melds.
///
/// Fixed-size array avoids heap allocation.  `Clone` / `Copy` is a cheap
/// fixed-size memcpy.
#[derive(Clone, Copy)]
pub(crate) struct SuitDecomp {
    melds: [Option<Meld>; 4],
    len: u8,
}

impl SuitDecomp {
    fn new() -> Self {
        Self {
            melds: [None; 4],
            len: 0,
        }
    }

    fn push(&mut self, m: Meld) {
        self.melds[self.len as usize] = Some(m);
        self.len += 1;
    }

    fn pop(&mut self) {
        self.len -= 1;
        self.melds[self.len as usize] = None;
    }

    fn len(&self) -> usize {
        self.len as usize
    }

    /// Copy melds into a `Vec<Meld>` (for the final `ConcealedDecomp`).
    fn extend_vec(&self, target: &mut Vec<Meld>) {
        for i in 0..self.len as usize {
            target.push(self.melds[i].unwrap());
        }
    }

    /// Copy melds into another `SuitDecomp` (for combining suits).
    fn copy_into(&self, target: &mut SuitDecomp) {
        for i in 0..self.len as usize {
            target.push(self.melds[i].unwrap());
        }
    }
}

/// All valid decompositions of a single suit row.
type SuitDecomps = Vec<SuitDecomp>;

// ═══════════════════════════════════════════════════════════════════════
// Suit solver — recursive DFS
// ═══════════════════════════════════════════════════════════════════════

/// Enumerate all ways to partition a suit row into pongs and chows.
///
/// Returns an empty vec if the row cannot be fully decomposed (e.g., a
/// lone single tile with no valid pong or chow).
fn decompose_suit(row: u64, row_idx: usize) -> SuitDecomps {
    let mut results = SuitDecomps::new();
    let mut path = SuitDecomp::new();
    dfs_decompose(row, row_idx, &mut path, &mut results);
    results
}

fn dfs_decompose(
    row: u64,
    row_idx: usize,
    path: &mut SuitDecomp,
    out: &mut SuitDecomps,
) {
    if row == 0 {
        out.push(*path);
        return;
    }

    let shift = (row.trailing_zeros() as usize) & !3usize;
    let nibble = (row >> shift) & 0xF;

    // Try pong (3 copies at this nibble)
    if nibble.count_ones() >= 3 {
        let tile = BitTileCounts::position_to_tile(row_idx, shift);
        path.push(Meld::Pung(Triplet::new(tile, true)));
        let mut next = row;
        BitTileCounts::remove_nibble(&mut next, shift, 3);
        dfs_decompose(next, row_idx, path, out);
        path.pop();
    }

    // Try chow (sequence starting at this nibble)
    if (BitTileCounts::find_sequences_for_row(row) >> shift) & 1 != 0 {
        let start = BitTileCounts::position_to_tile(row_idx, shift);
        path.push(Meld::Chow(Sequence::new(start, true)));
        let mut next = row;
        BitTileCounts::remove_sequence_from_row(&mut next, shift);
        dfs_decompose(next, row_idx, path, out);
        path.pop();
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Honor parser
// ═══════════════════════════════════════════════════════════════════════

struct HonorResult {
    pungs: Vec<Meld>,
    pair: Option<Tile>,
}

/// Parse the honors row.  Returns `None` if the row has a structure that
/// can never appear in a valid winning hand (singles, concealed kongs,
/// multiple pairs).
fn parse_honors(row: u64) -> Option<HonorResult> {
    const HONOR_MASK: u64 = 0x0FFFFFFF;
    let mut r = row & HONOR_MASK;
    let mut pungs = Vec::new();
    let mut pair = None;

    while r != 0 {
        let shift = (r.trailing_zeros() as usize) & !3usize;
        let nibble = (r >> shift) & 0xF;
        let tile = BitTileCounts::position_to_tile(3, shift);
        r &= !(0xF << shift);

        match nibble.count_ones() {
            1 => return None,
            2 if pair.is_some() => return None,
            2 => pair = Some(tile),
            3 => pungs.push(Meld::Pung(Triplet::new(tile, true))),
            4 => return None,
            _ => unreachable!(),
        }
    }

    Some(HonorResult { pungs, pair })
}

/// Find all nibble positions where tile count >= 2.
fn pair_positions_in_row(row: u64) -> Vec<usize> {
    let mut r = row;
    let mut positions = Vec::new();
    while r != 0 {
        let shift = (r.trailing_zeros() as usize) & !3usize;
        if ((r >> shift) & 0xF).count_ones() >= 2 {
            positions.push(shift);
        }
        r &= !(0xF << shift);
    }
    positions
}

// ═══════════════════════════════════════════════════════════════════════
// Suit combination  —  Cartesian product across the three suits
// ═══════════════════════════════════════════════════════════════════════

/// Cartesian product of one decomposition from each suit.
///
/// Each `SuitDecomp` from a given row has a fixed length determined by
/// the tile count of that row (tiles / 3).  Since `decompose_suit` only
/// returns decompositions that fully consume the row, every combination
/// is valid — no meld-count filter needed.
fn combine_triple(
    s0: &SuitDecomps,
    s1: &SuitDecomps,
    s2: &SuitDecomps,
) -> SuitDecomps {
    let mut results = SuitDecomps::new();
    for d0 in s0 {
        for d1 in s1 {
            for d2 in s2 {
                let mut combined = SuitDecomp::new();
                d0.copy_into(&mut combined);
                d1.copy_into(&mut combined);
                d2.copy_into(&mut combined);
                results.push(combined);
            }
        }
    }
    results
}

/// All three suits use their pre-computed decompositions.
fn combine_suits(suit_cache: &[SuitDecomps; 3]) -> SuitDecomps {
    combine_triple(&suit_cache[0], &suit_cache[1], &suit_cache[2])
}

/// One suit's decompositions replaced (pair was removed from that suit).
fn combine_suits_with(
    suit_cache: &[SuitDecomps; 3],
    replace_idx: usize,
    replacement: &SuitDecomps,
) -> SuitDecomps {
    let s0 = if replace_idx == 0 {
        replacement
    } else {
        &suit_cache[0]
    };
    let s1 = if replace_idx == 1 {
        replacement
    } else {
        &suit_cache[1]
    };
    let s2 = if replace_idx == 2 {
        replacement
    } else {
        &suit_cache[2]
    };
    combine_triple(s0, s1, s2)
}

// ═══════════════════════════════════════════════════════════════════════
// Top-level orchestration
// ═══════════════════════════════════════════════════════════════════════

pub(crate) struct HuSolver;

impl HuSolver {
    pub(crate) fn is_hu(counts: &BitTileCounts) -> bool {
        !Self::find_all_decompositions(counts).is_empty()
    }

    /// Find all valid decompositions into sets + pair.
    ///
    /// The tile count determines the structure: `total = n_sets * 3 + 2`.
    /// A hand with 0 or 1 tiles, or a total that doesn't satisfy this
    /// equation, cannot be a winning hand and returns empty.
    pub(crate) fn find_all_decompositions(
        counts: &BitTileCounts,
    ) -> Vec<ConcealedDecomp> {
        let Some(honors) = parse_honors(counts.rows[3]) else {
            return vec![];
        };

        // ── Pre-compute suit decompositions (no pair removed) ──
        let suit_cache = decompose_all_suits(&counts.rows[..3]);

        let mut out = Vec::new();

        // ── Case A: pair is in honours ──
        if let Some(pair_tile) = honors.pair {
            if counts.rows[0] != 0 && suit_cache[0].is_empty() {
                return vec![];
            }
            if counts.rows[1] != 0 && suit_cache[1].is_empty() {
                return vec![];
            }
            if counts.rows[2] != 0 && suit_cache[2].is_empty() {
                return vec![];
            }
            let combos = combine_suits(&suit_cache);
            emit_solutions(&honors.pungs, pair_tile, combos, &mut out);
            return out;
        }

        // ── Case B: pair is in a suit ──
        for suit in 0..3 {
            try_each_pair_in_suit(
                suit,
                counts,
                &honors.pungs,
                &suit_cache,
                &mut out,
            );
        }

        out
    }
}

// ── Helpers ──

fn decompose_all_suits(rows: &[u64]) -> [SuitDecomps; 3] {
    [
        decompose_suit(rows[0], 0),
        decompose_suit(rows[1], 1),
        decompose_suit(rows[2], 2),
    ]
}

/// Convert combined suit decompositions into `ConcealedDecomp` values.
fn emit_solutions(
    honor_pungs: &[Meld],
    pair_tile: Tile,
    combos: SuitDecomps,
    out: &mut Vec<ConcealedDecomp>,
) {
    for combo in &combos {
        let total = honor_pungs.len() + combo.len();
        let mut sets: Vec<Meld> = Vec::with_capacity(total);
        sets.extend_from_slice(honor_pungs);
        combo.extend_vec(&mut sets);
        out.push(ConcealedDecomp { pair_tile, sets });
    }
}

/// Try every valid pair position in one suit, emit solutions for each.
fn try_each_pair_in_suit(
    suit: usize,
    counts: &BitTileCounts,
    honor_pungs: &[Meld],
    suit_cache: &[SuitDecomps; 3],
    out: &mut Vec<ConcealedDecomp>,
) {
    let row = counts.rows[suit];
    for shift in pair_positions_in_row(row) {
        let pair_tile = BitTileCounts::position_to_tile(suit, shift);

        let row_minus_pair = {
            let mut r = row;
            BitTileCounts::remove_nibble(&mut r, shift, 2);
            r
        };
        let pair_decomps = decompose_suit(row_minus_pair, suit);

        for i in 0..3 {
            if i == suit {
                if row_minus_pair != 0 && pair_decomps.is_empty() {
                    return;
                }
                continue;
            }
            if counts.rows[i] != 0 && suit_cache[i].is_empty() {
                return;
            }
        }
        let combos = combine_suits_with(suit_cache, suit, &pair_decomps);
        emit_solutions(honor_pungs, pair_tile, combos, out);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

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

    fn single_suit_hand(tiles: &[Tile]) -> BitTileCounts {
        let mut h = BitTileCounts::default();
        for &t in tiles {
            h.insert(t);
        }
        h
    }

    // ── decompose_suit ──

    #[test]
    fn test_decompose_suit_pong() {
        let mut row = 0u64;
        for _ in 0..3 {
            BitTileCounts::add_to_row(&mut row, 0);
        }
        let results = decompose_suit(row, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].len(), 1);
        assert!(matches!(results[0].melds[0], Some(Meld::Pung(_))));
    }

    #[test]
    fn test_decompose_suit_chow() {
        let mut row = 0u64;
        BitTileCounts::add_to_row(&mut row, 0);
        BitTileCounts::add_to_row(&mut row, 4);
        BitTileCounts::add_to_row(&mut row, 8);
        let results = decompose_suit(row, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].len(), 1);
        assert!(matches!(results[0].melds[0], Some(Meld::Chow(_))));
    }

    #[test]
    fn test_decompose_suit_kong_split() {
        let mut row = 0u64;
        for _ in 0..4 {
            BitTileCounts::add_to_row(&mut row, 0);
        }
        BitTileCounts::add_to_row(&mut row, 4);
        BitTileCounts::add_to_row(&mut row, 8);
        let results = decompose_suit(row, 0);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.len() == 2));
    }

    #[test]
    fn test_decompose_suit_two_pungs() {
        let mut row = 0u64;
        for _ in 0..3 {
            BitTileCounts::add_to_row(&mut row, 0);
            BitTileCounts::add_to_row(&mut row, 4);
        }
        let results = decompose_suit(row, 0);
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.len() == 2));
    }

    // ── is_hu ──

    #[test]
    fn test_pure_hand_hu() {
        let hand = hand_from(&[
            (Tile::Character1, 3),
            (Tile::Character2, 1),
            (Tile::Character3, 1),
            (Tile::Character4, 1),
            (Tile::Character6, 1),
            (Tile::Character7, 1),
            (Tile::Character8, 1),
            (Tile::Character9, 3),
            (Tile::Character5, 2),
        ]);
        assert!(HuSolver::is_hu(&hand));
    }

    #[test]
    fn test_mixed_suit_hu() {
        let hand = hand_from(&[
            (Tile::Character1, 1),
            (Tile::Character2, 1),
            (Tile::Character3, 1),
            (Tile::Dot1, 1),
            (Tile::Dot2, 1),
            (Tile::Dot3, 1),
            (Tile::Red, 3),
            (Tile::Bamboo7, 1),
            (Tile::Bamboo8, 1),
            (Tile::Bamboo9, 1),
            (Tile::West, 2),
        ]);
        assert!(HuSolver::is_hu(&hand));
    }

    #[test]
    fn test_invalid_hand() {
        let hand = hand_from(&[
            (Tile::Character1, 2),
            (Tile::Character2, 1),
            (Tile::Character3, 1),
        ]);
        assert!(!HuSolver::is_hu(&hand));
    }

    // ── parse_honors ──

    #[test]
    fn test_parse_honors_all_pungs() {
        let mut h = BitTileCounts::default();
        h.insert(Tile::East);
        h.insert(Tile::East);
        h.insert(Tile::East);
        h.insert(Tile::South);
        h.insert(Tile::South);
        h.insert(Tile::South);
        let res = parse_honors(h.rows[3]).unwrap();
        assert_eq!(res.pungs.len(), 2);
        assert!(res.pair.is_none());
    }

    #[test]
    fn test_parse_honors_with_pair() {
        let mut h = BitTileCounts::default();
        h.insert(Tile::East);
        h.insert(Tile::East);
        h.insert(Tile::East);
        h.insert(Tile::South);
        h.insert(Tile::South);
        let res = parse_honors(h.rows[3]).unwrap();
        assert_eq!(res.pungs.len(), 1);
        assert_eq!(res.pair, Some(Tile::South));
    }

    #[test]
    fn test_parse_honors_two_pairs_invalid() {
        let mut h = BitTileCounts::default();
        h.insert(Tile::East);
        h.insert(Tile::East);
        h.insert(Tile::South);
        h.insert(Tile::South);
        assert!(parse_honors(h.rows[3]).is_none());
    }

    // ── find_all_decompositions ──

    #[test]
    fn test_find_decompositions_single_solution() {
        // 4 wind pungs + Red pair = 3*4 + 2 = 14 tiles
        let mut h = BitTileCounts::default();
        for _ in 0..3 {
            h.insert(Tile::East);
            h.insert(Tile::South);
            h.insert(Tile::West);
            h.insert(Tile::North);
        }
        h.insert(Tile::Red);
        h.insert(Tile::Red);
        let decomps = HuSolver::find_all_decompositions(&h);
        assert_eq!(decomps.len(), 1);
        assert_eq!(decomps[0].pair_tile, Tile::Red);
        assert_eq!(decomps[0].sets.len(), 4);
    }

    #[test]
    fn test_find_decompositions_pure_suit() {
        let tiles = [
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Character2,
            Tile::Character3,
            Tile::Character4,
            Tile::Character6,
            Tile::Character7,
            Tile::Character8,
            Tile::Character9,
            Tile::Character9,
            Tile::Character9,
            Tile::Character5,
            Tile::Character5,
        ];
        let h = single_suit_hand(&tiles);
        let decomps = HuSolver::find_all_decompositions(&h);
        assert!(!decomps.is_empty());
        for d in &decomps {
            assert_eq!(d.pair_tile, Tile::Character5);
        }
    }

    #[test]
    fn test_find_decompositions_eye_in_honors() {
        // 3x East, 3x South, 3x Char1, 3x Bam1, 2x Red = 14 tiles
        let mut h = BitTileCounts::default();
        for _ in 0..3 {
            h.insert(Tile::East);
            h.insert(Tile::South);
            h.insert(Tile::Character1);
            h.insert(Tile::Bamboo1);
        }
        h.insert(Tile::Red);
        h.insert(Tile::Red);
        let decomps = HuSolver::find_all_decompositions(&h);
        assert!(!decomps.is_empty());
        for d in &decomps {
            assert_eq!(d.pair_tile, Tile::Red);
        }
    }

    #[test]
    fn test_find_decompositions_fewer_sets() {
        // 11 tiles = 3 sets + 1 pair — e.g., 1 declared meld.
        // 3x East, 3x South, 3x Char1, 2x Red
        let mut h = BitTileCounts::default();
        for _ in 0..3 {
            h.insert(Tile::East);
            h.insert(Tile::South);
            h.insert(Tile::Character1);
        }
        h.insert(Tile::Red);
        h.insert(Tile::Red);
        let decomps = HuSolver::find_all_decompositions(&h);
        assert!(!decomps.is_empty());
        for d in &decomps {
            assert_eq!(d.pair_tile, Tile::Red);
        }
    }

    #[test]
    fn test_find_decompositions_multiple_solutions() {
        // 14 tiles, all Characters, multiple valid decompositions.
        let mut h = BitTileCounts::default();
        for _ in 0..3 {
            h.insert(Tile::Character1);
        }
        h.insert(Tile::Character2);
        h.insert(Tile::Character3);
        h.insert(Tile::Character4);
        h.insert(Tile::Character5);
        h.insert(Tile::Character6);
        h.insert(Tile::Character7);
        for _ in 0..3 {
            h.insert(Tile::Character9);
        }
        h.insert(Tile::Character8);
        h.insert(Tile::Character8);
        let decomps = HuSolver::find_all_decompositions(&h);
        assert!(decomps.len() >= 1);
    }

    #[test]
    fn test_find_decompositions_invalid_hand() {
        // 14 tiles, but a singleton Char1 prevents decomposition:
        // 3x East, 3x South, 3x West, 2x North, 2x Red, 1x Char1.
        let mut h = BitTileCounts::default();
        for _ in 0..3 {
            h.insert(Tile::East);
            h.insert(Tile::South);
            h.insert(Tile::West);
        }
        h.insert(Tile::North);
        h.insert(Tile::North);
        h.insert(Tile::Red);
        h.insert(Tile::Red);
        h.insert(Tile::Character1);
        assert!(HuSolver::find_all_decompositions(&h).is_empty());
    }

    #[test]
    fn test_find_decompositions_wrong_tile_count() {
        // 2 tiles + no other tiles → total=2, (2-2)/3 = 0 sets, 1 pair.
        // This CAN be a valid hand: 0 sets + 1 pair = 2 tiles.
        // But honors parse checks: 2 of the same honor → valid pair.
        let mut h = BitTileCounts::default();
        h.insert(Tile::East);
        h.insert(Tile::East);
        let decomps = HuSolver::find_all_decompositions(&h);
        assert!(!decomps.is_empty());
        assert_eq!(decomps[0].sets.len(), 0);
        assert_eq!(decomps[0].pair_tile, Tile::East);

        // 3 tiles → total=3, (3-2)/3 = 1/3, not integer → no decomposition.
        let mut h2 = BitTileCounts::default();
        h2.insert(Tile::East);
        h2.insert(Tile::East);
        h2.insert(Tile::East);
        assert!(HuSolver::find_all_decompositions(&h2).is_empty());
    }

    #[test]
    fn test_find_decompositions_empty_counts() {
        let h = BitTileCounts::default();
        assert!(HuSolver::find_all_decompositions(&h).is_empty());
    }
}
