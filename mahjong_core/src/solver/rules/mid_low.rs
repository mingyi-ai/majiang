// ═══════════════════════════════════════════════════════════════
// 12 / 8 / 6 point rules
// ═══════════════════════════════════════════════════════════════

#![allow(non_snake_case)]

use std::collections::HashSet;

use super::super::{DynamicFanContext, StaticFanContext, WaitType, WinMethod};
use super::helpers::{
    MeldKind, all_in_range, cand, is_chow, is_pung_or_kong,
    is_reversible_tile, meld_info, pair_info, rank_of,
};
use super::{FanInstance, FanType};
use crate::solver::{DecomposeResult, Decomposition};

// ── 12 points ──────────────────────────────────────────────────

pub(crate) fn lesser_honors_and_knitted_tiles(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    match &decomp.decompositions {
        // LesserHonorsAndKnittedTiles => {
        //     vec![cand(FanType::LesserHonorsAndKnittedTiles, 0, true)]
        // }
        _ => vec![],
    }
}

pub(crate) fn knitted_straight(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    match &decomp.decompositions {
        // KnittedStraight => {
        //     vec![cand(FanType::KnittedStraight, 0, true)]
        // }
        _ => vec![],
    }
}

pub(crate) fn upper_four(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    match &decomp.decompositions {
        Decomposition::Standard { pair, sets }
            if all_in_range(sets.as_slice(), pair, 6, 9) =>
        {
            vec![cand(FanType::UpperFour, 0, true)]
        }
        Decomposition::SevenPairs { pairs } => {
            if pairs.iter().all(|t| {
                let r = rank_of(t.tile());
                r >= 6 && r <= 9
            }) {
                vec![cand(FanType::UpperFour, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

pub(crate) fn lower_four(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    match &decomp.decompositions {
        Decomposition::Standard { pair, sets }
            if all_in_range(sets.as_slice(), pair, 1, 4) =>
        {
            vec![cand(FanType::LowerFour, 0, true)]
        }
        Decomposition::SevenPairs { pairs } => {
            if pairs.iter().all(|t| {
                let r = rank_of(t.tile());
                r >= 1 && r <= 4
            }) {
                vec![cand(FanType::LowerFour, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

pub(crate) fn big_three_winds(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let cnt = sets
        .iter()
        .filter(|m| meld_info(m).is_wind && is_pung_or_kong(m))
        .count();
    if cnt >= 3 {
        vec![cand(FanType::BigThreeWinds, 0b1111, false)]
    } else {
        vec![]
    }
}

// ── 8 points ──────────────────────────────────────────────────

pub(crate) fn mixed_straight(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
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
    if chows.len() < 3 {
        return vec![];
    }
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            for k in j + 1..chows.len() {
                let mut r = [chows[i].1, chows[j].1, chows[k].1];
                r.sort();
                if r != [1, 4, 7] {
                    continue;
                }
                let suits: HashSet<u8> =
                    [chows[i].2, chows[j].2, chows[k].2].into();
                if suits.len() == 3 {
                    return vec![cand(FanType::MixedStraight, 0, false)];
                }
            }
        }
    }
    vec![]
}

pub(crate) fn reversible_tiles(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    match &decomp.decompositions {
        Standard => {
            let n = sets.len();
            let all_rev = sets.iter().all(|m| match meld_info(m).kind {
                MeldKind::Chow => meld_info(m).tiles[..3]
                    .iter()
                    .all(|&t| is_reversible_tile(t)),
                _ => is_reversible_tile(meld_info(m).tile),
            }) && is_reversible_tile(pair_info(pair).tile);
            if all_rev {
                vec![cand(FanType::ReversibleTiles, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

pub(crate) fn mixed_triple_chow(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
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
    if chows.len() < 3 {
        return vec![];
    }
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            for k in j + 1..chows.len() {
                if chows[i].1 == chows[j].1 && chows[i].1 == chows[k].1 {
                    let suits: HashSet<u8> =
                        [chows[i].2, chows[j].2, chows[k].2].into();
                    if suits.len() == 3 {
                        return vec![cand(
                            FanType::MixedTripleChow,
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

pub(crate) fn mixed_shifted_pungs(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
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
    if p.len() < 3 {
        return vec![];
    }
    for i in 0..p.len() {
        for j in i + 1..p.len() {
            for k in j + 1..p.len() {
                let suits: HashSet<u8> = [p[i].2, p[j].2, p[k].2].into();
                if suits.len() != 3 {
                    continue;
                }
                let mut ranks = [p[i].1, p[j].1, p[k].1];
                ranks.sort();
                if ranks[1] == ranks[0] + 1 && ranks[2] == ranks[1] + 1 {
                    return vec![cand(
                        FanType::MixedShiftedPungs,
                        0b1111,
                        false,
                    )];
                }
            }
        }
    }
    vec![]
}

pub(crate) fn chicken_hand(
    _decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    vec![]
}

pub(crate) fn last_tile_draw(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    if static_ctx.is_last_tile_draw {
        vec![cand(FanType::LastTileDraw, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn last_tile_claim(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    if static_ctx.is_last_tile_claim {
        vec![cand(FanType::LastTileClaim, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn out_with_replacement_tile(
    _decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    if dynamic_ctx.is_kong_replacement {
        vec![cand(FanType::OutWithReplacementTile, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn robbing_the_kong(
    _decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    if dynamic_ctx.is_rob_kong {
        vec![cand(FanType::RobbingTheKong, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn two_concealed_kongs(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };

    let n = sets.len();
    let cnt = sets
        .iter()
        .filter(|m| {
            matches!(meld_info(m).kind, MeldKind::Kong)
                && meld_info(m).is_concealed
        })
        .count();
    if cnt >= 2 {
        vec![cand(FanType::TwoConcealedKongs, 0b1111, false)]
    } else {
        vec![]
    }
}

// ── 6 points ──────────────────────────────────────────────────

pub(crate) fn all_pungs(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    if false {
        vec![cand(FanType::AllPungs, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn half_flush(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let mut suits = HashSet::new();
    for m in sets.iter() {
        if !meld_info(m).is_honor {
            suits.insert(meld_info(m).suit);
        }
    }
    if !pair_info(pair).is_honor {
        suits.insert(pair_info(pair).suit);
    }
    if false {
        vec![cand(FanType::HalfFlush, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn mixed_shifted_chows(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
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
    if chows.len() < 3 {
        return vec![];
    }
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            for k in j + 1..chows.len() {
                let suits: HashSet<u8> =
                    [chows[i].2, chows[j].2, chows[k].2].into();
                if suits.len() != 3 {
                    continue;
                }
                let mut ranks = [chows[i].1, chows[j].1, chows[k].1];
                ranks.sort();
                if ranks[1] == ranks[0] + 1 && ranks[2] == ranks[1] + 1 {
                    return vec![cand(
                        FanType::MixedShiftedChows,
                        0b1111,
                        false,
                    )];
                }
            }
        }
    }
    vec![]
}

pub(crate) fn all_types(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let mut mask: u8 = 0;
    for m in sets.iter() {
        mask |= 1 << meld_info(m).suit.min(2); // map suit 0-2 to bits 0-2; honor winds/dragons go to bit 3
    }
    if pair_info(pair).is_honor {
        mask |= 1 << 3; // honors
    } else {
        mask |= 1 << pair_info(pair).suit;
    }
    // All 5 types (3 suits + winds + dragons) need 5 bits
    // We track: bit 0=char, 1=dot, 2=bamboo, 3=winds, 4=dragons
    let mut full_mask: u16 = 0;
    for m in sets.iter() {
        if meld_info(m).is_dragon {
            full_mask |= 1 << 4;
        } else if meld_info(m).is_wind {
            full_mask |= 1 << 3;
        } else if meld_info(m).suit < 3 {
            full_mask |= 1 << meld_info(m).suit;
        }
    }
    if pair_info(pair).is_dragon {
        full_mask |= 1 << 4;
    } else if pair_info(pair).is_wind {
        full_mask |= 1 << 3;
    } else if pair_info(pair).suit < 3 {
        full_mask |= 1 << pair_info(pair).suit;
    }
    if full_mask == 0b11111 {
        vec![cand(FanType::AllTypes, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn melded_hand(
    decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let all_exposed = sets.iter().all(|m| !meld_info(m).is_concealed);
    if all_exposed
        && static_ctx.win_method == WinMethod::Discard
        && wait_type == WaitType::Single
    {
        vec![cand(FanType::MeldedHand, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn two_dragon_pungs(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };

    let n = sets.len();
    let cnt = sets
        .iter()
        .filter(|m| meld_info(m).is_dragon && is_pung_or_kong(m))
        .count();
    if cnt >= 2 {
        vec![cand(FanType::TwoDragonPungs, 0b1111, false)]
    } else {
        vec![]
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests — sourced from the official MCR rulebook
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::super::FanType;
    use super::super::test_helpers::*;
    use crate::solver::{
        DynamicFanContext, StaticFanContext, WaitType, WinMethod, solve_fan,
    };
    use crate::structs::{Hand, Meld, Quad, Sequence, Tile, Triplet, Wind};

    #[test]
    fn test_three_concealed_pungs_example2() {
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Bamboo4, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo7, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo8, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo1, false)),
            ],
            Tile::Bamboo5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ThreeConcealedPungs);
    }

    // ═══════════════════════════════════════════════════════════════
    // 12-point fans
    // ═══════════════════════════════════════════════════════════════

    // ── 34. Lesser Honors and Knitted Tiles ──

    #[test]
    fn test_lesser_honors_and_knitted_example1() {
        // Requires special decomposition — test through standard engine
        // when decompose_special supports it.
    }

    // ── 35. Knitted Straight ──

    #[test]
    fn test_knitted_straight_example1() {
        // Requires special decomposition — test through standard engine
        // when decompose_special supports it.
    }

    // ── 36. Upper Four ──

    #[test]
    fn test_upper_four_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
                Meld::Chow(Sequence::new(Tile::Dot6, true)),
                Meld::Pung(Triplet::new(Tile::Dot7, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
            ],
            Tile::Bamboo9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::UpperFour);
        assert_not_contains_fan(&result, FanType::NoHonors);
    }

    #[test]
    fn test_upper_four_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
                Meld::Chow(Sequence::new(Tile::Dot7, true)),
                Meld::Chow(Sequence::new(Tile::Dot7, true)),
            ],
            Tile::Dot6,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::UpperFour);
    }

    #[test]
    fn test_upper_four_example3() {
        let hand = seven_pairs_hand(&[
            Tile::Bamboo6,
            Tile::Bamboo8,
            Tile::Character6,
            Tile::Character7,
            Tile::Dot6,
            Tile::Dot8,
            Tile::Dot9,
        ]);
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::UpperFour);
    }

    // ── 37. Lower Four ──

    #[test]
    fn test_lower_four_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Dot1, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
            ],
            Tile::Character4,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LowerFour);
        assert_not_contains_fan(&result, FanType::NoHonors);
    }

    #[test]
    fn test_lower_four_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Bamboo1, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo2, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo3, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo4, true)),
            ],
            Tile::Character2,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LowerFour);
    }

    // ── 38. Big Three Winds ──

    #[test]
    fn test_big_three_winds_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::East, true)),
                Meld::Pung(Triplet::new(Tile::South, true)),
                Meld::Pung(Triplet::new(Tile::West, true)),
                Meld::Pung(Triplet::new(Tile::Character9, true)),
            ],
            Tile::Bamboo9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigThreeWinds);
    }

    #[test]
    fn test_big_three_winds_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::South, true)),
                Meld::Pung(Triplet::new(Tile::West, true)),
                Meld::Pung(Triplet::new(Tile::North, true)),
                Meld::Chow(Sequence::new(Tile::Character2, true)),
            ],
            Tile::Character5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigThreeWinds);
    }

    #[test]
    fn test_big_three_winds_example3() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::South, true)),
                Meld::Pung(Triplet::new(Tile::West, true)),
                Meld::Pung(Triplet::new(Tile::North, true)),
                Meld::Pung(Triplet::new(Tile::Red, true)),
            ],
            Tile::Green,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigThreeWinds);
    }

    // ═══════════════════════════════════════════════════════════════
    // 8-point fans
    // ═══════════════════════════════════════════════════════════════

    // ── 39. Mixed Straight ──

    #[test]
    fn test_mixed_straight_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
                Meld::Chow(Sequence::new(Tile::Dot4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            ],
            Tile::Dot8,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MixedStraight);
    }

    #[test]
    fn test_mixed_straight_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
                Meld::Chow(Sequence::new(Tile::Dot7, true)),
                Meld::Chow(Sequence::new(Tile::Dot4, true)),
            ],
            Tile::Character7,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MixedStraight);
    }

    // ── 40. Reversible Tiles ──

    #[test]
    fn test_reversible_tiles_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            ],
            Tile::Bamboo5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ReversibleTiles);
    }

    #[test]
    fn test_reversible_tiles_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Dot2, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
                Meld::Pung(Triplet::new(Tile::Dot8, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo8, true)),
            ],
            Tile::White,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ReversibleTiles);
    }

    #[test]
    fn test_reversible_tiles_example3() {
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Dot8, false)),
                Meld::Pung(Triplet::new(Tile::Dot9, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo9, false)),
                Meld::Pung(Triplet::new(Tile::White, false)),
            ],
            Tile::Bamboo8,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ReversibleTiles);
    }

    // ── 41. Mixed Triple Chow ──

    #[test]
    fn test_mixed_triple_chow_example1() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Bamboo3, true)),
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Character3, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
            ],
            Tile::Character4,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MixedTripleChow);
    }

    #[test]
    fn test_mixed_triple_chow_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character6, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo6, true)),
                Meld::Chow(Sequence::new(Tile::Dot6, true)),
                Meld::Chow(Sequence::new(Tile::Dot6, true)),
            ],
            Tile::Bamboo9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MixedTripleChow);
    }

    // ── 42. Mixed Shifted Pungs ──

    #[test]
    fn test_mixed_shifted_pungs_example1() {
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Dot2, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo3, false)),
                Meld::Pung(Triplet::new(Tile::Character4, false)),
                Meld::Pung(Triplet::new(Tile::Dot3, false)),
            ],
            Tile::Character2,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MixedShiftedPungs);
    }

    #[test]
    fn test_mixed_shifted_pungs_example2() {
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Character7, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
                Meld::Pung(Triplet::new(Tile::Dot9, false)),
                Meld::Chow(Sequence::new(Tile::Dot7, false)),
            ],
            Tile::Character9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MixedShiftedPungs);
    }

    // ── 43. Chicken Hand ──

    #[test]
    fn test_chicken_hand() {
        // A hand with only a non-scoring pattern (e.g., mixed chows with honors
        // that don't form any scoring combination).
    }

    // ── 48. Two Concealed Kongs ──

    #[test]
    fn test_two_concealed_kongs() {
        // Kongs must be declared melds; they can be marked concealed (self-drawn).
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Bamboo1, true)),
                Meld::Kong(Quad::new(Tile::East, true)),
                Meld::Pung(Triplet::new(Tile::South, false)),
                Meld::Chow(Sequence::new(Tile::Character7, false)),
            ],
            Tile::Character8,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::TwoConcealedKongs);
    }

    // ── 44. Last Tile Draw ──

    #[test]
    fn test_last_tile_draw() {
        let hand = all_declared_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, false)),
                Meld::Chow(Sequence::new(Tile::Character4, false)),
                Meld::Chow(Sequence::new(Tile::Character7, false)),
                Meld::Chow(Sequence::new(Tile::Character1, false)),
            ],
            Tile::Character5,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.is_last_tile_draw = true;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LastTileDraw);
    }

    // ── 45. Last Tile Claim ──

    #[test]
    fn test_last_tile_claim() {
        let hand = all_declared_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, false)),
                Meld::Chow(Sequence::new(Tile::Character4, false)),
                Meld::Chow(Sequence::new(Tile::Character7, false)),
                Meld::Chow(Sequence::new(Tile::Character1, false)),
            ],
            Tile::Character5,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.is_last_tile_claim = true;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LastTileClaim);
    }

    // ── 46. Out With Replacement Tile ──

    #[test]
    fn test_out_with_replacement_tile() {
        let hand = all_declared_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, false)),
                Meld::Chow(Sequence::new(Tile::Character4, false)),
                Meld::Chow(Sequence::new(Tile::Character7, false)),
                Meld::Chow(Sequence::new(Tile::Character1, false)),
            ],
            Tile::Character5,
        );
        let static_ctx = default_static_ctx();
        let mut dynamic_ctx = default_dynamic_ctx();
        dynamic_ctx.is_kong_replacement = true;
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::OutWithReplacementTile);
    }

    // ── 47. Robbing The Kong ──

    #[test]
    fn test_robbing_the_kong() {
        let hand = all_declared_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, false)),
                Meld::Chow(Sequence::new(Tile::Character4, false)),
                Meld::Chow(Sequence::new(Tile::Character7, false)),
                Meld::Chow(Sequence::new(Tile::Character1, false)),
            ],
            Tile::Character5,
        );
        let static_ctx = default_static_ctx();
        let mut dynamic_ctx = default_dynamic_ctx();
        dynamic_ctx.is_rob_kong = true;
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::RobbingTheKong);
    }

    // ═══════════════════════════════════════════════════════════════
    // 6-point fans
    // ═══════════════════════════════════════════════════════════════

    // ── 49. All Pungs ──

    #[test]
    // Fully concealed 4-pung hand qualifies for Four Concealed Pungs (64 pt)
    // which excludes All Pungs (6 pt).  To test All Pungs in isolation, use
    // exposed (declared) melds so Four Concealed Pungs doesn't apply.
    #[test]
    fn test_all_pungs() {
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Character8, false)),
                Meld::Pung(Triplet::new(Tile::Dot8, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
                Meld::Pung(Triplet::new(Tile::White, false)),
            ],
            Tile::East,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::AllPungs);
    }

    // ── 50. Half Flush ──

    #[test]
    fn test_half_flush() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character2, true)),
                Meld::Chow(Sequence::new(Tile::Character3, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
            ],
            Tile::Red,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::HalfFlush);
    }

    // ── 51. Mixed Shifted Chows ──

    #[test]
    fn test_mixed_shifted_chows() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
                Meld::Chow(Sequence::new(Tile::Character2, true)),
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            ],
            Tile::Character5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MixedShiftedChows);
    }

    // ── 52. All Types ──

    #[test]
    fn test_all_types() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Dot4, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
                Meld::Pung(Triplet::new(Tile::Green, true)),
            ],
            Tile::North,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::AllTypes);
    }

    // ── 53. Melded Hand ──

    #[test]
    fn test_melded_hand() {
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Character2, false)),
                Meld::Chow(Sequence::new(Tile::Dot4, false)),
                Meld::Chow(Sequence::new(Tile::Character6, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
            ],
            Tile::Character6,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.win_method = WinMethod::Discard;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MeldedHand);
    }

    // ── 54. Two Dragon Pungs ──
}
