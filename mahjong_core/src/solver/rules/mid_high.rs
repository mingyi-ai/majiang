// ═══════════════════════════════════════════════════════════════
// 32 / 24 / 16 point rules
// ═══════════════════════════════════════════════════════════════

#![allow(non_snake_case)]

use std::collections::HashSet;

use super::{FanCandidate, FanType};
use super::super::{DynamicFanContext, StaticFanContext, WaitType};
use super::profile::{
    HandProfile, MeldKind, ProfileKind, is_even_rank, is_honor_tile,
    is_terminal_tile, rank_of,
};
use super::helpers::{all_in_range, cand, is_chow, is_pung_or_kong};
use crate::structs::Tile;

// ── 32 points ──────────────────────────────────────────────────

pub(crate) fn four_shifted_chows(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let chows: Vec<(usize, u8, u8)> = profile.melds[..n]
        .iter()
        .enumerate()
        .filter(|(_, m)| is_chow(m))
        .map(|(i, m)| (i, m.rank, m.suit))
        .collect();
    if chows.len() < 4 {
        return vec![];
    }
    for a in 0..chows.len() {
        for b in a + 1..chows.len() {
            for c in b + 1..chows.len() {
                for d in c + 1..chows.len() {
                    let selected = [chows[a], chows[b], chows[c], chows[d]];
                    if !selected.iter().all(|x| x.2 == chows[a].2) {
                        continue;
                    }
                    let mut ranks: Vec<u8> =
                        selected.iter().map(|x| x.1).collect();
                    ranks.sort();
                    let shift1 = ranks[1] - ranks[0];
                    let shift2 = ranks[2] - ranks[1];
                    let shift3 = ranks[3] - ranks[2];
                    if (shift1 == 1 || shift1 == 2)
                        && shift1 == shift2
                        && shift2 == shift3
                    {
                        return vec![cand(
                            FanType::FourShiftedChows,
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
pub(crate) const FOUR_SHIFTED_CHOWS_EXCLUDES: &[FanType] =
    &[FanType::ShortStraight];

pub(crate) fn three_kongs(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if profile.n_kongs >= 3 {
        vec![cand(FanType::ThreeKongs, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const THREE_KONGS_EXCLUDES: &[FanType] = &[];

pub(crate) fn all_terminals_and_honors(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let f = |t: Tile| is_terminal_tile(t) || is_honor_tile(t);
    if profile.melds[..n].iter().all(|m| f(m.tile)) && f(profile.pair.tile) {
        vec![cand(FanType::AllTerminalsAndHonors, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const ALL_TERMINALS_AND_HONORS_EXCLUDES: &[FanType] =
    &[FanType::AllPungs, FanType::PungOfTerminalsOrHonors];

// ── 24 points ──────────────────────────────────────────────────

pub(crate) fn seven_pairs(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::SevenPairs => vec![cand(FanType::SevenPairs, 0, true)],
        _ => vec![],
    }
}
pub(crate) const SEVEN_PAIRS_EXCLUDES: &[FanType] =
    &[FanType::ConcealedHand, FanType::SingleWait];

pub(crate) fn all_even_pungs(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    if profile.melds[..n]
        .iter()
        .all(|m| is_pung_or_kong(m) && is_even_rank(m.tile))
        && is_even_rank(profile.pair.tile)
    {
        vec![cand(FanType::AllEvenPungs, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const ALL_EVEN_PUNGS_EXCLUDES: &[FanType] =
    &[FanType::AllPungs, FanType::AllSimples];

pub(crate) fn full_flush(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let s = profile.melds[0].suit;
    if s >= 3 {
        return vec![];
    }
    let n = profile.n_sets as usize;
    if profile.melds[..n].iter().all(|m| m.suit == s) && profile.pair.suit == s
    {
        vec![cand(FanType::FullFlush, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const FULL_FLUSH_EXCLUDES: &[FanType] = &[FanType::NoHonors];

pub(crate) fn pure_triple_chow(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let chows: Vec<(usize, u8, u8)> = profile.melds[..n]
        .iter()
        .enumerate()
        .filter(|(_, m)| is_chow(m))
        .map(|(i, m)| (i, m.rank, m.suit))
        .collect();
    if chows.len() < 3 {
        return vec![];
    }
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            for k in j + 1..chows.len() {
                if chows[i].1 == chows[j].1
                    && chows[i].1 == chows[k].1
                    && chows[i].2 == chows[j].2
                    && chows[i].2 == chows[k].2
                {
                    return vec![cand(FanType::PureTripleChow, 0b1111, false)];
                }
            }
        }
    }
    vec![]
}
pub(crate) const PURE_TRIPLE_CHOW_EXCLUDES: &[FanType] =
    &[FanType::PureShiftedPungs, FanType::PureDoubleChow];

pub(crate) fn pure_shifted_pungs(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let p: Vec<(usize, u8, u8)> = profile.melds[..n]
        .iter()
        .enumerate()
        .filter(|(_, m)| is_pung_or_kong(m))
        .map(|(i, m)| (i, m.rank, m.suit))
        .collect();
    if p.len() < 3 {
        return vec![];
    }
    for i in 0..p.len() {
        for j in i + 1..p.len() {
            for k in j + 1..p.len() {
                let mut ranks = [p[i].1, p[j].1, p[k].1];
                ranks.sort();
                if p[i].2 == p[j].2
                    && p[i].2 == p[k].2
                    && ranks[1] == ranks[0] + 1
                    && ranks[2] == ranks[1] + 1
                {
                    return vec![cand(
                        FanType::PureShiftedPungs,
                        0b1111,
                        false,
                    )];
                }
            }
        }
    }
    vec![]
}
pub(crate) const PURE_SHIFTED_PUNGS_EXCLUDES: &[FanType] =
    &[FanType::PureTripleChow];

pub(crate) fn upper_tiles(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::Standard if all_in_range(profile, 7, 9) => {
            vec![cand(FanType::UpperTiles, 0b1111, true)]
        }
        ProfileKind::SevenPairs => {
            if profile.pair_tiles.iter().all(|&t| {
                let r = rank_of(t);
                (7..=9).contains(&r)
            }) {
                vec![cand(FanType::UpperTiles, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}
pub(crate) const UPPER_TILES_EXCLUDES: &[FanType] = &[FanType::NoHonors];

pub(crate) fn middle_tiles(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::Standard if all_in_range(profile, 4, 6) => {
            vec![cand(FanType::MiddleTiles, 0b1111, true)]
        }
        ProfileKind::SevenPairs => {
            if profile.pair_tiles.iter().all(|&t| {
                let r = rank_of(t);
                (4..=6).contains(&r)
            }) {
                vec![cand(FanType::MiddleTiles, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}
pub(crate) const MIDDLE_TILES_EXCLUDES: &[FanType] =
    &[FanType::NoHonors, FanType::AllSimples];

pub(crate) fn lower_tiles(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::Standard if all_in_range(profile, 1, 3) => {
            vec![cand(FanType::LowerTiles, 0b1111, true)]
        }
        ProfileKind::SevenPairs => {
            if profile.pair_tiles.iter().all(|&t| {
                let r = rank_of(t);
                (1..=3).contains(&r)
            }) {
                vec![cand(FanType::LowerTiles, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}
pub(crate) const LOWER_TILES_EXCLUDES: &[FanType] = &[FanType::NoHonors];

// ── 16 points ──────────────────────────────────────────────────

pub(crate) fn pure_straight(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let chows: Vec<(usize, u8, u8)> = profile.melds[..n]
        .iter()
        .enumerate()
        .filter(|(_, m)| is_chow(m))
        .map(|(i, m)| (i, m.rank, m.suit))
        .collect();
    if chows.len() < 3 {
        return vec![];
    }
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            for k in j + 1..chows.len() {
                let s = chows[i].2;
                if chows[j].2 != s || chows[k].2 != s {
                    continue;
                }
                let mut r = [chows[i].1, chows[j].1, chows[k].1];
                r.sort();
                if r == [1, 4, 7] {
                    let mask = (1 << chows[i].0)
                        | (1 << chows[j].0)
                        | (1 << chows[k].0);
                    return vec![cand(FanType::PureStraight, mask, false)];
                }
            }
        }
    }
    vec![]
}
pub(crate) const PURE_STRAIGHT_EXCLUDES: &[FanType] = &[];

pub(crate) fn three_suited_terminal_chows(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() || profile.n_sets != 4 {
        return vec![];
    }
    if profile.pair.is_honor || profile.pair.rank != 5 {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let chows: Vec<(usize, u8, u8)> = profile.melds[..n]
        .iter()
        .enumerate()
        .filter(|(_, m)| is_chow(m))
        .map(|(i, m)| (i, m.rank, m.suit))
        .collect();
    if chows.len() != 4 {
        return vec![];
    }
    let mut suits_found = HashSet::new();
    for (_, _, s) in &chows {
        suits_found.insert(*s);
    }
    if suits_found.len() != 2 {
        return vec![];
    }
    let s_list: Vec<u8> = suits_found.into_iter().collect();
    for &s in &s_list {
        let has_1 = chows.iter().any(|&(_, r, ss)| ss == s && r == 1);
        let has_7 = chows.iter().any(|&(_, r, ss)| ss == s && r == 7);
        if !has_1 || !has_7 {
            return vec![];
        }
    }
    if s_list.contains(&profile.pair.suit) {
        return vec![];
    }
    vec![cand(FanType::ThreeSuitedTerminalChows, 0b1111, true)]
}
pub(crate) const THREE_SUITED_TERMINAL_CHOWS_EXCLUDES: &[FanType] = &[
    FanType::PureDoubleChow,
    FanType::TwoTerminalChows,
    FanType::NoHonors,
    FanType::AllChows,
];

pub(crate) fn pure_shifted_chows(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let chows: Vec<(usize, u8, u8)> = profile.melds[..n]
        .iter()
        .enumerate()
        .filter(|(_, m)| is_chow(m))
        .map(|(i, m)| (i, m.rank, m.suit))
        .collect();
    if chows.len() < 3 {
        return vec![];
    }
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            for k in j + 1..chows.len() {
                let selected = [chows[i], chows[j], chows[k]];
                if !selected.iter().all(|x| x.2 == chows[i].2) {
                    continue;
                }
                let mut ranks: Vec<u8> =
                    selected.iter().map(|x| x.1).collect();
                ranks.sort();
                let s1 = ranks[1] - ranks[0];
                let s2 = ranks[2] - ranks[1];
                if (s1 == 1 || s1 == 2) && s1 == s2 {
                    return vec![cand(
                        FanType::PureShiftedChows,
                        0b1111,
                        false,
                    )];
                }
            }
        }
    }
    vec![]
}
pub(crate) const PURE_SHIFTED_CHOWS_EXCLUDES: &[FanType] = &[];

pub(crate) fn all_fives(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let f = |r: u8| r == 5;
    let ok = profile.melds[..n].iter().all(|m| match m.kind {
        MeldKind::Chow => m.tiles[..3].iter().any(|&t| rank_of(t) == 5),
        _ => f(m.rank),
    }) && f(profile.pair.rank);
    if ok {
        vec![cand(FanType::AllFives, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const ALL_FIVES_EXCLUDES: &[FanType] = &[FanType::AllSimples];

pub(crate) fn triple_pung(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let p: Vec<(usize, Tile, u8, u8)> = profile.melds[..n]
        .iter()
        .enumerate()
        .filter(|(_, m)| is_pung_or_kong(m))
        .map(|(i, m)| (i, m.tile, m.rank, m.suit))
        .collect();
    for i in 0..p.len() {
        for j in i + 1..p.len() {
            for k in j + 1..p.len() {
                let r = p[i].2;
                if p[j].2 == r && p[k].2 == r {
                    let suits: HashSet<u8> = [p[i].3, p[j].3, p[k].3].into();
                    if suits.len() == 3 {
                        return vec![cand(FanType::TriplePung, 0b1111, false)];
                    }
                }
            }
        }
    }
    vec![]
}
pub(crate) const TRIPLE_PUNG_EXCLUDES: &[FanType] = &[];

pub(crate) fn three_concealed_pungs(
    profile: &HandProfile,
    _static_ctx: &StaticFanContext, _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let n = profile.n_sets as usize;
    let cnt = profile.melds[..n]
        .iter()
        .filter(|m| is_pung_or_kong(m) && m.is_concealed)
        .count();
    if cnt >= 3 {
        vec![cand(FanType::ThreeConcealedPungs, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const THREE_CONCEALED_PUNGS_EXCLUDES: &[FanType] = &[];

// ═══════════════════════════════════════════════════════════════
// Tests — sourced from the official MCR rulebook
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::super::test_helpers::*;
    use super::super::FanType;
    use crate::structs::{Hand, Meld, Quad, Sequence, Tile, Triplet, Wind};
    use crate::solver::{DynamicFanContext, StaticFanContext, WaitType, WinMethod, solve_fan};

#[test]
fn test_four_pure_shifted_pungs() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Dot1, false)),
            Meld::Pung(Triplet::new(Tile::Dot2, false)),
            Meld::Pung(Triplet::new(Tile::Dot3, false)),
            Meld::Pung(Triplet::new(Tile::Dot4, false)),
        ],
        Tile::Bamboo2,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::FourPureShiftedPungs);
}

// ═══════════════════════════════════════════════════════════════
// 32-point fans
// ═══════════════════════════════════════════════════════════════

// ── 16. Four Shifted Chows ──

#[test]
fn test_four_shifted_chows_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
        ],
        Tile::Character1,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::FourShiftedChows);
}

#[test]
fn test_four_shifted_chows_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo5, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
        ],
        Tile::Dot5,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::FourShiftedChows);
}

// ── 17. Three Kongs ──

#[test]
fn test_three_kongs() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Character2, false)),
            Meld::Kong(Quad::new(Tile::Character3, false)),
            Meld::Kong(Quad::new(Tile::Character4, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo2, false)),
        ],
        Tile::Bamboo3,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::ThreeKongs);
}

// ── 18. All Terminals and Honors ──

#[test]
fn test_all_terminals_and_honors() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, false)),
            Meld::Pung(Triplet::new(Tile::Character1, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo9, false)),
            Meld::Pung(Triplet::new(Tile::Dot9, false)),
        ],
        Tile::Red,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::AllTerminalsAndHonors);
    assert_not_contains_fan(&result, FanType::PungOfTerminalsOrHonors);
}

// ═══════════════════════════════════════════════════════════════
// 24-point fans
// ═══════════════════════════════════════════════════════════════

// ── 19. Seven Pairs ──

#[test]
fn test_seven_pairs_example1() {
    let hand = seven_pairs_hand(&[
        Tile::Dot7,
        Tile::Bamboo2,
        Tile::Character3,
        Tile::Red,
        Tile::White,
        Tile::East,
        Tile::North,
    ]);
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::SevenPairs);
}

#[test]
fn test_seven_pairs_example2() {
    // 4× Dot9 forms two pairs of Dot9 (per MCR seven pairs rules).
    // seven_pairs_hand doubles each entry, so list Dot9 once → 2 tiles.
    // We need Dot9 × 4 = two pairs. Manually insert 4 Dot9 tiles.
    let mut hand = Hand::default();
    for &t in &[
        Tile::Dot1,
        Tile::Character1,
        Tile::Bamboo1,
        Tile::Dot9,
        Tile::Dot9,
        Tile::Character9,
        Tile::Bamboo9,
    ] {
        hand.concealed.insert(t);
        hand.concealed.insert(t);
    }
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::SevenPairs);
    // Also qualifies for All Terminals
    assert_contains_fan(&result, FanType::AllTerminals);
}

#[test]
fn test_seven_pairs_example3() {
    let hand = seven_pairs_hand(&[
        Tile::East,
        Tile::South,
        Tile::West,
        Tile::North,
        Tile::Red,
        Tile::Green,
        Tile::White,
    ]);
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::SevenPairs);
}

// ── 20. Greater Honors and Knitted Tiles ──

#[test]
fn test_greater_honors_and_knitted_tiles() {
    // This special decomposition is constructed by decompose_special.
    // For now, use a standard hand and expect the rule to fire through
    // the decomposition engine when it supports this pattern.
    // (Hand construction for this pattern not yet ergonomic.)
}

// ── 21. All Even Pungs ──

#[test]
fn test_all_even_pungs_example1() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character2, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, false)),
            Meld::Pung(Triplet::new(Tile::Dot6, false)),
            Meld::Pung(Triplet::new(Tile::Dot8, false)),
        ],
        Tile::Dot4,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::AllEvenPungs);
}

#[test]
fn test_all_even_pungs_example2() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character2, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo2, false)),
            Meld::Pung(Triplet::new(Tile::Dot2, false)),
            Meld::Pung(Triplet::new(Tile::Character4, false)),
        ],
        Tile::Bamboo4,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::AllEvenPungs);
}

#[test]
fn test_all_even_pungs_example3() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character6, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo6, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
            Meld::Pung(Triplet::new(Tile::Character8, false)),
        ],
        Tile::Bamboo2,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::AllEvenPungs);
}

// ── 22. Full Flush ──

#[test]
fn test_full_flush_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character6, true)),
            Meld::Pung(Triplet::new(Tile::Character7, true)),
            Meld::Pung(Triplet::new(Tile::Character8, true)),
            Meld::Pung(Triplet::new(Tile::Character9, true)),
        ],
        Tile::Character5,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::FullFlush);
    assert_not_contains_fan(&result, FanType::NoHonors);
}

#[test]
fn test_full_flush_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
        ],
        Tile::Dot9,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::FullFlush);
}

// ── 23. Pure Triple Chow ──

#[test]
fn test_pure_triple_chow() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
        ],
        Tile::Dot5,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::PureTripleChow);
}

// ── 24. Pure Shifted Pungs ──

#[test]
fn test_pure_shifted_pungs() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo2, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo3, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, false)),
            Meld::Pung(Triplet::new(Tile::Dot3, false)),
        ],
        Tile::Dot2,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::PureShiftedPungs);
}

// ── 25. Upper Tiles ──

#[test]
fn test_upper_tiles_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
        ],
        Tile::Dot9,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::UpperTiles);
    assert_not_contains_fan(&result, FanType::NoHonors);
}

#[test]
fn test_upper_tiles_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo7, true)),
            Meld::Pung(Triplet::new(Tile::Character8, true)),
            Meld::Pung(Triplet::new(Tile::Dot7, true)),
            Meld::Pung(Triplet::new(Tile::Character9, true)),
        ],
        Tile::Bamboo8,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::UpperTiles);
}

#[test]
fn test_upper_tiles_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Character8,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::UpperTiles);
}

// ── 26. Middle Tiles ──

#[test]
fn test_middle_tiles_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo5, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo6, true)),
        ],
        Tile::Dot6,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::MiddleTiles);
    assert_not_contains_fan(&result, FanType::NoHonors);
    assert_not_contains_fan(&result, FanType::AllSimples);
}

#[test]
fn test_middle_tiles_example2() {
    // Seven pairs of 4-5-6 across suits (7 distinct tiles all rank 4-6)
    let hand = seven_pairs_hand(&[
        Tile::Character4,
        Tile::Character5,
        Tile::Character6,
        Tile::Dot4,
        Tile::Dot5,
        Tile::Dot6,
        Tile::Bamboo4,
    ]);
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::MiddleTiles);
}

// ── 27. Lower Tiles ──

#[test]
fn test_lower_tiles() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
        ],
        Tile::Bamboo3,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::LowerTiles);
    assert_not_contains_fan(&result, FanType::NoHonors);
}

// ═══════════════════════════════════════════════════════════════
// 16-point fans
// ═══════════════════════════════════════════════════════════════

// ── 28. Pure Straight ──
//
// NOTE: Current implementation requires EXACTLY 3 chows in the
// decomposition; hands with 4 chows are missed.

#[test]
fn test_pure_straight_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
        ],
        Tile::Character7,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::PureStraight);
}

#[test]
fn test_pure_straight_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Pung(Triplet::new(Tile::Dot6, true)),
        ],
        Tile::Dot8,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::PureStraight);
}

// ── 29. Three-Suited Terminal Chows ──

#[test]
fn test_three_suited_terminal_chows() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Bamboo5,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::ThreeSuitedTerminalChows);
}

// ── 30. Pure Shifted Chows ──

#[test]
fn test_pure_shifted_chows_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Dot2, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo2, true)),
        ],
        Tile::Bamboo2,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::PureShiftedChows);
}

#[test]
fn test_pure_shifted_chows_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo5, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
        ],
        Tile::Bamboo2,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::PureShiftedChows);
}

#[test]
fn test_pure_shifted_chows_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Chow(Sequence::new(Tile::Character5, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Bamboo7,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::PureShiftedChows);
}

// ── 31. All Fives ──

#[test]
fn test_all_fives_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
        ],
        Tile::Character5,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::AllFives);
}

#[test]
fn test_all_fives_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Pung(Triplet::new(Tile::Dot5, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo5, true)),
        ],
        Tile::Character5,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::AllFives);
}

// ── 32. Triple Pung ──

#[test]
fn test_triple_pung_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Dot3, true)),
            Meld::Pung(Triplet::new(Tile::Character3, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo2, true)),
        ],
        Tile::Character4,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::TriplePung);
}

#[test]
fn test_triple_pung_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character1, true)),
            Meld::Pung(Triplet::new(Tile::Dot1, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo1, true)),
            Meld::Pung(Triplet::new(Tile::Dot9, true)),
        ],
        Tile::Bamboo9,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::TriplePung);
}

// ── 33. Three Concealed Pungs ──

#[test]
fn test_three_concealed_pungs_example1() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::Character9, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo1, false)),
        ],
        Tile::Dot9,
    );
    let results = solve_default(&hand);
    let result = results.first().unwrap();
    assert_contains_fan(&result, FanType::ThreeConcealedPungs);
}

}
