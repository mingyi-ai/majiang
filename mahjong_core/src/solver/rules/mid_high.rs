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
