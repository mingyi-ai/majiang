// ═══════════════════════════════════════════════════════════════
// MCR Fan Rule Registry
// ═══════════════════════════════════════════════════════════════
//
// All 81 rules are inline `mod` blocks, ordered as in the rulebook
// (descending by point value).  The `define_rules!` macro generates
// the `RuleKind` enum + match-arm dispatch + `ALL_RULE_KINDS` slice.
//
// Each rule module exports:
//   pub(crate) fn check(decomp, ctx) -> Vec<FanCandidate>
//   pub(crate) const EXCLUDES: &[FanType]
//
// Adding a rule: add `mod name { ... }` block, add `name,` to
// `define_rules!`.  Three minutes of work.

use super::decomposition::Decomposition;
use super::fan_context::{FanContext, WaitType, WinMethod};
use super::types::{FanCandidate, FanExclusionSet, FanType};
use crate::array_vec::ArrayVec;
use crate::structs::{Meld, Pair, Tile, TileType};
use std::collections::HashSet;

// ── Shared helpers ──

fn is_pung_or_kong(m: &Meld) -> bool {
    matches!(m, Meld::Pung(_) | Meld::Kong(_))
}
fn is_chow(m: &Meld) -> bool {
    matches!(m, Meld::Chow(_))
}
fn is_concealed(m: &Meld) -> bool {
    match m {
        Meld::Pung(t) => t.is_concealed(),
        Meld::Chow(s) => s.is_concealed(),
        Meld::Kong(q) => q.is_concealed(),
    }
}
fn meld_tile(m: Meld) -> Tile {
    match m {
        Meld::Pung(t) => t.tile(),
        Meld::Chow(s) => s.start(),
        Meld::Kong(q) => q.tile(),
    }
}
fn is_terminal(t: Tile) -> bool {
    matches!(t as u8, 0 | 32 | 64 | 96 | 128 | 160)
}
fn island(t: Tile) -> bool {
    t.is_honor()
}
fn rank(t: Tile) -> u8 {
    let v = t as u8;
    match v {
        0..=32 => v / 4 + 1,
        64..=96 => (v - 64) / 4 + 1,
        128..=160 => (v - 128) / 4 + 1,
        _ => 0,
    }
}
fn suit_idx(t: Tile) -> usize {
    match t as u8 {
        0..=32 => 0,
        64..=96 => 1,
        128..=160 => 2,
        _ => 3,
    }
}
fn all_in_range(
    sets: &ArrayVec<Meld, 4>,
    pair: &Pair,
    lo: u8,
    hi: u8,
) -> bool {
    let f = |t: Tile| {
        t.is_suit() && {
            let r = rank(t);
            lo <= r && r <= hi
        }
    };
    sets.iter().all(|m| f(meld_tile(*m))) && f(pair.tile())
}
fn cand(ft: FanType, mask: u64, uses_pair: bool) -> FanCandidate {
    FanCandidate {
        fan_type: ft,
        used_set_mask: mask,
        uses_pair,
        score: ft.points(),
        excludes_mask: FanExclusionSet::default(),
    }
}
fn dragon(t: Tile) -> bool {
    matches!(t, Tile::Red | Tile::Green | Tile::White)
}
fn wind(t: Tile) -> bool {
    matches!(t, Tile::East | Tile::South | Tile::West | Tile::North)
}
fn even_rank(t: Tile) -> bool {
    t.is_suit() && rank(t) % 2 == 0
}
fn is_melded_kong(m: &Meld) -> bool {
    matches!(m, Meld::Kong(q) if !q.is_concealed())
}
fn is_green_tile(t: Tile) -> bool {
    matches!(t,
        Tile::Bamboo2 | Tile::Bamboo3 | Tile::Bamboo4
        | Tile::Bamboo6 | Tile::Bamboo8 | Tile::Green
    )
}
fn is_reversible_tile(t: Tile) -> bool {
    matches!(t,
        Tile::Dot1 | Tile::Dot2 | Tile::Dot3 | Tile::Dot4 | Tile::Dot5
        | Tile::Dot8 | Tile::Dot9
        | Tile::Bamboo2 | Tile::Bamboo4 | Tile::Bamboo5 | Tile::Bamboo6
        | Tile::Bamboo8 | Tile::Bamboo9
        | Tile::White
    )
}

// ═══════════════════════════════════════════════════════════════
// 88-point fans
// ═══════════════════════════════════════════════════════════════

mod big_four_winds {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. }
                if sets.len() == 4
                    && sets.iter().all(|m| is_pung_or_kong(m)) =>
            {
                let tiles: Vec<Tile> =
                    sets.iter().map(|m| meld_tile(*m)).collect();
                if [Tile::East, Tile::South, Tile::West, Tile::North]
                    .iter()
                    .all(|w| tiles.contains(w))
                {
                    return vec![cand(FanType::BigFourWinds, 0b1111, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::BigThreeWinds,
        FanType::AllPungs,
        FanType::PrevalentWind,
        FanType::SeatWind,
        FanType::PungOfTerminalsOrHonors,
    ];
}

mod big_three_dragons {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let found: Vec<Tile> = sets
                    .iter()
                    .filter_map(|m| match m {
                        Meld::Pung(t) if dragon(t.tile()) => Some(t.tile()),
                        Meld::Kong(q) if dragon(q.tile()) => Some(q.tile()),
                        _ => None,
                    })
                    .collect();
                if found.len() >= 3 {
                    return vec![cand(
                        FanType::BigThreeDragons,
                        0b1111,
                        false,
                    )];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::TwoDragonPungs, FanType::DragonPung];
}

mod all_green {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let all_green = sets.iter().all(|m| {
                    match m {
                        Meld::Chow(s) => s.tiles().iter().all(|&t| is_green_tile(t)),
                        Meld::Pung(t) => is_green_tile(t.tile()),
                        Meld::Kong(q) => is_green_tile(q.tile()),
                    }
                }) && is_green_tile(pair.tile());
                if all_green {
                    return vec![cand(FanType::AllGreen, 0b1111, true)];
                }
                vec![]
            }
            Decomposition::SevenPairs { pairs } => {
                if pairs.iter().all(|p| is_green_tile(p.tile())) {
                    return vec![cand(FanType::AllGreen, 0, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod nine_gates {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                // Count tiles of each rank in the suit across all melds + pair
                // Nine Gates pattern: 1112345678999 + one more of 1-9
                let s = suit_idx(meld_tile(sets[0]));
                if !sets.iter().all(|m| {
                    let t = meld_tile(*m);
                    t.is_suit() && suit_idx(t) == s
                }) || !pair.tile().is_suit() || suit_idx(pair.tile()) != s {
                    return vec![];
                }
                // Count must be from a single suit, same as Full Flush
                let mut counts = [0u8; 9];
                for m in sets.iter() {
                    match m {
                        Meld::Pung(t) => {
                            counts[rank(t.tile()) as usize - 1] += 3;
                        }
                        Meld::Kong(q) => {
                            counts[rank(q.tile()) as usize - 1] += 4;
                        }
                        Meld::Chow(s) => {
                            let r = rank(s.start()) as usize - 1;
                            counts[r] += 1;
                            counts[r + 1] += 1;
                            counts[r + 2] += 1;
                        }
                    }
                }
                counts[rank(pair.tile()) as usize - 1] += 2;
                // Nine Gates: 1×3, 2×1, 3×1, 4×1, 5×1, 6×1, 7×1, 8×1, 9×3
                // with exactly one position having +1 extra (the winning tile)
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
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::FullFlush,
        FanType::ConcealedHand,
        FanType::PungOfTerminalsOrHonors,
    ];
}

mod four_kongs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                if sets.iter().filter(|m| matches!(m, Meld::Kong(_))).count()
                    == 4
                {
                    return vec![cand(FanType::FourKongs, 0b1111, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::SingleWait];
}

mod seven_shifted_pairs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::SevenPairs { pairs } => {
                // All pairs must be in same suit, ranks must be consecutive
                let s = suit_idx(pairs[0].tile());
                if !pairs.iter().all(|p| {
                    p.tile().is_suit() && suit_idx(p.tile()) == s
                }) {
                    return vec![];
                }
                let mut ranks: Vec<u8> =
                    pairs.iter().map(|p| rank(p.tile())).collect();
                ranks.sort();
                if ranks.windows(2).all(|w| w[1] == w[0] + 1) {
                    return vec![cand(FanType::SevenShiftedPairs, 0, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::FullFlush,
        FanType::ConcealedHand,
        FanType::SingleWait,
    ];
}

mod thirteen_orphans {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::ThirteenOrphans { .. } => {
                vec![cand(FanType::ThirteenOrphans, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::AllTypes,
        FanType::ConcealedHand,
        FanType::SingleWait,
    ];
}

// ═══════════════════════════════════════════════════════════════
// 64-point fans
// ═══════════════════════════════════════════════════════════════

mod all_terminals {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if sets.iter().all(|m| is_terminal(meld_tile(*m)))
                    && is_terminal(pair.tile())
                {
                    return vec![cand(FanType::AllTerminals, 0b1111, true)];
                }
                vec![]
            }
            Decomposition::SevenPairs { pairs } => {
                if pairs.iter().all(|p| is_terminal(p.tile())) {
                    return vec![cand(FanType::AllTerminals, 0, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::AllPungs,
        FanType::OutsideHand,
        FanType::PungOfTerminalsOrHonors,
        FanType::NoHonors,
    ];
}

mod little_four_winds {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if !wind(pair.tile()) {
                    return vec![];
                }
                let wind_pungs: Vec<Tile> = sets
                    .iter()
                    .filter_map(|m| match m {
                        Meld::Pung(t) if wind(t.tile()) => Some(t.tile()),
                        Meld::Kong(q) if wind(q.tile()) => Some(q.tile()),
                        _ => None,
                    })
                    .collect();
                if wind_pungs.len() < 3 {
                    return vec![];
                }
                let all = [Tile::East, Tile::South, Tile::West, Tile::North];
                let missing: Vec<Tile> = all
                    .iter()
                    .filter(|w| !wind_pungs.contains(w) && **w != pair.tile())
                    .copied()
                    .collect();
                if missing.is_empty() {
                    return vec![cand(FanType::LittleFourWinds, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::BigThreeWinds, FanType::PungOfTerminalsOrHonors];
}

mod little_three_dragons {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let dp: Vec<Tile> = sets
                    .iter()
                    .filter_map(|m| match m {
                        Meld::Pung(t) if dragon(t.tile()) => Some(t.tile()),
                        Meld::Kong(q) if dragon(q.tile()) => Some(q.tile()),
                        _ => None,
                    })
                    .collect();
                if dp.len() == 2 && dragon(pair.tile()) {
                    let all = [Tile::Red, Tile::Green, Tile::White];
                    let missing: Vec<Tile> = all
                        .iter()
                        .filter(|d| !dp.contains(d) && **d != pair.tile())
                        .copied()
                        .collect();
                    if missing.is_empty() {
                        return vec![cand(
                            FanType::LittleThreeDragons,
                            0b1111,
                            true,
                        )];
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::DragonPung, FanType::TwoDragonPungs];
}

mod all_honors {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if sets.iter().all(|m| island(meld_tile(*m)))
                    && island(pair.tile())
                {
                    return vec![cand(FanType::AllHonors, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::AllPungs,
        FanType::OutsideHand,
        FanType::PungOfTerminalsOrHonors,
    ];
}

mod four_concealed_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. }
                if sets.len() == 4
                    && sets
                        .iter()
                        .all(|m| is_pung_or_kong(m) && is_concealed(m)) =>
            {
                vec![cand(FanType::FourConcealedPungs, 0b1111, false)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::AllPungs, FanType::ConcealedHand];
}

mod pure_terminal_chows {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if sets.len() != 4 || pair.tile().is_honor() {
                    return vec![];
                }
                let s = suit_idx(meld_tile(sets[0]));
                let same_suit = |t: Tile| t.is_suit() && suit_idx(t) == s;
                if !same_suit(pair.tile()) || rank(pair.tile()) != 5 {
                    return vec![];
                }
                // Need: two 1-2-3 chows + two 7-8-9 chows in same suit
                if !sets.iter().all(|m| is_chow(m) && same_suit(meld_tile(*m))) {
                    return vec![];
                }
                let mut r1 = 0u8; // count of chows starting at 1
                let mut r7 = 0u8;
                for m in sets.iter() {
                    match rank(meld_tile(*m)) {
                        1 => r1 += 1,
                        7 => r7 += 1,
                        _ => {}
                    }
                }
                if r1 == 2 && r7 == 2 {
                    return vec![cand(FanType::PureTerminalChows, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::SevenPairs,
        FanType::FullFlush,
        FanType::AllChows,
        FanType::PureDoubleChow,
        FanType::TwoTerminalChows,
    ];
}

// ═══════════════════════════════════════════════════════════════
// 48-point fans
// ═══════════════════════════════════════════════════════════════

mod quadruple_chow {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                if chows.len() < 4 {
                    return vec![];
                }
                for i in 0..chows.len() {
                    for j in i + 1..chows.len() {
                        for k in j + 1..chows.len() {
                            for l in k + 1..chows.len() {
                                if chows[i].1 == chows[j].1
                                    && chows[i].1 == chows[k].1
                                    && chows[i].1 == chows[l].1
                                    && chows[i].2 == chows[j].2
                                    && chows[i].2 == chows[k].2
                                    && chows[i].2 == chows[l].2
                                {
                                    return vec![cand(FanType::QuadrupleChow, 0b1111, false)];
                                }
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::PureShiftedPungs,
        FanType::TileHog,
        FanType::PureDoubleChow,
    ];
}

mod four_pure_shifted_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let p: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Pung(t) => Some((i, rank(t.tile()), suit_idx(t.tile()))),
                        Meld::Kong(q) => Some((i, rank(q.tile()), suit_idx(q.tile()))),
                        _ => None,
                    })
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
                                let same_suit = p[a].2 == p[b].2
                                    && p[a].2 == p[c].2
                                    && p[a].2 == p[d].2;
                                if same_suit
                                    && ranks[1] == ranks[0] + 1
                                    && ranks[2] == ranks[1] + 1
                                    && ranks[3] == ranks[2] + 1
                                {
                                    return vec![cand(FanType::FourPureShiftedPungs, 0b1111, false)];
                                }
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::PureTripleChow, FanType::AllPungs];
}

// ═══════════════════════════════════════════════════════════════
// 32-point fans
// ═══════════════════════════════════════════════════════════════

mod four_shifted_chows {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                if chows.len() < 4 {
                    return vec![];
                }
                for a in 0..chows.len() {
                    for b in a + 1..chows.len() {
                        for c in b + 1..chows.len() {
                            for d in c + 1..chows.len() {
                                let selected = [chows[a], chows[b], chows[c], chows[d]];
                                let same_suit = selected.iter().all(|x| x.2 == chows[a].2);
                                if !same_suit { continue; }
                                let mut ranks: Vec<u8> = selected.iter().map(|x| x.1).collect();
                                ranks.sort();
                                // All shifts must be consistent: all +1 or all +2
                                let shift1 = ranks[1] - ranks[0];
                                let shift2 = ranks[2] - ranks[1];
                                let shift3 = ranks[3] - ranks[2];
                                if (shift1 == 1 || shift1 == 2)
                                    && shift1 == shift2
                                    && shift2 == shift3
                                {
                                    return vec![cand(FanType::FourShiftedChows, 0b1111, false)];
                                }
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::ShortStraight];
}

mod three_kongs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let n = sets.iter().filter(|m| matches!(m, Meld::Kong(_))).count();
                if n >= 3 {
                    return vec![cand(FanType::ThreeKongs, 0b1111, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod all_terminals_and_honors {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let f = |t: Tile| is_terminal(t) || island(t);
                if sets.iter().all(|m| f(meld_tile(*m))) && f(pair.tile()) {
                    return vec![cand(FanType::AllTerminalsAndHonors, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::AllPungs, FanType::PungOfTerminalsOrHonors];
}

// ═══════════════════════════════════════════════════════════════
// 24-point fans
// ═══════════════════════════════════════════════════════════════

mod seven_pairs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::SevenPairs { .. } => {
                vec![cand(FanType::SevenPairs, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::ConcealedHand, FanType::SingleWait];
}

mod greater_honors_and_knitted_tiles {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::GreaterHonorsAndKnittedTiles { .. } => {
                vec![cand(FanType::GreaterHonorsAndKnittedTiles, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::AllTypes, FanType::ConcealedHand];
}

mod all_even_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if sets.iter().all(|m| {
                    is_pung_or_kong(m) && even_rank(meld_tile(*m))
                }) && even_rank(pair.tile()) {
                    return vec![cand(FanType::AllEvenPungs, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::AllPungs, FanType::AllSimples];
}

mod full_flush {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let s = suit_idx(meld_tile(sets[0]));
                let same = |t: Tile| t.is_suit() && suit_idx(t) == s;
                if sets.iter().all(|m| same(meld_tile(*m)))
                    && same(pair.tile())
                {
                    return vec![cand(FanType::FullFlush, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::NoHonors];
}

mod pure_triple_chow {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
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
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::PureShiftedPungs, FanType::PureDoubleChow];
}

mod pure_shifted_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let p: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Pung(t) => Some((i, rank(t.tile()), suit_idx(t.tile()))),
                        Meld::Kong(q) => Some((i, rank(q.tile()), suit_idx(q.tile()))),
                        _ => None,
                    })
                    .collect();
                if p.len() < 3 {
                    return vec![];
                }
                for i in 0..p.len() {
                    for j in i + 1..p.len() {
                        for k in j + 1..p.len() {
                            let mut ranks = [p[i].1, p[j].1, p[k].1];
                            ranks.sort();
                            let same_suit = p[i].2 == p[j].2
                                && p[i].2 == p[k].2;
                            if same_suit
                                && ranks[1] == ranks[0] + 1
                                && ranks[2] == ranks[1] + 1
                            {
                                return vec![cand(FanType::PureShiftedPungs, 0b1111, false)];
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::PureTripleChow];
}

mod upper_tiles {
    use super::*;
    fn pairs_in_range(pairs: &[Pair; 7], lo: u8, hi: u8) -> bool {
        let f = |t: Tile| t.is_suit() && lo <= rank(t) && rank(t) <= hi;
        pairs.iter().all(|p| f(p.tile()))
    }
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair }
                if all_in_range(sets, pair, 7, 9) =>
            {
                vec![cand(FanType::UpperTiles, 0b1111, true)]
            }
            Decomposition::SevenPairs { pairs }
                if pairs_in_range(pairs, 7, 9) =>
            {
                vec![cand(FanType::UpperTiles, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::NoHonors];
}

mod middle_tiles {
    use super::*;
    fn pairs_in_range(pairs: &[Pair; 7], lo: u8, hi: u8) -> bool {
        let f = |t: Tile| t.is_suit() && lo <= rank(t) && rank(t) <= hi;
        pairs.iter().all(|p| f(p.tile()))
    }
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair }
                if all_in_range(sets, pair, 4, 6) =>
            {
                vec![cand(FanType::MiddleTiles, 0b1111, true)]
            }
            Decomposition::SevenPairs { pairs }
                if pairs_in_range(pairs, 4, 6) =>
            {
                vec![cand(FanType::MiddleTiles, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::NoHonors, FanType::AllSimples];
}

mod lower_tiles {
    use super::*;
    fn pairs_in_range(pairs: &[Pair; 7], lo: u8, hi: u8) -> bool {
        let f = |t: Tile| t.is_suit() && lo <= rank(t) && rank(t) <= hi;
        pairs.iter().all(|p| f(p.tile()))
    }
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair }
                if all_in_range(sets, pair, 1, 3) =>
            {
                vec![cand(FanType::LowerTiles, 0b1111, true)]
            }
            Decomposition::SevenPairs { pairs }
                if pairs_in_range(pairs, 1, 3) =>
            {
                vec![cand(FanType::LowerTiles, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::NoHonors];
}

// ═══════════════════════════════════════════════════════════════
// 16-point fans
// ═══════════════════════════════════════════════════════════════

mod pure_straight {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                if chows.len() < 3 {
                    return vec![];
                }
                // Try all 3-element subsets to find a Pure Straight
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
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod three_suited_terminal_chows {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if sets.len() != 4 || pair.tile().is_honor() || rank(pair.tile()) != 5 {
                    return vec![];
                }
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                if chows.len() != 4 {
                    return vec![];
                }
                // Need 1-2-3 and 7-8-9 in two suits (4 chows total)
                let mut suits_found = HashSet::new();
                for (_, r, s) in &chows {
                    suits_found.insert(*s);
                }
                if suits_found.len() != 2 {
                    return vec![];
                }
                // Check each suit has both terminal chows
                let mut s_list: Vec<usize> = suits_found.into_iter().collect();
                for s in &s_list {
                    let has_1 = chows.iter().any(|&(_, r, ss)| ss == *s && r == 1);
                    let has_7 = chows.iter().any(|&(_, r, ss)| ss == *s && r == 7);
                    if !has_1 || !has_7 {
                        return vec![];
                    }
                }
                // Pair must be 5 in the remaining (third) suit
                let pair_s = suit_idx(pair.tile());
                if s_list.contains(&pair_s) {
                    return vec![];
                }
                vec![cand(FanType::ThreeSuitedTerminalChows, 0b1111, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[
        FanType::PureDoubleChow,
        FanType::TwoTerminalChows,
        FanType::NoHonors,
        FanType::AllChows,
    ];
}

mod pure_shifted_chows {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                if chows.len() < 3 {
                    return vec![];
                }
                for i in 0..chows.len() {
                    for j in i + 1..chows.len() {
                        for k in j + 1..chows.len() {
                            let selected = [chows[i], chows[j], chows[k]];
                            let same_suit = selected.iter().all(|x| x.2 == chows[i].2);
                            if !same_suit { continue; }
                            let mut ranks: Vec<u8> = selected.iter().map(|x| x.1).collect();
                            ranks.sort();
                            let s1 = ranks[1] - ranks[0];
                            let s2 = ranks[2] - ranks[1];
                            if (s1 == 1 || s1 == 2) && s1 == s2 {
                                return vec![cand(FanType::PureShiftedChows, 0b1111, false)];
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod all_fives {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let f = |t: Tile| t.is_suit() && rank(t) == 5;
                let ok = sets.iter().all(|m| match m {
                    Meld::Chow(s) => s.tiles().iter().any(|&t| f(t)),
                    Meld::Pung(_) | Meld::Kong(_) => f(meld_tile(*m)),
                }) && f(pair.tile());
                if ok {
                    return vec![cand(FanType::AllFives, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::AllSimples];
}

mod triple_pung {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let p: Vec<(usize, Tile)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Pung(t) => Some((i, t.tile())),
                        Meld::Kong(q) => Some((i, q.tile())),
                        _ => None,
                    })
                    .collect();
                for i in 0..p.len() {
                    for j in i + 1..p.len() {
                        for k in j + 1..p.len() {
                            let r = rank(p[i].1);
                            if rank(p[j].1) == r && rank(p[k].1) == r {
                                let suits: HashSet<usize> = [
                                    suit_idx(p[i].1),
                                    suit_idx(p[j].1),
                                    suit_idx(p[k].1),
                                ]
                                .into();
                                if suits.len() == 3 {
                                    return vec![cand(
                                        FanType::TriplePung,
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
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod three_concealed_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let n = sets
                    .iter()
                    .filter(|m| is_pung_or_kong(m) && is_concealed(m))
                    .count();
                if n >= 3 {
                    return vec![cand(FanType::ThreeConcealedPungs, 0b1111, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

// ═══════════════════════════════════════════════════════════════
// 12-point fans
// ═══════════════════════════════════════════════════════════════

mod lesser_honors_and_knitted_tiles {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::LesserHonorsAndKnittedTiles { .. } => {
                vec![cand(FanType::LesserHonorsAndKnittedTiles, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] =
        &[FanType::AllTypes, FanType::ConcealedHand];
}

mod knitted_straight {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::KnittedStraight { .. } => {
                vec![cand(FanType::KnittedStraight, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod upper_four {
    use super::*;
    fn pairs_in_range(pairs: &[Pair; 7], lo: u8, hi: u8) -> bool {
        let f = |t: Tile| t.is_suit() && lo <= rank(t) && rank(t) <= hi;
        pairs.iter().all(|p| f(p.tile()))
    }
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair }
                if all_in_range(sets, pair, 6, 9) =>
            {
                vec![cand(FanType::UpperFour, 0b1111, true)]
            }
            Decomposition::SevenPairs { pairs }
                if pairs_in_range(pairs, 6, 9) =>
            {
                vec![cand(FanType::UpperFour, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::NoHonors];
}

mod lower_four {
    use super::*;
    fn pairs_in_range(pairs: &[Pair; 7], lo: u8, hi: u8) -> bool {
        let f = |t: Tile| t.is_suit() && lo <= rank(t) && rank(t) <= hi;
        pairs.iter().all(|p| f(p.tile()))
    }
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair }
                if all_in_range(sets, pair, 1, 4) =>
            {
                vec![cand(FanType::LowerFour, 0b1111, true)]
            }
            Decomposition::SevenPairs { pairs }
                if pairs_in_range(pairs, 1, 4) =>
            {
                vec![cand(FanType::LowerFour, 0, true)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::NoHonors];
}

mod big_three_winds {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let n = sets
                    .iter()
                    .filter(|m| match m {
                        Meld::Pung(t) => wind(t.tile()),
                        Meld::Kong(q) => wind(q.tile()),
                        _ => false,
                    })
                    .count();
                if n >= 3 {
                    return vec![cand(FanType::BigThreeWinds, 0b1111, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

// ═══════════════════════════════════════════════════════════════
// 8-point fans
// ═══════════════════════════════════════════════════════════════

mod mixed_straight {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                if chows.len() < 3 {
                    return vec![];
                }
                // Find three chows covering 1-2-3, 4-5-6, 7-8-9 in different suits
                for i in 0..chows.len() {
                    for j in i + 1..chows.len() {
                        for k in j + 1..chows.len() {
                            let mut r: Vec<u8> = vec![chows[i].1, chows[j].1, chows[k].1];
                            r.sort();
                            if r != vec![1, 4, 7] { continue; }
                            let suits: HashSet<usize> =
                                [chows[i].2, chows[j].2, chows[k].2].into();
                            if suits.len() == 3 {
                                return vec![cand(FanType::MixedStraight, 0b1111, false)];
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod reversible_tiles {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let all_rev = sets.iter().all(|m| {
                    match m {
                        Meld::Chow(s) => s.tiles().iter().all(|&t| is_reversible_tile(t)),
                        Meld::Pung(t) => is_reversible_tile(t.tile()),
                        Meld::Kong(q) => is_reversible_tile(q.tile()),
                    }
                }) && is_reversible_tile(pair.tile());
                if all_rev {
                    return vec![cand(FanType::ReversibleTiles, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::OneVoidedSuit];
}

mod mixed_triple_chow {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                if chows.len() < 3 {
                    return vec![];
                }
                for i in 0..chows.len() {
                    for j in i + 1..chows.len() {
                        for k in j + 1..chows.len() {
                            if chows[i].1 == chows[j].1
                                && chows[i].1 == chows[k].1
                            {
                                let suits: HashSet<usize> =
                                    [chows[i].2, chows[j].2, chows[k].2].into();
                                if suits.len() == 3 {
                                    return vec![cand(FanType::MixedTripleChow, 0b1111, false)];
                                }
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod mixed_shifted_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let p: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Pung(t) => Some((i, rank(t.tile()), suit_idx(t.tile()))),
                        Meld::Kong(q) => Some((i, rank(q.tile()), suit_idx(q.tile()))),
                        _ => None,
                    })
                    .collect();
                if p.len() < 3 {
                    return vec![];
                }
                for i in 0..p.len() {
                    for j in i + 1..p.len() {
                        for k in j + 1..p.len() {
                            let suits: HashSet<usize> =
                                [p[i].2, p[j].2, p[k].2].into();
                            if suits.len() != 3 { continue; }
                            let mut ranks = [p[i].1, p[j].1, p[k].1];
                            ranks.sort();
                            if ranks[1] == ranks[0] + 1 && ranks[2] == ranks[1] + 1 {
                                return vec![cand(FanType::MixedShiftedPungs, 0b1111, false)];
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod chicken_hand {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        vec![]
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod last_tile_draw {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.is_last_tile_draw {
            vec![cand(FanType::LastTileDraw, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod last_tile_claim {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.is_last_tile_claim {
            vec![cand(FanType::LastTileClaim, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod out_with_replacement_tile {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.win_method == WinMethod::KongReplacement {
            vec![cand(FanType::OutWithReplacementTile, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod robbing_the_kong {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.win_method == WinMethod::RobKong {
            vec![cand(FanType::RobbingTheKong, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod two_concealed_kongs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let n = sets
                    .iter()
                    .filter(|m| matches!(m, Meld::Kong(q) if q.is_concealed()))
                    .count();
                if n >= 2 {
                    return vec![cand(
                        FanType::TwoConcealedKongs,
                        0b1111,
                        false,
                    )];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

// ═══════════════════════════════════════════════════════════════
// 6-point fans
// ═══════════════════════════════════════════════════════════════

mod all_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. }
                if sets.len() == 4
                    && sets.iter().all(|m| is_pung_or_kong(m)) =>
            {
                vec![cand(FanType::AllPungs, 0b1111, false)]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod half_flush {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let mut suits: HashSet<usize> = sets
                    .iter()
                    .filter_map(|m| {
                        let t = meld_tile(*m);
                        if t.is_suit() { Some(suit_idx(t)) } else { None }
                    })
                    .collect();
                if pair.tile().is_suit() {
                    suits.insert(suit_idx(pair.tile()));
                }
                if suits.len() == 1
                    && (sets.iter().any(|m| island(meld_tile(*m)))
                        || island(pair.tile()))
                {
                    return vec![cand(FanType::HalfFlush, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod mixed_shifted_chows {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                if chows.len() < 3 {
                    return vec![];
                }
                for i in 0..chows.len() {
                    for j in i + 1..chows.len() {
                        for k in j + 1..chows.len() {
                            let suits: HashSet<usize> =
                                [chows[i].2, chows[j].2, chows[k].2].into();
                            if suits.len() != 3 { continue; }
                            let mut ranks = [chows[i].1, chows[j].1, chows[k].1];
                            ranks.sort();
                            if ranks[1] == ranks[0] + 1 && ranks[2] == ranks[1] + 1 {
                                return vec![cand(FanType::MixedShiftedChows, 0b1111, false)];
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod all_types {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let mut t = HashSet::new();
                for m in sets.iter() {
                    match meld_tile(*m).get_type() {
                        TileType::Character => {
                            t.insert("c");
                        }
                        TileType::Dot => {
                            t.insert("d");
                        }
                        TileType::Bamboo => {
                            t.insert("b");
                        }
                        TileType::Wind => {
                            t.insert("w");
                        }
                        TileType::Dragon => {
                            t.insert("r");
                        }
                        _ => {}
                    }
                }
                match pair.tile().get_type() {
                    TileType::Character => {
                        t.insert("c");
                    }
                    TileType::Dot => {
                        t.insert("d");
                    }
                    TileType::Bamboo => {
                        t.insert("b");
                    }
                    TileType::Wind => {
                        t.insert("w");
                    }
                    TileType::Dragon => {
                        t.insert("r");
                    }
                    _ => {}
                }
                if t.len() == 5 {
                    return vec![cand(FanType::AllTypes, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod melded_hand {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                // All sets must be exposed (not self-drawn) and the win
                // must be by discard. The pair also from discard.
                let all_exposed = sets.iter().all(|m| !is_concealed(m));
                if all_exposed
                    && ctx.win_method == WinMethod::Discard
                    && ctx.wait_type == WaitType::Single
                {
                    return vec![cand(FanType::MeldedHand, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[FanType::SingleWait];
}

mod two_dragon_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let n = sets
                    .iter()
                    .filter(|m| match m {
                        Meld::Pung(t) => dragon(t.tile()),
                        Meld::Kong(q) => dragon(q.tile()),
                        _ => false,
                    })
                    .count();
                if n >= 2 {
                    return vec![cand(FanType::TwoDragonPungs, 0b1111, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

// ═══════════════════════════════════════════════════════════════
// 4-point fans
// ═══════════════════════════════════════════════════════════════

mod outside_hand {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        fn meld_has_terminal_or_honor(m: &Meld) -> bool {
            match m {
                Meld::Chow(s) => {
                    s.tiles().iter().any(|&t| is_terminal(t) || island(t))
                }
                Meld::Pung(t) => {
                    let tile = t.tile();
                    is_terminal(tile) || island(tile)
                }
                Meld::Kong(q) => {
                    let tile = q.tile();
                    is_terminal(tile) || island(tile)
                }
            }
        }
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if sets.iter().all(|m| meld_has_terminal_or_honor(m))
                    && (is_terminal(pair.tile()) || island(pair.tile()))
                {
                    return vec![cand(FanType::OutsideHand, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod fully_concealed {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.is_fully_concealed {
            vec![cand(FanType::FullyConcealed, 0, true)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod two_melded_kongs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let n = sets.iter().filter(|m| is_melded_kong(m)).count();
                if n >= 2 {
                    return vec![cand(FanType::TwoMeldedKongs, 0b1111, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod last_tile {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.is_last_tile_of_kind {
            vec![cand(FanType::LastTile, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

// ═══════════════════════════════════════════════════════════════
// 2-point fans
// ═══════════════════════════════════════════════════════════════

mod dragon_pung {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let mut mask = 0u64;
                for (i, m) in sets.iter().enumerate() {
                    if let Meld::Pung(t) = m {
                        if dragon(t.tile()) {
                            mask |= 1 << i;
                        }
                    }
                    if let Meld::Kong(q) = m {
                        if dragon(q.tile()) {
                            mask |= 1 << i;
                        }
                    }
                }
                if mask != 0 {
                    return vec![cand(FanType::DragonPung, mask, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod prevalent_wind {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let target = ctx.prevalent_wind_to_tile();
                for (i, m) in sets.iter().enumerate() {
                    if let Meld::Pung(t) = m {
                        if t.tile() == target {
                            return vec![cand(
                                FanType::PrevalentWind,
                                1 << i,
                                false,
                            )];
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod seat_wind {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let target = ctx.seat_wind_to_tile();
                for (i, m) in sets.iter().enumerate() {
                    if let Meld::Pung(t) = m {
                        if t.tile() == target {
                            return vec![cand(
                                FanType::SeatWind,
                                1 << i,
                                false,
                            )];
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod concealed_hand {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.is_concealed && ctx.win_method == WinMethod::Discard {
            vec![cand(FanType::ConcealedHand, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod all_chows {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if sets.len() == 4
                    && sets.iter().all(|m| is_chow(m))
                    && !island(pair.tile())
                {
                    return vec![cand(FanType::AllChows, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod tile_hog {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                // Count occurrences of each suit tile across all melds + pair
                let mut counts = [0u8; 9 * 3]; // 3 suits × 9 ranks
                for m in sets.iter() {
                    match m {
                        Meld::Pung(t) => {
                            let t = t.tile();
                            if t.is_suit() {
                                let idx = suit_idx(t) * 9 + (rank(t) as usize - 1);
                                counts[idx] += 3;
                            }
                        }
                        Meld::Kong(q) => {
                            // Kong already uses all 4 — no hog possible
                        }
                        Meld::Chow(s) => {
                            for t in s.tiles() {
                                let idx = suit_idx(t) * 9 + (rank(t) as usize - 1);
                                counts[idx] += 1;
                            }
                        }
                    }
                }
                if pair.tile().is_suit() {
                    let t = pair.tile();
                    let idx = suit_idx(t) * 9 + (rank(t) as usize - 1);
                    counts[idx] += 2;
                }
                let mut mask = 0u64;
                if counts.iter().any(|&c| c == 4) {
                    mask = 0b1111; // marks that tile hog applies to the hand
                }
                if mask != 0 {
                    return vec![cand(FanType::TileHog, mask, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod double_pung {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let p: Vec<(usize, Tile)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Pung(t) => Some((i, t.tile())),
                        Meld::Kong(q) => Some((i, q.tile())),
                        _ => None,
                    })
                    .collect();
                let mut mask = 0u64;
                for i in 0..p.len() {
                    for j in i + 1..p.len() {
                        let (r1, r2) = (rank(p[i].1), rank(p[j].1));
                        let (s1, s2) = (suit_idx(p[i].1), suit_idx(p[j].1));
                        if r1 > 0 && r1 == r2 && s1 != s2 {
                            mask |= 1 << p[i].0 | 1 << p[j].0;
                        }
                    }
                }
                if mask != 0 {
                    return vec![cand(FanType::DoublePung, mask, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod two_concealed_pungs {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                if sets.iter().filter(|m| is_concealed(m)).count() >= 2 {
                    return vec![cand(
                        FanType::TwoConcealedPungs,
                        0b1111,
                        false,
                    )];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod concealed_kong {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let mut mask = 0u64;
                for (i, m) in sets.iter().enumerate() {
                    if let Meld::Kong(q) = m {
                        if q.is_concealed() {
                            mask |= 1 << i;
                        }
                    }
                }
                if mask != 0 {
                    return vec![cand(FanType::ConcealedKong, mask, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod all_simples {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let f = |t: Tile| t.is_suit() && !is_terminal(t);
                if sets.iter().all(|m| f(meld_tile(*m))) && f(pair.tile()) {
                    return vec![cand(FanType::AllSimples, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

// ═══════════════════════════════════════════════════════════════
// 1-point fans
// ═══════════════════════════════════════════════════════════════

mod pure_double_chow {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
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
                    return vec![cand(FanType::PureDoubleChow, mask, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod mixed_double_chow {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
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
                    return vec![cand(FanType::MixedDoubleChow, mask, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod short_straight {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                let mut mask = 0u64;
                for i in 0..chows.len() {
                    for j in i + 1..chows.len() {
                        if chows[i].2 == chows[j].2
                            && (chows[i].1 + 3 == chows[j].1 || chows[j].1 + 3 == chows[i].1)
                        {
                            mask |= 1 << chows[i].0 | 1 << chows[j].0;
                        }
                    }
                }
                if mask != 0 {
                    return vec![cand(FanType::ShortStraight, mask, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod two_terminal_chows {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let chows: Vec<(usize, u8, usize)> = sets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, m)| match m {
                        Meld::Chow(s) => Some((i, rank(s.start()), suit_idx(s.start()))),
                        _ => None,
                    })
                    .collect();
                // Group chows by suit
                let mut by_suit: [Vec<u8>; 3] = [vec![], vec![], vec![]];
                let mut id_by_suit: [Vec<usize>; 3] = [vec![], vec![], vec![]];
                for (i, r, s) in &chows {
                    by_suit[*s].push(*r);
                    id_by_suit[*s].push(*i);
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
                    return vec![cand(FanType::TwoTerminalChows, mask, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod pung_of_terminals_or_honors {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let mut mask = 0u64;
                for (i, m) in sets.iter().enumerate() {
                    let t = meld_tile(*m);
                    if is_pung_or_kong(m) && (is_terminal(t) || wind(t)) {
                        mask |= 1 << i;
                    }
                }
                if mask != 0 {
                    return vec![cand(
                        FanType::PungOfTerminalsOrHonors,
                        mask,
                        false,
                    )];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod melded_kong {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, .. } => {
                let mut mask = 0u64;
                for (i, m) in sets.iter().enumerate() {
                    if is_melded_kong(m) {
                        mask |= 1 << i;
                    }
                }
                if mask != 0 {
                    return vec![cand(FanType::MeldedKong, mask, false)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod one_voided_suit {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                let mut suits = HashSet::new();
                for m in sets.iter() {
                    let t = meld_tile(*m);
                    if t.is_suit() {
                        suits.insert(suit_idx(t));
                    }
                }
                if pair.tile().is_suit() {
                    suits.insert(suit_idx(pair.tile()));
                }
                if suits.len() <= 2 && !suits.is_empty() {
                    return vec![cand(FanType::OneVoidedSuit, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod no_honors {
    use super::*;
    pub(crate) fn check(
        decomp: &Decomposition,
        _ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        match decomp {
            Decomposition::Standard { sets, pair } => {
                if sets.iter().all(|m| !island(meld_tile(*m)))
                    && !island(pair.tile())
                {
                    return vec![cand(FanType::NoHonors, 0b1111, true)];
                }
                vec![]
            }
            _ => vec![],
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod edge_wait {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.wait_type == WaitType::Edge {
            vec![cand(FanType::EdgeWait, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod closed_wait {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.wait_type == WaitType::Closed {
            vec![cand(FanType::ClosedWait, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod single_wait {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.wait_type == WaitType::Single {
            vec![cand(FanType::SingleWait, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod self_drawn {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.win_method == WinMethod::SelfDraw {
            vec![cand(FanType::SelfDrawn, 0, true)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

mod flower_tiles {
    use super::*;
    pub(crate) fn check(
        _decomp: &Decomposition,
        ctx: &FanContext,
    ) -> Vec<FanCandidate> {
        if ctx.flower_count > 0 {
            vec![cand(FanType::FlowerTiles, 0, false)]
        } else {
            vec![]
        }
    }
    pub(crate) const EXCLUDES: &[FanType] = &[];
}

// ═══════════════════════════════════════════════════════════════
// Enum + dispatch — generated by macro
// ═══════════════════════════════════════════════════════════════

macro_rules! define_rules {
    ($($mod:ident),+ $(,)?) => {
        #[allow(non_camel_case_types)]
        pub(crate) enum RuleKind { $($mod),+ }

        impl RuleKind {
            pub(crate) fn check(&self, decomp: &Decomposition, ctx: &FanContext) -> Vec<FanCandidate> {
                match self { $(RuleKind::$mod => $mod::check(decomp, ctx)),+ }
            }
            pub(crate) fn excludes(&self) -> &'static [FanType] {
                match self { $(RuleKind::$mod => $mod::EXCLUDES),+ }
            }
        }

        pub(crate) const ALL_RULE_KINDS: &[RuleKind] = &[$(RuleKind::$mod),+];
    };
}

define_rules! {
    big_four_winds, big_three_dragons, all_green, nine_gates, four_kongs,
    seven_shifted_pairs, thirteen_orphans,
    all_terminals, little_four_winds, little_three_dragons, all_honors,
    four_concealed_pungs, pure_terminal_chows,
    quadruple_chow, four_pure_shifted_pungs,
    four_shifted_chows, three_kongs, all_terminals_and_honors,
    seven_pairs, greater_honors_and_knitted_tiles, all_even_pungs, full_flush,
    pure_triple_chow, pure_shifted_pungs, upper_tiles, middle_tiles, lower_tiles,
    pure_straight, three_suited_terminal_chows, pure_shifted_chows, all_fives,
    triple_pung, three_concealed_pungs,
    lesser_honors_and_knitted_tiles, knitted_straight, upper_four, lower_four,
    big_three_winds,
    mixed_straight, reversible_tiles, mixed_triple_chow, mixed_shifted_pungs,
    chicken_hand, last_tile_draw, last_tile_claim, out_with_replacement_tile,
    robbing_the_kong, two_concealed_kongs,
    all_pungs, half_flush, mixed_shifted_chows, all_types, melded_hand,
    two_dragon_pungs,
    outside_hand, fully_concealed, two_melded_kongs, last_tile,
    dragon_pung, prevalent_wind, seat_wind, concealed_hand, all_chows, tile_hog,
    double_pung, two_concealed_pungs, concealed_kong, all_simples,
    pure_double_chow, mixed_double_chow, short_straight, two_terminal_chows,
    pung_of_terminals_or_honors, melded_kong, one_voided_suit, no_honors,
    edge_wait, closed_wait, single_wait, self_drawn, flower_tiles,
}

// ═══════════════════════════════════════════════════════════════
// Public entry point
// ═══════════════════════════════════════════════════════════════

/// Check all rules against a decomposition + context.
/// Returns candidates paired with their exclusion lists.
pub(crate) fn check_all(
    decomp: &Decomposition,
    ctx: &FanContext,
) -> Vec<(FanCandidate, &'static [FanType])> {
    let mut result = Vec::new();
    for kind in ALL_RULE_KINDS {
        let candidates = kind.check(decomp, ctx);
        if !candidates.is_empty() {
            let ex = kind.excludes();
            result.extend(candidates.into_iter().map(|c| (c, ex)));
        }
    }
    result
}
