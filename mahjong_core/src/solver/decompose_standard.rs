use crate::array_vec::ArrayVec;
use crate::structs::{BitTileCounts, Meld, Sequence, Tile, Triplet};

/// Result of decomposing only the concealed tiles (no declared melds).
/// Private to the standard decomposer; the orchestration layer combines
/// these with declared melds to produce `Decomposition::Standard`.
pub(super) struct ConcealedDecompStd {
    pub(super) pair_tile: Tile,
    pub(super) melds: ArrayVec<Meld, 4>,
}

// ────────────────────────────────────────────────────────────────
// SuitDecomp — stack-allocated meld sequence for DFS
// ────────────────────────────────────────────────────────────────

type SuitDecomp = ArrayVec<Meld, 4>;
type SuitDecomps = Vec<SuitDecomp>;

// ────────────────────────────────────────────────────────────────
// Suit solver — recursive DFS
// ────────────────────────────────────────────────────────────────

/// Enumerate all ways to partition a suit row into pongs and chows.
/// Returns an empty vec if the row cannot be fully decomposed.
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

// ────────────────────────────────────────────────────────────────
// Honor parser
// ────────────────────────────────────────────────────────────────

struct HonorResult {
    pungs: Vec<Meld>,
    pair: Option<Tile>,
}

/// Parse the honors row. Returns `None` if the row has a structure
/// that cannot appear in any winning hand (singles, concealed kongs,
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

// ────────────────────────────────────────────────────────────────
// Suit combination — Cartesian product across three suits
// ────────────────────────────────────────────────────────────────

/// Cartesian product of one decomposition from each suit.
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
                combined.extend_from(d0);
                combined.extend_from(d1);
                combined.extend_from(d2);
                results.push(combined);
            }
        }
    }
    results
}

fn combine_suits(cache: &[SuitDecomps; 3]) -> SuitDecomps {
    combine_triple(&cache[0], &cache[1], &cache[2])
}

fn combine_suits_with(
    cache: &[SuitDecomps; 3],
    idx: usize,
    replacement: &SuitDecomps,
) -> SuitDecomps {
    let s = |i: usize| if i == idx { replacement } else { &cache[i] };
    combine_triple(s(0), s(1), s(2))
}

// ────────────────────────────────────────────────────────────────
// Main entry — enumerate all standard decompositions
// ────────────────────────────────────────────────────────────────

/// Enumerate all ways to partition the concealed tiles into
/// sets + a pair (standard pattern).
pub(crate) fn decompose_standard(
    counts: &BitTileCounts,
) -> Vec<ConcealedDecompStd> {
    let Some(honors) = parse_honors(counts.rows[3]) else {
        return vec![];
    };

    let suit_cache = decompose_all_suits(&counts.rows[..3]);
    let mut out = Vec::new();

    // Case A: pair is in honors
    if let Some(pair_tile) = honors.pair {
        if counts.rows[0] != 0 && suit_cache[0].is_empty()
            || counts.rows[1] != 0 && suit_cache[1].is_empty()
            || counts.rows[2] != 0 && suit_cache[2].is_empty()
        {
            return vec![];
        }
        let combos = combine_suits(&suit_cache);
        emit_standard(&honors.pungs, pair_tile, combos, &mut out);
        return out;
    }

    // Case B: pair is in a suit
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

// ── Internal helpers ──

fn decompose_all_suits(rows: &[u64]) -> [SuitDecomps; 3] {
    [
        decompose_suit(rows[0], 0),
        decompose_suit(rows[1], 1),
        decompose_suit(rows[2], 2),
    ]
}

fn emit_standard(
    honor_pungs: &[Meld],
    pair_tile: Tile,
    combos: SuitDecomps,
    out: &mut Vec<ConcealedDecompStd>,
) {
    for combo in &combos {
        let mut sets = ArrayVec::new();
        for &pung in honor_pungs {
            sets.push(pung);
        }
        for i in 0..combo.len() {
            sets.push(combo[i]);
        }
        out.push(ConcealedDecompStd {
            pair_tile,
            melds: sets,
        });
    }
}

fn try_each_pair_in_suit(
    suit_idx: usize,
    counts: &BitTileCounts,
    honor_pungs: &[Meld],
    suit_cache: &[SuitDecomps; 3],
    out: &mut Vec<ConcealedDecompStd>,
) {
    // Other suits' decomposability doesn't depend on pair position.
    for (i, cache) in suit_cache.iter().enumerate() {
        if i != suit_idx && counts.rows[i] != 0 && cache.is_empty() {
            return;
        }
    }

    let row = counts.rows[suit_idx];
    for shift in pair_positions_in_row(row) {
        let pair_tile = BitTileCounts::position_to_tile(suit_idx, shift);

        let row_minus_pair = {
            let mut r = row;
            BitTileCounts::remove_nibble(&mut r, shift, 2);
            r
        };
        let pair_decomps = decompose_suit(row_minus_pair, suit_idx);
        if row_minus_pair != 0 && pair_decomps.is_empty() {
            continue;
        }
        let combos = combine_suits_with(suit_cache, suit_idx, &pair_decomps);
        emit_standard(honor_pungs, pair_tile, combos, out);
    }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::{Meld, Tile};

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
        assert!(matches!(results[0][0], Meld::Pung(_)));
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
        assert!(matches!(results[0][0], Meld::Chow(_)));
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

    // ── decompose_standard ──

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
        assert!(!decompose_standard(&hand).is_empty());
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
        assert!(!decompose_standard(&hand).is_empty());
    }

    #[test]
    fn test_invalid_hand() {
        let hand = hand_from(&[
            (Tile::Character1, 2),
            (Tile::Character2, 1),
            (Tile::Character3, 1),
        ]);
        assert!(decompose_standard(&hand).is_empty());
    }

    #[test]
    fn test_decompositions_single_solution() {
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
        let decomps = decompose_standard(&h);
        assert_eq!(decomps.len(), 1);
        assert_eq!(decomps[0].pair_tile, Tile::Red);
        assert_eq!(decomps[0].melds.len(), 4);
    }

    #[test]
    fn test_decompositions_pure_suit() {
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
        let decomps = decompose_standard(&h);
        assert!(!decomps.is_empty());
        for d in &decomps {
            assert_eq!(d.pair_tile, Tile::Character5);
        }
    }

    #[test]
    fn test_decompositions_eye_in_honors() {
        let mut h = BitTileCounts::default();
        for _ in 0..3 {
            h.insert(Tile::East);
            h.insert(Tile::South);
            h.insert(Tile::Character1);
            h.insert(Tile::Bamboo1);
        }
        h.insert(Tile::Red);
        h.insert(Tile::Red);
        let decomps = decompose_standard(&h);
        assert!(!decomps.is_empty());
        for d in &decomps {
            assert_eq!(d.pair_tile, Tile::Red);
        }
    }

    #[test]
    fn test_decompositions_fewer_sets() {
        // 11 tiles = 3 sets + 1 pair — e.g., 1 declared meld.
        let mut h = BitTileCounts::default();
        for _ in 0..3 {
            h.insert(Tile::East);
            h.insert(Tile::South);
            h.insert(Tile::Character1);
        }
        h.insert(Tile::Red);
        h.insert(Tile::Red);
        let decomps = decompose_standard(&h);
        assert!(!decomps.is_empty());
        for d in &decomps {
            assert_eq!(d.pair_tile, Tile::Red);
        }
    }

    #[test]
    fn test_decompositions_multiple_solutions() {
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
        let decomps = decompose_standard(&h);
        assert!(decomps.len() >= 1);
    }

    #[test]
    fn test_decompositions_invalid_hand() {
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
        assert!(decompose_standard(&h).is_empty());
    }

    #[test]
    fn test_decompositions_wrong_tile_count() {
        // 2 tiles of the same honor = valid pair-only decomposition.
        let mut h = BitTileCounts::default();
        h.insert(Tile::East);
        h.insert(Tile::East);
        let decomps = decompose_standard(&h);
        assert!(!decomps.is_empty());
        assert_eq!(decomps[0].melds.len(), 0);
        assert_eq!(decomps[0].pair_tile, Tile::East);

        // 3 tiles of the same honor can't form a valid hand.
        let mut h2 = BitTileCounts::default();
        h2.insert(Tile::East);
        h2.insert(Tile::East);
        h2.insert(Tile::East);
        assert!(decompose_standard(&h2).is_empty());
    }

    #[test]
    fn test_decompositions_empty_counts() {
        let h = BitTileCounts::default();
        assert!(decompose_standard(&h).is_empty());
    }
}
