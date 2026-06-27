// ═══════════════════════════════════════════════════════════════
// 4 / 2 / 1 point rules
// ═══════════════════════════════════════════════════════════════

#![allow(non_snake_case)]

use std::collections::HashSet;

use super::super::fan_context::{FanContext, WaitType, WinMethod};
use super::super::types::{FanCandidate, FanType};
use super::super::view::{
    HandProfile, MeldKind, is_honor_tile, is_terminal_tile, is_wind_tile,
    rank_of,
};
use super::helpers::{cand, is_chow, is_melded_kong, is_pung_or_kong};

// ── 4 points ──────────────────────────────────────────────────

pub(crate) fn outside_hand(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    fn meld_has_terminal_or_honor(m: &super::super::view::MeldInfo) -> bool {
        match m.kind {
            MeldKind::Chow => m.tiles[..3]
                .iter()
                .any(|&t| is_terminal_tile(t) || is_honor_tile(t)),
            _ => is_terminal_tile(m.tile) || is_honor_tile(m.tile),
        }
    }
    if profile.melds[..n]
        .iter()
        .all(|m| meld_has_terminal_or_honor(m))
        && (is_terminal_tile(profile.pair.tile)
            || is_honor_tile(profile.pair.tile))
    {
        vec![cand(FanType::OutsideHand, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const OUTSIDE_HAND_EXCLUDES: &[FanType] = &[];

pub(crate) fn fully_concealed(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.is_fully_concealed {
        vec![cand(FanType::FullyConcealed, 0, true)]
    } else {
        vec![]
    }
}
pub(crate) const FULLY_CONCEALED_EXCLUDES: &[FanType] = &[];

pub(crate) fn two_melded_kongs(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    let n = profile.n_sets as usize;
    let cnt = profile.melds[..n]
        .iter()
        .filter(|m| is_melded_kong(m))
        .count();
    if cnt >= 2 {
        vec![cand(FanType::TwoMeldedKongs, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const TWO_MELDED_KONGS_EXCLUDES: &[FanType] = &[];

pub(crate) fn last_tile(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.is_last_tile_of_kind {
        vec![cand(FanType::LastTile, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const LAST_TILE_EXCLUDES: &[FanType] = &[];

// ── 2 points ──────────────────────────────────────────────────

pub(crate) fn dragon_pung(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let mut mask = 0u64;
    for (i, m) in profile.melds[..n].iter().enumerate() {
        if m.is_dragon && is_pung_or_kong(m) {
            mask |= 1 << i;
        }
    }
    if mask != 0 {
        vec![cand(FanType::DragonPung, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const DRAGON_PUNG_EXCLUDES: &[FanType] = &[];

pub(crate) fn prevalent_wind(
    profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let target = ctx.prevalent_wind_to_tile();
    for (i, m) in profile.melds[..n].iter().enumerate() {
        if matches!(m.kind, MeldKind::Pung) && m.tile == target {
            return vec![cand(FanType::PrevalentWind, 1 << i, false)];
        }
    }
    vec![]
}
pub(crate) const PREVALENT_WIND_EXCLUDES: &[FanType] = &[];

pub(crate) fn seat_wind(
    profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let target = ctx.seat_wind_to_tile();
    for (i, m) in profile.melds[..n].iter().enumerate() {
        if matches!(m.kind, MeldKind::Pung) && m.tile == target {
            return vec![cand(FanType::SeatWind, 1 << i, false)];
        }
    }
    vec![]
}
pub(crate) const SEAT_WIND_EXCLUDES: &[FanType] = &[];

pub(crate) fn concealed_hand(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.is_concealed && ctx.win_method == WinMethod::Discard {
        vec![cand(FanType::ConcealedHand, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const CONCEALED_HAND_EXCLUDES: &[FanType] = &[];

pub(crate) fn all_chows(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if profile.all_chows && profile.n_sets == 4 && !profile.pair.is_honor {
        vec![cand(FanType::AllChows, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const ALL_CHOWS_EXCLUDES: &[FanType] = &[];

pub(crate) fn tile_hog(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    // Count occurrences of each suit tile across all melds + pair
    // 3 suits × 9 ranks = 27 counters
    let mut counts = [0u8; 27];
    for m in &profile.melds[..n] {
        if m.is_honor {
            continue;
        }
        let base = m.suit as usize * 9;
        match m.kind {
            MeldKind::Pung => counts[base + m.rank as usize - 1] += 3,
            MeldKind::Kong => counts[base + m.rank as usize - 1] += 4,
            MeldKind::Chow => {
                // All three tiles in the chow
                for t in &m.tiles[..3] {
                    if !t.is_suit() {
                        continue;
                    }
                    let r = rank_of(*t);
                    counts[base + r as usize - 1] += 1;
                }
            }
        }
    }
    if !profile.pair.is_honor {
        let base = profile.pair.suit as usize * 9;
        counts[base + profile.pair.rank as usize - 1] += 2;
    }
    if counts.iter().any(|&c| c >= 4) {
        vec![cand(FanType::TileHog, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const TILE_HOG_EXCLUDES: &[FanType] = &[];

pub(crate) fn double_pung(
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
    let mut mask = 0u64;
    for i in 0..p.len() {
        for j in i + 1..p.len() {
            if p[i].1 > 0 && p[i].1 == p[j].1 && p[i].2 != p[j].2 {
                mask |= 1 << p[i].0 | 1 << p[j].0;
            }
        }
    }
    if mask != 0 {
        vec![cand(FanType::DoublePung, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const DOUBLE_PUNG_EXCLUDES: &[FanType] = &[];

pub(crate) fn two_concealed_pungs(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    let n = profile.n_sets as usize;
    if profile.melds[..n].iter().filter(|m| m.is_concealed).count() >= 2 {
        vec![cand(FanType::TwoConcealedPungs, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const TWO_CONCEALED_PUNGS_EXCLUDES: &[FanType] = &[];

pub(crate) fn concealed_kong(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let mut mask = 0u64;
    for (i, m) in profile.melds[..n].iter().enumerate() {
        if matches!(m.kind, MeldKind::Kong) && m.is_concealed {
            mask |= 1 << i;
        }
    }
    if mask != 0 {
        vec![cand(FanType::ConcealedKong, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const CONCEALED_KONG_EXCLUDES: &[FanType] = &[];

pub(crate) fn all_simples(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    if profile.melds[..n]
        .iter()
        .all(|m| m.suit < 3 && !m.is_terminal)
        && profile.pair.suit < 3
        && !profile.pair.is_terminal
    {
        vec![cand(FanType::AllSimples, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const ALL_SIMPLES_EXCLUDES: &[FanType] = &[];

// ── 1 point ───────────────────────────────────────────────────

pub(crate) fn pure_double_chow(
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
    let mut mask = 0u64;
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            if chows[i].1 == chows[j].1 && chows[i].2 == chows[j].2 {
                mask |= 1 << chows[i].0 | 1 << chows[j].0;
            }
        }
    }
    if mask != 0 {
        vec![cand(FanType::PureDoubleChow, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const PURE_DOUBLE_CHOW_EXCLUDES: &[FanType] = &[];

pub(crate) fn mixed_double_chow(
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
    let mut mask = 0u64;
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            if chows[i].1 == chows[j].1 && chows[i].2 != chows[j].2 {
                mask |= 1 << chows[i].0 | 1 << chows[j].0;
            }
        }
    }
    if mask != 0 {
        vec![cand(FanType::MixedDoubleChow, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const MIXED_DOUBLE_CHOW_EXCLUDES: &[FanType] = &[];

pub(crate) fn short_straight(
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
    let mut mask = 0u64;
    for i in 0..chows.len() {
        for j in i + 1..chows.len() {
            if chows[i].2 == chows[j].2
                && (chows[i].1 + 3 == chows[j].1
                    || chows[j].1 + 3 == chows[i].1)
            {
                mask |= 1 << chows[i].0 | 1 << chows[j].0;
            }
        }
    }
    if mask != 0 {
        vec![cand(FanType::ShortStraight, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const SHORT_STRAIGHT_EXCLUDES: &[FanType] = &[];

pub(crate) fn two_terminal_chows(
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
    let mut by_suit: [Vec<u8>; 3] = [vec![], vec![], vec![]];
    let mut id_by_suit: [Vec<usize>; 3] = [vec![], vec![], vec![]];
    for &(i, r, s) in &chows {
        if s < 3 {
            by_suit[s as usize].push(r);
            id_by_suit[s as usize].push(i);
        }
    }
    let mut mask = 0u64;
    for s in 0..3 {
        let has_1 = by_suit[s].iter().any(|&r| r == 1);
        let has_7 = by_suit[s].iter().any(|&r| r == 7);
        if has_1 && has_7 {
            for (idx, &r) in by_suit[s].iter().enumerate() {
                if r == 1 || r == 7 {
                    mask |= 1 << id_by_suit[s][idx];
                }
            }
        }
    }
    if mask != 0 {
        vec![cand(FanType::TwoTerminalChows, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const TWO_TERMINAL_CHOWS_EXCLUDES: &[FanType] = &[];

pub(crate) fn pung_of_terminals_or_honors(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let mut mask = 0u64;
    for (i, m) in profile.melds[..n].iter().enumerate() {
        if is_pung_or_kong(m)
            && (is_terminal_tile(m.tile) || is_wind_tile(m.tile))
        {
            mask |= 1 << i;
        }
    }
    if mask != 0 {
        vec![cand(FanType::PungOfTerminalsOrHonors, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const PUNG_OF_TERMINALS_OR_HONORS_EXCLUDES: &[FanType] = &[];

pub(crate) fn melded_kong(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let mut mask = 0u64;
    for (i, m) in profile.melds[..n].iter().enumerate() {
        if is_melded_kong(m) {
            mask |= 1 << i;
        }
    }
    if mask != 0 {
        vec![cand(FanType::MeldedKong, mask, false)]
    } else {
        vec![]
    }
}
pub(crate) const MELDED_KONG_EXCLUDES: &[FanType] = &[];

pub(crate) fn one_voided_suit(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let mut suits = HashSet::new();
    for m in &profile.melds[..n] {
        if m.suit < 3 {
            suits.insert(m.suit);
        }
    }
    if profile.pair.suit < 3 {
        suits.insert(profile.pair.suit);
    }
    if suits.len() <= 2 && !suits.is_empty() {
        vec![cand(FanType::OneVoidedSuit, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const ONE_VOIDED_SUIT_EXCLUDES: &[FanType] = &[];

pub(crate) fn no_honors(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if profile.is_standard() && profile.all_suit_tiles() {
        vec![cand(FanType::NoHonors, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const NO_HONORS_EXCLUDES: &[FanType] = &[];

pub(crate) fn edge_wait(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.wait_type == WaitType::Edge {
        vec![cand(FanType::EdgeWait, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const EDGE_WAIT_EXCLUDES: &[FanType] = &[];

pub(crate) fn closed_wait(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.wait_type == WaitType::Closed {
        vec![cand(FanType::ClosedWait, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const CLOSED_WAIT_EXCLUDES: &[FanType] = &[];

pub(crate) fn single_wait(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.wait_type == WaitType::Single {
        vec![cand(FanType::SingleWait, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const SINGLE_WAIT_EXCLUDES: &[FanType] = &[];

pub(crate) fn self_drawn(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.win_method == WinMethod::SelfDraw {
        vec![cand(FanType::SelfDrawn, 0, true)]
    } else {
        vec![]
    }
}
pub(crate) const SELF_DRAWN_EXCLUDES: &[FanType] = &[];

pub(crate) fn flower_tiles(
    _profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<FanCandidate> {
    if ctx.flower_count > 0 {
        vec![cand(FanType::FlowerTiles, 0, false)]
    } else {
        vec![]
    }
}
pub(crate) const FLOWER_TILES_EXCLUDES: &[FanType] = &[];
