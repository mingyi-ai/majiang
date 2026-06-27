// ═══════════════════════════════════════════════════════════════
// 12 / 8 / 6 point rules
// ═══════════════════════════════════════════════════════════════

#![allow(non_snake_case)]

use std::collections::HashSet;

use super::super::fan_context::{FanContext, WaitType, WinMethod};
use super::super::types::{FanCandidate, FanType};
use super::super::view::{
    HandProfile, MeldKind, ProfileKind, is_reversible_tile, rank_of,
};
use super::helpers::{all_in_range, cand, is_chow, is_pung_or_kong};

// ── 12 points ──────────────────────────────────────────────────

pub(crate) fn lesser_honors_and_knitted_tiles(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::LesserHonorsAndKnittedTiles => {
            vec![cand(FanType::LesserHonorsAndKnittedTiles, 0, true)]
        }
        _ => vec![],
    }
}
pub(crate) const LESSER_HONORS_AND_KNITTED_TILES_EXCLUDES: &[FanType] =
    &[FanType::AllTypes, FanType::ConcealedHand];

pub(crate) fn knitted_straight(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::KnittedStraight => {
            vec![cand(FanType::KnittedStraight, 0, true)]
        }
        _ => vec![],
    }
}
pub(crate) const KNITTED_STRAIGHT_EXCLUDES: &[FanType] = &[];

pub(crate) fn upper_four(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::Standard if all_in_range(profile, 6, 9) => {
            vec![cand(FanType::UpperFour, 0b1111, true)]
        }
        ProfileKind::SevenPairs => {
            if profile.pair_tiles.iter().all(|&t| {
                let r = rank_of(t);
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
pub(crate) const UPPER_FOUR_EXCLUDES: &[FanType] = &[FanType::NoHonors];

pub(crate) fn lower_four(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::Standard if all_in_range(profile, 1, 4) => {
            vec![cand(FanType::LowerFour, 0b1111, true)]
        }
        ProfileKind::SevenPairs => {
            if profile.pair_tiles.iter().all(|&t| {
                let r = rank_of(t);
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
pub(crate) const LOWER_FOUR_EXCLUDES: &[FanType] = &[FanType::NoHonors];

pub(crate) fn big_three_winds(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let cnt = profile.melds[..n]
        .iter()
        .filter(|m| m.is_wind && is_pung_or_kong(m))
        .count();
    if cnt >= 3 {
        vec![cand(FanType::BigThreeWinds, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const BIG_THREE_WINDS_EXCLUDES: &[FanType] = &[];

// ── 8 points ──────────────────────────────────────────────────

pub(crate) fn mixed_straight(
    profile: &HandProfile,
    _ctx: &FanContext,
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
                let mut r = [chows[i].1, chows[j].1, chows[k].1];
                r.sort();
                if r != [1, 4, 7] {
                    continue;
                }
                let suits: HashSet<u8> =
                    [chows[i].2, chows[j].2, chows[k].2].into();
                if suits.len() == 3 {
                    return vec![cand(FanType::MixedStraight, 0b1111, false)];
                }
            }
        }
    }
    vec![]
}
pub(crate) const MIXED_STRAIGHT_EXCLUDES: &[FanType] = &[];

pub(crate) fn reversible_tiles(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::Standard => {
            let n = profile.n_sets as usize;
            let all_rev = profile.melds[..n].iter().all(|m| match m.kind {
                MeldKind::Chow => {
                    m.tiles[..3].iter().all(|&t| is_reversible_tile(t))
                }
                _ => is_reversible_tile(m.tile),
            }) && is_reversible_tile(profile.pair.tile);
            if all_rev {
                vec![cand(FanType::ReversibleTiles, 0b1111, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}
pub(crate) const REVERSIBLE_TILES_EXCLUDES: &[FanType] =
    &[FanType::OneVoidedSuit];

pub(crate) fn mixed_triple_chow(
    profile: &HandProfile,
    _ctx: &FanContext,
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
pub(crate) const MIXED_TRIPLE_CHOW_EXCLUDES: &[FanType] = &[];

pub(crate) fn mixed_shifted_pungs(
    profile: &HandProfile,
    _ctx: &FanContext,
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
pub(crate) const MIXED_SHIFTED_PUNGS_EXCLUDES: &[FanType] = &[];

pub(crate) fn chicken_hand(
    _profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    vec![]
}
pub(crate) const CHICKEN_HAND_EXCLUDES: &[FanType] = &[];

pub(crate) fn last_tile_draw(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.is_last_tile_draw {
        vec![cand(FanType::LastTileDraw, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const LAST_TILE_DRAW_EXCLUDES: &[FanType] = &[];

pub(crate) fn last_tile_claim(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.is_last_tile_claim {
        vec![cand(FanType::LastTileClaim, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const LAST_TILE_CLAIM_EXCLUDES: &[FanType] = &[];

pub(crate) fn out_with_replacement_tile(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.win_method == WinMethod::KongReplacement {
        vec![cand(FanType::OutWithReplacementTile, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const OUT_WITH_REPLACEMENT_TILE_EXCLUDES: &[FanType] = &[];

pub(crate) fn robbing_the_kong(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.win_method == WinMethod::RobKong {
        vec![cand(FanType::RobbingTheKong, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const ROBBING_THE_KONG_EXCLUDES: &[FanType] = &[];

pub(crate) fn two_concealed_kongs(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    let n = profile.n_sets as usize;
    let cnt = profile.melds[..n]
        .iter()
        .filter(|m| matches!(m.kind, MeldKind::Kong) && m.is_concealed)
        .count();
    if cnt >= 2 {
        vec![cand(FanType::TwoConcealedKongs, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const TWO_CONCEALED_KONGS_EXCLUDES: &[FanType] = &[];

// ── 6 points ──────────────────────────────────────────────────

pub(crate) fn all_pungs(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if profile.all_pungs && profile.n_sets == 4 {
        vec![cand(FanType::AllPungs, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const ALL_PUNGS_EXCLUDES: &[FanType] = &[];

pub(crate) fn half_flush(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let mut suits = HashSet::new();
    for m in &profile.melds[..n] {
        if !m.is_honor {
            suits.insert(m.suit);
        }
    }
    if !profile.pair.is_honor {
        suits.insert(profile.pair.suit);
    }
    if suits.len() == 1 && profile.has_honors {
        vec![cand(FanType::HalfFlush, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const HALF_FLUSH_EXCLUDES: &[FanType] = &[];

pub(crate) fn mixed_shifted_chows(
    profile: &HandProfile,
    _ctx: &FanContext,
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
pub(crate) const MIXED_SHIFTED_CHOWS_EXCLUDES: &[FanType] = &[];

pub(crate) fn all_types(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let mut mask: u8 = 0;
    for m in &profile.melds[..n] {
        mask |= 1 << m.suit.min(2); // map suit 0-2 to bits 0-2; honor winds/dragons go to bit 3
    }
    if profile.pair.is_honor {
        mask |= 1 << 3; // honors
    } else {
        mask |= 1 << profile.pair.suit;
    }
    // All 5 types (3 suits + winds + dragons) need 5 bits
    // We track: bit 0=char, 1=dot, 2=bamboo, 3=winds, 4=dragons
    let mut full_mask: u16 = 0;
    for m in &profile.melds[..n] {
        if m.is_dragon {
            full_mask |= 1 << 4;
        } else if m.is_wind {
            full_mask |= 1 << 3;
        } else if m.suit < 3 {
            full_mask |= 1 << m.suit;
        }
    }
    if profile.pair.is_dragon {
        full_mask |= 1 << 4;
    } else if profile.pair.is_wind {
        full_mask |= 1 << 3;
    } else if profile.pair.suit < 3 {
        full_mask |= 1 << profile.pair.suit;
    }
    if full_mask == 0b11111 {
        vec![cand(FanType::AllTypes, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const ALL_TYPES_EXCLUDES: &[FanType] = &[];

pub(crate) fn melded_hand(
    profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let all_exposed = profile.melds[..n].iter().all(|m| !m.is_concealed);
    if all_exposed
        && ctx.win_method == WinMethod::Discard
        && ctx.wait_type == WaitType::Single
    {
        vec![cand(FanType::MeldedHand, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const MELDED_HAND_EXCLUDES: &[FanType] = &[FanType::SingleWait];

pub(crate) fn two_dragon_pungs(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    let n = profile.n_sets as usize;
    let cnt = profile.melds[..n]
        .iter()
        .filter(|m| m.is_dragon && is_pung_or_kong(m))
        .count();
    if cnt >= 2 {
        vec![cand(FanType::TwoDragonPungs, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const TWO_DRAGON_PUNGS_EXCLUDES: &[FanType] = &[];
