// ═══════════════════════════════════════════════════════════════
// 88 / 64 / 48 point rules
// ═══════════════════════════════════════════════════════════════

#![allow(non_snake_case)]

use super::super::fan_context::FanContext;
use super::super::types::{FanCandidate, FanType};
use super::super::view::{
    HandProfile, MeldKind, ProfileKind, is_green_tile, is_terminal_tile,
    rank_of, suit_of,
};
use super::helpers::{cand, is_chow, is_pung_or_kong};
use crate::structs::Tile;

// ── 88 points ──────────────────────────────────────────────────

pub(crate) fn big_four_winds(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() || profile.n_sets != 4 {
        return vec![];
    }
    let n = profile.n_sets as usize;
    if profile.melds[..n].iter().all(is_pung_or_kong) {
        let winds = [Tile::East, Tile::South, Tile::West, Tile::North];
        let tiles: Vec<Tile> =
            profile.melds[..n].iter().map(|m| m.tile).collect();
        if winds.iter().all(|w| tiles.contains(w)) {
            return vec![cand(FanType::BigFourWinds, 0b1111, false)];
        }
    }
    vec![]
}
pub(crate) const BIG_FOUR_WINDS_EXCLUDES: &[FanType] = &[
    FanType::BigThreeWinds,
    FanType::AllPungs,
    FanType::PrevalentWind,
    FanType::SeatWind,
    FanType::PungOfTerminalsOrHonors,
];

pub(crate) fn big_three_dragons(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let found: Vec<Tile> = profile.melds[..n]
        .iter()
        .filter(|m| m.is_dragon && is_pung_or_kong(m))
        .map(|m| m.tile)
        .collect();
    if found.len() >= 3 {
        return vec![cand(FanType::BigThreeDragons, 0b1111, false)];
    }
    vec![]
}
pub(crate) const BIG_THREE_DRAGONS_EXCLUDES: &[FanType] =
    &[FanType::TwoDragonPungs, FanType::DragonPung];

pub(crate) fn all_green(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::Standard => {
            let n = profile.n_sets as usize;
            let all_green = profile.melds[..n].iter().all(|m| match m.kind {
                MeldKind::Chow => {
                    m.tiles[..3].iter().all(|&t| is_green_tile(t))
                }
                _ => is_green_tile(m.tile),
            }) && is_green_tile(profile.pair.tile);
            if all_green {
                vec![cand(FanType::AllGreen, 0b1111, true)]
            } else {
                vec![]
            }
        }
        ProfileKind::SevenPairs => {
            if profile.pair_tiles.iter().all(|&t| is_green_tile(t)) {
                vec![cand(FanType::AllGreen, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}
pub(crate) const ALL_GREEN_EXCLUDES: &[FanType] = &[];

pub(crate) fn nine_gates(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let s = profile.melds[0].suit;
    if s >= 3 {
        return vec![];
    }
    if !profile.melds[..n]
        .iter()
        .all(|m| m.suit == s && !m.is_honor)
    {
        return vec![];
    }
    if profile.pair.suit != s || profile.pair.is_honor {
        return vec![];
    }

    let mut counts = [0u8; 9];
    for m in &profile.melds[..n] {
        match m.kind {
            MeldKind::Pung => counts[m.rank as usize - 1] += 3,
            MeldKind::Kong => counts[m.rank as usize - 1] += 4,
            MeldKind::Chow => {
                let r = m.rank as usize - 1;
                counts[r] += 1;
                counts[r + 1] += 1;
                counts[r + 2] += 1;
            }
        }
    }
    counts[profile.pair.rank as usize - 1] += 2;

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
            return vec![cand(FanType::NineGates, 0b1111, true)];
        }
    }
    vec![]
}
pub(crate) const NINE_GATES_EXCLUDES: &[FanType] = &[
    FanType::FullFlush,
    FanType::ConcealedHand,
    FanType::PungOfTerminalsOrHonors,
];

pub(crate) fn four_kongs(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    if profile.n_kongs == 4 {
        return vec![cand(FanType::FourKongs, 0b1111, false)];
    }
    vec![]
}
pub(crate) const FOUR_KONGS_EXCLUDES: &[FanType] = &[FanType::SingleWait];

pub(crate) fn seven_shifted_pairs(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::SevenPairs => {
            let s = suit_of(profile.pair_tiles[0]);
            if profile.pair_tiles.iter().all(|&t| suit_of(t) == s) {
                let mut ranks: Vec<u8> =
                    profile.pair_tiles.iter().map(|&t| rank_of(t)).collect();
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
pub(crate) const SEVEN_SHIFTED_PAIRS_EXCLUDES: &[FanType] = &[
    FanType::FullFlush,
    FanType::ConcealedHand,
    FanType::SingleWait,
];

pub(crate) fn thirteen_orphans(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if profile.kind == ProfileKind::ThirteenOrphans {
        vec![cand(FanType::ThirteenOrphans, 0, true)]
    } else {
        vec![]
    }
}
pub(crate) const THIRTEEN_ORPHANS_EXCLUDES: &[FanType] = &[
    FanType::AllTypes,
    FanType::ConcealedHand,
    FanType::SingleWait,
];

// ── 64 points ──────────────────────────────────────────────────

pub(crate) fn all_terminals(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    match profile.kind {
        ProfileKind::Standard => {
            // Check meld_tile (start tile for chows), matching old code's
            // is_terminal(meld_tile(*m)) behavior.
            let n = profile.n_sets as usize;
            if profile.melds[..n].iter().all(|m| is_terminal_tile(m.tile))
                && is_terminal_tile(profile.pair.tile)
            {
                vec![cand(FanType::AllTerminals, 0b1111, true)]
            } else {
                vec![]
            }
        }
        ProfileKind::SevenPairs => {
            if profile.pair_tiles.iter().all(|&t| is_terminal_tile(t)) {
                vec![cand(FanType::AllTerminals, 0, true)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}
pub(crate) const ALL_TERMINALS_EXCLUDES: &[FanType] = &[
    FanType::AllPungs,
    FanType::OutsideHand,
    FanType::PungOfTerminalsOrHonors,
    FanType::NoHonors,
];

pub(crate) fn little_four_winds(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    if !profile.pair.is_wind {
        return vec![];
    }
    let wind_pungs: Vec<Tile> = profile.melds[..n]
        .iter()
        .filter(|m| m.is_wind && is_pung_or_kong(m))
        .map(|m| m.tile)
        .collect();
    if wind_pungs.len() < 3 {
        return vec![];
    }
    let all = [Tile::East, Tile::South, Tile::West, Tile::North];
    let missing: Vec<Tile> = all
        .iter()
        .filter(|w| !wind_pungs.contains(w) && **w != profile.pair.tile)
        .copied()
        .collect();
    if missing.is_empty() {
        vec![cand(FanType::LittleFourWinds, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const LITTLE_FOUR_WINDS_EXCLUDES: &[FanType] =
    &[FanType::BigThreeWinds, FanType::PungOfTerminalsOrHonors];

pub(crate) fn little_three_dragons(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    let dp: Vec<Tile> = profile.melds[..n]
        .iter()
        .filter(|m| m.is_dragon && is_pung_or_kong(m))
        .map(|m| m.tile)
        .collect();
    if dp.len() == 2 && profile.pair.is_dragon {
        let all = [Tile::Red, Tile::Green, Tile::White];
        let missing: Vec<Tile> = all
            .iter()
            .filter(|d| !dp.contains(d) && **d != profile.pair.tile)
            .copied()
            .collect();
        if missing.is_empty() {
            return vec![cand(FanType::LittleThreeDragons, 0b1111, true)];
        }
    }
    vec![]
}
pub(crate) const LITTLE_THREE_DRAGONS_EXCLUDES: &[FanType] =
    &[FanType::DragonPung, FanType::TwoDragonPungs];

pub(crate) fn all_honors(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() {
        return vec![];
    }
    let n = profile.n_sets as usize;
    if profile.melds[..n].iter().all(|m| m.is_honor) && profile.pair.is_honor {
        vec![cand(FanType::AllHonors, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const ALL_HONORS_EXCLUDES: &[FanType] = &[
    FanType::AllPungs,
    FanType::OutsideHand,
    FanType::PungOfTerminalsOrHonors,
];

pub(crate) fn four_concealed_pungs(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() || profile.n_sets != 4 {
        return vec![];
    }
    let n = profile.n_sets as usize;
    if profile.melds[..n]
        .iter()
        .all(|m| is_pung_or_kong(m) && m.is_concealed)
    {
        vec![cand(FanType::FourConcealedPungs, 0b1111, false)]
    } else {
        vec![]
    }
}
pub(crate) const FOUR_CONCEALED_PUNGS_EXCLUDES: &[FanType] =
    &[FanType::AllPungs, FanType::ConcealedHand];

pub(crate) fn pure_terminal_chows(
    profile: &HandProfile,
    _ctx: &FanContext,
) -> Vec<FanCandidate> {
    if !profile.is_standard() || profile.n_sets != 4 {
        return vec![];
    }
    if profile.pair.is_honor || profile.pair.rank != 5 {
        return vec![];
    }
    let s = profile.melds[0].suit;
    if profile.pair.suit != s {
        return vec![];
    }
    let n = profile.n_sets as usize;
    if !profile.melds[..n].iter().all(|m| is_chow(m) && m.suit == s) {
        return vec![];
    }
    let r1 = profile.melds[..n].iter().filter(|m| m.rank == 1).count() as u8;
    let r7 = profile.melds[..n].iter().filter(|m| m.rank == 7).count() as u8;
    if r1 == 2 && r7 == 2 {
        vec![cand(FanType::PureTerminalChows, 0b1111, true)]
    } else {
        vec![]
    }
}
pub(crate) const PURE_TERMINAL_CHOWS_EXCLUDES: &[FanType] = &[
    FanType::SevenPairs,
    FanType::FullFlush,
    FanType::AllChows,
    FanType::PureDoubleChow,
    FanType::TwoTerminalChows,
];

// ── 48 points ──────────────────────────────────────────────────

pub(crate) fn quadruple_chow(
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
pub(crate) const QUADRUPLE_CHOW_EXCLUDES: &[FanType] = &[
    FanType::PureShiftedPungs,
    FanType::TileHog,
    FanType::PureDoubleChow,
];

pub(crate) fn four_pure_shifted_pungs(
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
pub(crate) const FOUR_PURE_SHIFTED_PUNGS_EXCLUDES: &[FanType] =
    &[FanType::PureTripleChow, FanType::AllPungs];
