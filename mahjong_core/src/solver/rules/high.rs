// ═══════════════════════════════════════════════════════════════
// 88 / 64 / 48 point rules
// ═══════════════════════════════════════════════════════════════

#![allow(non_snake_case)]

use super::super::{DynamicFanContext, StaticFanContext, WaitType};
use super::helpers::{
    MeldKind, cand, is_chow, is_green_tile, is_pung_or_kong, is_terminal_tile,
    meld_info, pair_info, rank_of, suit_of,
};
use super::{FanCandidate, FanType};
use crate::solver::{DecomposeResult, Decomposition};
use crate::structs::Tile;

// ── 88 points ──────────────────────────────────────────────────

pub(crate) fn big_four_winds(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    if sets.len() != 4 {
        return vec![];
    }
    let n = sets.len();
    if sets.iter().all(is_pung_or_kong) {
        let winds = [Tile::East, Tile::South, Tile::West, Tile::North];
        let tiles: Vec<Tile> =
            sets.iter().map(|m| meld_info(m).tile).collect();
        if winds.iter().all(|w| tiles.contains(w)) {
            return vec![cand(FanType::BigFourWinds, 0b1111, false)];
        }
    }
    vec![]
}

pub(crate) fn big_three_dragons(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let found: Vec<Tile> = sets
        .iter()
        .filter(|m| meld_info(m).is_dragon && is_pung_or_kong(m))
        .map(|m| meld_info(m).tile)
        .collect();
    if found.len() >= 3 {
        return vec![cand(FanType::BigThreeDragons, 0b1111, false)];
    }
    vec![]
}

pub(crate) fn all_green(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    match &decomp.decompositions {
        Standard => {
            let n = sets.len();
            let all_green = sets.iter().all(|m| match meld_info(m).kind {
                MeldKind::Chow => {
                    meld_info(m).tiles[..3].iter().all(|&t| is_green_tile(t))
                }
                _ => is_green_tile(meld_info(m).tile),
            }) && is_green_tile(pair_info(pair).tile);
            if all_green {
                vec![cand(FanType::AllGreen, 0, true)]
            } else {
                vec![]
            }
        }
        Decomposition::SevenPairs { pairs } => {
            if pairs.iter().all(|t| is_green_tile(t.tile())) {
                vec![cand(FanType::AllGreen, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

pub(crate) fn nine_gates(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let s = meld_info(&sets[0]).suit;
    if s >= 3 {
        return vec![];
    }
    if !sets
        .iter()
        .all(|m| meld_info(m).suit == s && !meld_info(m).is_honor)
    {
        return vec![];
    }
    if pair_info(pair).suit != s || pair_info(pair).is_honor {
        return vec![];
    }

    let mut counts = [0u8; 9];
    for m in sets.iter() {
        match meld_info(m).kind {
            MeldKind::Pung => counts[meld_info(m).rank as usize - 1] += 3,
            MeldKind::Kong => counts[meld_info(m).rank as usize - 1] += 4,
            MeldKind::Chow => {
                let r = meld_info(m).rank as usize - 1;
                counts[r] += 1;
                counts[r + 1] += 1;
                counts[r + 2] += 1;
            }
        }
    }
    counts[pair_info(pair).rank as usize - 1] += 2;

    if counts.iter().all(|&c| c >= 1) {
        let expected_base = [3u8, 1, 1, 1, 1, 1, 1, 1, 3];
        let mut diff_count = 0;
        for i in 0..9 {
            if counts[i] < expected_base[i] {
                return vec![];
            }
            if counts[i] > expected_base[i] {
                diff_count += 1;
            }
        }
        if diff_count == 1 {
            return vec![cand(FanType::NineGates, 0, true)];
        }
    }
    vec![]
}

pub(crate) fn four_kongs(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };

    if !matches!(decomp.decompositions, Decomposition::Standard { .. }) {
        return vec![];
    }
    if sets
        .iter()
        .filter(|m| matches!(meld_info(m).kind, MeldKind::Kong))
        .count() as u8
        == 4
    {
        return vec![cand(FanType::FourKongs, 0b1111, false)];
    }
    vec![]
}

pub(crate) fn seven_shifted_pairs(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    match &decomp.decompositions {
        Decomposition::SevenPairs { pairs } => {
            let s = suit_of(pairs[0].tile());
            if pairs.iter().all(|t| suit_of(t.tile()) == s) {
                let mut ranks: Vec<u8> =
                    pairs.iter().map(|t| rank_of(t.tile())).collect();
                ranks.sort();
                if ranks.windows(2).all(|w| w[1] == w[0] + 1) {
                    return vec![cand(FanType::SevenShiftedPairs, 0, true)];
                }
            }
            vec![]
        }
        _ => vec![],
    }
}

pub(crate) fn thirteen_orphans(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if matches!(decomp.decompositions, Decomposition::ThirteenOrphans { .. }) {
        vec![cand(FanType::ThirteenOrphans, 0, true)]
    } else {
        vec![]
    }
}

// ── 64 points ──────────────────────────────────────────────────

pub(crate) fn all_terminals(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    match &decomp.decompositions {
        Standard => {
            // Check meld_tile (start tile for chows), matching old code's
            // is_terminal(meld_tile(*m)) behavior.
            let n = sets.len();
            if sets.iter().all(|m| is_terminal_tile(meld_info(m).tile))
                && is_terminal_tile(pair_info(pair).tile)
            {
                vec![cand(FanType::AllTerminals, 0, true)]
            } else {
                vec![]
            }
        }
        Decomposition::SevenPairs { pairs } => {
            if pairs.iter().all(|t| is_terminal_tile(t.tile())) {
                vec![cand(FanType::AllTerminals, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

pub(crate) fn little_four_winds(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    if !pair_info(pair).is_wind {
        return vec![];
    }
    let wind_pungs: Vec<Tile> = sets
        .iter()
        .filter(|m| meld_info(m).is_wind && is_pung_or_kong(m))
        .map(|m| meld_info(m).tile)
        .collect();
    if wind_pungs.len() < 3 {
        return vec![];
    }
    let all = [Tile::East, Tile::South, Tile::West, Tile::North];
    let missing: Vec<Tile> = all
        .iter()
        .filter(|w| !wind_pungs.contains(w) && **w != pair_info(pair).tile)
        .copied()
        .collect();
    if missing.is_empty() {
        vec![cand(FanType::LittleFourWinds, 0b1111, true)]
    } else {
        vec![]
    }
}

pub(crate) fn little_three_dragons(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let dp: Vec<Tile> = sets
        .iter()
        .filter(|m| meld_info(m).is_dragon && is_pung_or_kong(m))
        .map(|m| meld_info(m).tile)
        .collect();
    if dp.len() == 2 && pair_info(pair).is_dragon {
        let all = [Tile::Red, Tile::Green, Tile::White];
        let missing: Vec<Tile> = all
            .iter()
            .filter(|d| !dp.contains(d) && **d != pair_info(pair).tile)
            .copied()
            .collect();
        if missing.is_empty() {
            return vec![cand(FanType::LittleThreeDragons, 0b1111, true)];
        }
    }
    vec![]
}

pub(crate) fn all_honors(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    if sets.iter().all(|m| meld_info(m).is_honor) && pair_info(pair).is_honor {
        vec![cand(FanType::AllHonors, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn four_concealed_pungs(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    if sets.len() != 4 {
        return vec![];
    }
    let n = sets.len();
    if sets
        .iter()
        .all(|m| is_pung_or_kong(m) && meld_info(m).is_concealed)
    {
        vec![cand(FanType::FourConcealedPungs, 0b1111, false)]
    } else {
        vec![]
    }
}

pub(crate) fn pure_terminal_chows(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    if sets.len() != 4 {
        return vec![];
    }
    if pair_info(pair).is_honor || pair_info(pair).rank != 5 {
        return vec![];
    }
    let s = meld_info(&sets[0]).suit;
    if pair_info(pair).suit != s {
        return vec![];
    }
    let n = sets.len();
    if !sets.iter().all(|m| is_chow(m) && meld_info(m).suit == s) {
        return vec![];
    }
    let r1 = sets.iter().filter(|m| meld_info(m).rank == 1).count() as u8;
    let r7 = sets.iter().filter(|m| meld_info(m).rank == 7).count() as u8;
    if r1 == 2 && r7 == 2 {
        vec![cand(FanType::PureTerminalChows, 0, true)]
    } else {
        vec![]
    }
}

// ── 48 points ──────────────────────────────────────────────────

pub(crate) fn quadruple_chow(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let chows: Vec<(usize, u8, u8)> = sets
        .iter()
        .enumerate()
        .filter(|(_, m)| is_chow(m))
        .map(|(i, m)| (i, meld_info(m).rank, meld_info(m).suit))
        .collect();
    if chows.len() < 4 {
        return vec![];
    }
    for a in 0..chows.len() {
        for b in a + 1..chows.len() {
            for c in b + 1..chows.len() {
                for d in c + 1..chows.len() {
                    if chows[a].1 == chows[b].1
                        && chows[a].1 == chows[c].1
                        && chows[a].1 == chows[d].1
                        && chows[a].2 == chows[b].2
                        && chows[a].2 == chows[c].2
                        && chows[a].2 == chows[d].2
                    {
                        return vec![cand(
                            FanType::QuadrupleChow,
                            0b1111,
                            false,
                        )];
                    }
                }
            }
        }
    }
    vec![]
}

pub(crate) fn four_pure_shifted_pungs(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let p: Vec<(usize, u8, u8)> = sets
        .iter()
        .enumerate()
        .filter(|(_, m)| is_pung_or_kong(m))
        .map(|(i, m)| (i, meld_info(m).rank, meld_info(m).suit))
        .collect();
    if p.len() < 4 {
        return vec![];
    }
    for a in 0..p.len() {
        for b in a + 1..p.len() {
            for c in b + 1..p.len() {
                for d in c + 1..p.len() {
                    let mut ranks = [p[a].1, p[b].1, p[c].1, p[d].1];
                    ranks.sort();
                    if p[a].2 == p[b].2
                        && p[a].2 == p[c].2
                        && p[a].2 == p[d].2
                        && ranks[1] == ranks[0] + 1
                        && ranks[2] == ranks[1] + 1
                        && ranks[3] == ranks[2] + 1
                    {
                        return vec![cand(
                            FanType::FourPureShiftedPungs,
                            0b1111,
                            false,
                        )];
                    }
                }
            }
        }
    }
    vec![]
}

// ═══════════════════════════════════════════════════════════════
// Tests — sourced from the official MCR rulebook
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::super::FanType;
    use super::super::test_helpers::*;
    use crate::solver::{DynamicFanContext, StaticFanContext, WinMethod};
    use crate::structs::{Hand, Meld, Quad, Sequence, Tile, Triplet, Wind};

    #[test]
    fn test_big_four_winds_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::East, true)),
                Meld::Pung(Triplet::new(Tile::South, true)),
                Meld::Pung(Triplet::new(Tile::West, true)),
                Meld::Pung(Triplet::new(Tile::North, true)),
            ],
            Tile::Red,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigFourWinds);
        assert_not_contains_fan(&result, FanType::AllPungs);
        assert_not_contains_fan(&result, FanType::BigThreeWinds);
        // Combined with All Honors (stub — will pass when implemented)
        // assert_contains_fan(&result, FanType::AllHonors);
    }

    #[test]
    fn test_big_four_winds_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::East, true)),
                Meld::Pung(Triplet::new(Tile::South, true)),
                Meld::Pung(Triplet::new(Tile::West, true)),
                Meld::Pung(Triplet::new(Tile::North, true)),
            ],
            Tile::Character9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigFourWinds);
    }

    #[test]
    fn test_big_four_winds_example3() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::East, true)),
                Meld::Pung(Triplet::new(Tile::South, true)),
                Meld::Pung(Triplet::new(Tile::West, true)),
                Meld::Pung(Triplet::new(Tile::North, true)),
            ],
            Tile::Bamboo3,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigFourWinds);
    }

    // ── 2. Big Three Dragons ──

    #[test]
    fn test_big_three_dragons_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, true)),
                Meld::Pung(Triplet::new(Tile::Green, true)),
                Meld::Pung(Triplet::new(Tile::White, true)),
                Meld::Pung(Triplet::new(Tile::Character1, true)),
            ],
            Tile::Dot9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigThreeDragons);
        assert_not_contains_fan(&result, FanType::TwoDragonPungs);
        assert_not_contains_fan(&result, FanType::DragonPung);
    }

    #[test]
    fn test_big_three_dragons_example2() {
        // Original doc: pair = White, but White pung + White pair = 5 tiles
        // which is impossible in MCR. Adjusted to West per the user's fix.
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, true)),
                Meld::Pung(Triplet::new(Tile::Green, true)),
                Meld::Pung(Triplet::new(Tile::White, true)),
                Meld::Pung(Triplet::new(Tile::East, true)),
            ],
            Tile::West,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigThreeDragons);
    }

    #[test]
    fn test_big_three_dragons_example3() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, true)),
                Meld::Pung(Triplet::new(Tile::Green, true)),
                Meld::Pung(Triplet::new(Tile::White, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo4, true)),
            ],
            Tile::Bamboo5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigThreeDragons);
    }

    // ── 3. All Green ──

    #[test]
    fn test_all_green_example1() {
        // Seven pairs of green tiles + Green Dragon
        // Use declared hand with 4 pungs of green tiles + pair Green Dragon
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Bamboo2, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo4, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo6, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
            ],
            Tile::Green,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::AllGreen);
    }

    // ── 4. Nine Gates ──

    #[test]
    fn test_nine_gates() {
        // 1112345678999 Wan + winning tile Wan9
        // Need all in concealed hand for the decomposition engine
        let mut hand = Hand::default();
        // 1×3, 2,3,4,5,6,7,8, 9×4 = 14 tiles
        for _ in 0..3 {
            hand.concealed.insert(Tile::Character1);
        }
        hand.concealed.insert(Tile::Character2);
        hand.concealed.insert(Tile::Character3);
        hand.concealed.insert(Tile::Character4);
        hand.concealed.insert(Tile::Character5);
        hand.concealed.insert(Tile::Character6);
        hand.concealed.insert(Tile::Character7);
        hand.concealed.insert(Tile::Character8);
        for _ in 0..4 {
            hand.concealed.insert(Tile::Character9);
        }
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::NineGates);
    }

    // ── 5. Four Kongs ──

    #[test]
    fn test_four_kongs_example1() {
        // Kongs must be declared melds — cannot have 4 identical tiles concealed.
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Bamboo2, false)),
                Meld::Kong(Quad::new(Tile::Character5, false)),
                Meld::Kong(Quad::new(Tile::Dot7, false)),
                Meld::Kong(Quad::new(Tile::East, false)),
            ],
            Tile::Red,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::FourKongs);
        assert_not_contains_fan(&result, FanType::SingleWait);
    }

    #[test]
    fn test_four_kongs_example2() {
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Character1, false)),
                Meld::Kong(Quad::new(Tile::Bamboo2, false)),
                Meld::Kong(Quad::new(Tile::Dot7, false)),
                Meld::Kong(Quad::new(Tile::Bamboo1, false)),
            ],
            Tile::Character4,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::FourKongs);
    }

    #[test]
    fn test_four_kongs_example3() {
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Red, false)),
                Meld::Kong(Quad::new(Tile::Green, false)),
                Meld::Kong(Quad::new(Tile::White, false)),
                Meld::Kong(Quad::new(Tile::North, false)),
            ],
            Tile::East,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::FourKongs);
    }

    // ── 6. Seven Shifted Pairs ──

    #[test]
    fn test_seven_shifted_pairs() {
        // Dot2-2, Dot3-3, Dot4-4, Dot5-5, Dot6-6, Dot7-7, Dot8-8
        // All 14 tiles in concealed — special decomposition engine handles it
        let mut hand = Hand::default();
        for i in 2..=8 {
            let t = match i {
                2 => Tile::Dot2,
                3 => Tile::Dot3,
                4 => Tile::Dot4,
                5 => Tile::Dot5,
                6 => Tile::Dot6,
                7 => Tile::Dot7,
                8 => Tile::Dot8,
                _ => unreachable!(),
            };
            hand.concealed.insert(t);
            hand.concealed.insert(t);
        }
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::SevenShiftedPairs);
    }

    // ── 7. Thirteen Orphans ──

    #[test]
    fn test_thirteen_orphans() {
        // 1-9 each suit + all honors + pair of one
        let mut hand = Hand::default();
        // 13 orphans: Dot1, Dot9, Bam1, Bam9, Wan1, Wan9, all 7 honors
        hand.concealed.insert(Tile::Dot1);
        hand.concealed.insert(Tile::Dot9);
        hand.concealed.insert(Tile::Bamboo1);
        hand.concealed.insert(Tile::Bamboo9);
        hand.concealed.insert(Tile::Character1);
        hand.concealed.insert(Tile::Character9);
        hand.concealed.insert(Tile::East);
        hand.concealed.insert(Tile::South);
        hand.concealed.insert(Tile::West);
        hand.concealed.insert(Tile::North);
        hand.concealed.insert(Tile::Red);
        hand.concealed.insert(Tile::Green);
        hand.concealed.insert(Tile::White);
        // Pair: North
        hand.concealed.insert(Tile::North);
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ThirteenOrphans);
    }

    // ═══════════════════════════════════════════════════════════════
    // 64-point fans
    // ═══════════════════════════════════════════════════════════════

    // ── 8. All Terminals ──

    #[test]
    fn test_all_terminals() {
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Character1, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo1, false)),
                Meld::Pung(Triplet::new(Tile::Character9, false)),
                Meld::Pung(Triplet::new(Tile::Dot9, false)),
            ],
            Tile::Bamboo1,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::AllTerminals);
        assert_not_contains_fan(&result, FanType::NoHonors);
        assert_not_contains_fan(&result, FanType::PungOfTerminalsOrHonors);
    }

    // ── 9. Little Four Winds ──

    #[test]
    fn test_little_four_winds_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::East, true)),
                Meld::Pung(Triplet::new(Tile::West, true)),
                Meld::Pung(Triplet::new(Tile::North, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            ],
            Tile::South,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LittleFourWinds);
        assert_not_contains_fan(&result, FanType::BigThreeWinds);
    }

    #[test]
    fn test_little_four_winds_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::West, true)),
                Meld::Pung(Triplet::new(Tile::South, true)),
                Meld::Pung(Triplet::new(Tile::North, true)),
                Meld::Pung(Triplet::new(Tile::White, true)),
            ],
            Tile::East,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LittleFourWinds);
    }

    // ── 10. Little Three Dragons ──

    #[test]
    fn test_little_three_dragons_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, true)),
                Meld::Pung(Triplet::new(Tile::Green, true)),
                Meld::Pung(Triplet::new(Tile::Dot1, true)),
                Meld::Pung(Triplet::new(Tile::Dot9, true)),
            ],
            Tile::White,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LittleThreeDragons);
        assert_not_contains_fan(&result, FanType::DragonPung);
        assert_not_contains_fan(&result, FanType::TwoDragonPungs);
    }

    #[test]
    fn test_little_three_dragons_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, true)),
                Meld::Pung(Triplet::new(Tile::White, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
                Meld::Pung(Triplet::new(Tile::Dot1, true)),
            ],
            Tile::Green,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LittleThreeDragons);
    }

    #[test]
    fn test_little_three_dragons_example3() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, true)),
                Meld::Pung(Triplet::new(Tile::White, true)),
                Meld::Pung(Triplet::new(Tile::East, true)),
                Meld::Pung(Triplet::new(Tile::North, true)),
            ],
            Tile::Green,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LittleThreeDragons);
    }

    // ── 11. All Honors ──

    #[test]
    fn test_all_honors() {
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, false)),
                Meld::Pung(Triplet::new(Tile::White, false)),
                Meld::Pung(Triplet::new(Tile::East, false)),
                Meld::Pung(Triplet::new(Tile::South, false)),
            ],
            Tile::North,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::AllHonors);
        assert_not_contains_fan(&result, FanType::PungOfTerminalsOrHonors);
    }

    // ── 12. Four Concealed Pungs ──

    #[test]
    fn test_four_concealed_pungs() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Bamboo2, true)),
                Meld::Pung(Triplet::new(Tile::Dot3, true)),
                Meld::Pung(Triplet::new(Tile::Character4, true)),
                Meld::Pung(Triplet::new(Tile::Dot2, true)),
            ],
            Tile::Character3,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::FourConcealedPungs);
        assert_not_contains_fan(&result, FanType::AllPungs);
        assert_not_contains_fan(&result, FanType::ConcealedHand);
    }

    // ── 13. Pure Terminal Chows ──

    #[test]
    fn test_pure_terminal_chows() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            ],
            Tile::Bamboo5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::PureTerminalChows);
    }

    // ═══════════════════════════════════════════════════════════════
    // 48-point fans
    // ═══════════════════════════════════════════════════════════════

    // ── 14. Quadruple Chow ──

    #[test]
    fn test_quadruple_chow() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
            ],
            Tile::Character4,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::QuadrupleChow);
    }

    // ── 15. Four Pure Shifted Pungs ──
}
