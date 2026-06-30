// ═══════════════════════════════════════════════════════════════
// 4 / 2 / 1 point rules
// ═══════════════════════════════════════════════════════════════

#![allow(non_snake_case)]

use std::collections::HashSet;

use super::super::{DynamicFanContext, StaticFanContext, WaitType, WinMethod};
use crate::solver::{DecomposeResult, Decomposition};
use super::helpers::{
    MeldKind, cand, is_chow, is_honor_tile, is_melded_kong, is_pung_or_kong,
    is_terminal_tile, is_wind_tile, meld_info, pair_info, rank_of,
};
use super::{FanCandidate, FanType};

// ── 4 points ──────────────────────────────────────────────────

pub(crate) fn outside_hand(
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
    fn meld_has_terminal_or_honor(m: &super::helpers::MeldInfo) -> bool {
        match m.kind {
            MeldKind::Chow => m.tiles[..3]
                .iter()
                .any(|&t| is_terminal_tile(t) || is_honor_tile(t)),
            _ => is_terminal_tile(m.tile) || is_honor_tile(m.tile),
        }
    }
    if sets
        .iter()
        .all(|m| meld_has_terminal_or_honor(&meld_info(m)))
        && (is_terminal_tile(pair_info(pair).tile)
            || is_honor_tile(pair_info(pair).tile))
    {
        vec![cand(FanType::OutsideHand, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn fully_concealed(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if static_ctx.is_fully_concealed {
        vec![cand(FanType::FullyConcealed, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn two_melded_kongs(
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
    let cnt = sets
        .iter()
        .filter(|m| is_melded_kong(m))
        .count();
    if cnt >= 2 {
        vec![cand(FanType::TwoMeldedKongs, 0b1111, false)]
    } else {
        vec![]
    }
}

pub(crate) fn last_tile(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if static_ctx.is_last_tile_of_kind {
        vec![cand(FanType::LastTile, 0, false)]
    } else {
        vec![]
    }
}

// ── 2 points ──────────────────────────────────────────────────

pub(crate) fn dragon_pung(
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
    let mut mask = 0u64;
    for (i, m) in sets.iter().enumerate() {
        if meld_info(m).is_dragon && is_pung_or_kong(m) {
            mask |= 1 << i;
        }
    }
    if mask != 0 {
        vec![cand(FanType::DragonPung, mask, false)]
    } else {
        vec![]
    }
}

pub(crate) fn prevalent_wind(
    decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let target = static_ctx.prevalent_wind.to_tile();
    for (i, m) in sets.iter().enumerate() {
        if matches!(meld_info(m).kind, MeldKind::Pung) && meld_info(m).tile == target {
            return vec![cand(FanType::PrevalentWind, 1 << i, false)];
        }
    }
    vec![]
}

pub(crate) fn seat_wind(
    decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };
    let n = sets.len();
    let target = static_ctx.seat_wind.to_tile();
    for (i, m) in sets.iter().enumerate() {
        if matches!(meld_info(m).kind, MeldKind::Pung) && meld_info(m).tile == target {
            return vec![cand(FanType::SeatWind, 1 << i, false)];
        }
    }
    vec![]
}

pub(crate) fn concealed_hand(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if static_ctx.is_concealed && static_ctx.win_method == WinMethod::Discard {
        vec![cand(FanType::ConcealedHand, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn all_chows(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    let (pair, sets) = match &decomp.decompositions {
        Decomposition::Standard { pair, sets } => (pair, sets),
        _ => return vec![],
    };

    if false {
        vec![cand(FanType::AllChows, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn tile_hog(
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
    // Count occurrences of each suit tile across all melds + pair
    // 3 suits × 9 ranks = 27 counters
    let mut counts = [0u8; 27];
    for m in sets.iter() {
        if meld_info(m).is_honor {
            continue;
        }
        let base = meld_info(m).suit as usize * 9;
        match meld_info(m).kind {
            MeldKind::Pung => counts[base + meld_info(m).rank as usize - 1] += 3,
            MeldKind::Kong => counts[base + meld_info(m).rank as usize - 1] += 4,
            MeldKind::Chow => {
                // All three tiles in the chow
                for t in &meld_info(m).tiles[..3] {
                    if !t.is_suit() {
                        continue;
                    }
                    let r = rank_of(*t);
                    counts[base + r as usize - 1] += 1;
                }
            }
        }
    }
    if !pair_info(pair).is_honor {
        let base = pair_info(pair).suit as usize * 9;
        counts[base + pair_info(pair).rank as usize - 1] += 2;
    }
    if counts.iter().any(|&c| c >= 4) {
        vec![cand(FanType::TileHog, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn double_pung(
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

pub(crate) fn two_concealed_pungs(
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
    if sets.iter().filter(|m| meld_info(m).is_concealed).count() >= 2 {
        vec![cand(FanType::TwoConcealedPungs, 0b1111, false)]
    } else {
        vec![]
    }
}

pub(crate) fn concealed_kong(
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
    let mut mask = 0u64;
    for (i, m) in sets.iter().enumerate() {
        if matches!(meld_info(m).kind, MeldKind::Kong) && meld_info(m).is_concealed {
            mask |= 1 << i;
        }
    }
    if mask != 0 {
        vec![cand(FanType::ConcealedKong, mask, false)]
    } else {
        vec![]
    }
}

pub(crate) fn all_simples(
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
    if sets
        .iter()
        .all(|m| meld_info(m).suit < 3 && !meld_info(m).is_terminal)
        && pair_info(pair).suit < 3
        && !pair_info(pair).is_terminal
    {
        vec![cand(FanType::AllSimples, 0, true)]
    } else {
        vec![]
    }
}

// ── 1 point ───────────────────────────────────────────────────

pub(crate) fn pure_double_chow(
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

pub(crate) fn mixed_double_chow(
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

pub(crate) fn short_straight(
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

pub(crate) fn two_terminal_chows(
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

pub(crate) fn pung_of_terminals_or_honors(
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
    let mut mask = 0u64;
    for (i, m) in sets.iter().enumerate() {
        if is_pung_or_kong(m)
            && (is_terminal_tile(meld_info(m).tile) || is_wind_tile(meld_info(m).tile))
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

pub(crate) fn melded_kong(
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
    let mut mask = 0u64;
    for (i, m) in sets.iter().enumerate() {
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

pub(crate) fn one_voided_suit(
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
    let mut suits = HashSet::new();
    for m in sets.iter() {
        if meld_info(m).suit < 3 {
            suits.insert(meld_info(m).suit);
        }
    }
    if pair_info(pair).suit < 3 {
        suits.insert(pair_info(pair).suit);
    }
    if suits.len() <= 2 && !suits.is_empty() {
        vec![cand(FanType::OneVoidedSuit, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn no_honors(
    decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if false {
        vec![cand(FanType::NoHonors, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn edge_wait(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    wait_type: WaitType,
) -> Vec<FanCandidate> {
    if wait_type == WaitType::Edge {
        vec![cand(FanType::EdgeWait, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn closed_wait(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    wait_type: WaitType,
) -> Vec<FanCandidate> {
    if wait_type == WaitType::Closed {
        vec![cand(FanType::ClosedWait, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn single_wait(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    wait_type: WaitType,
) -> Vec<FanCandidate> {
    if wait_type == WaitType::Single {
        vec![cand(FanType::SingleWait, 0, false)]
    } else {
        vec![]
    }
}

pub(crate) fn self_drawn(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if static_ctx.win_method == WinMethod::SelfDraw {
        vec![cand(FanType::SelfDrawn, 0, true)]
    } else {
        vec![]
    }
}

pub(crate) fn flower_tiles(
    _decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanCandidate> {
    if static_ctx.flower_count > 0 {
        vec![cand(FanType::FlowerTiles, 0, false)]
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
    fn test_two_dragon_pungs() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, true)),
                Meld::Pung(Triplet::new(Tile::Green, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
            ],
            Tile::Bamboo8,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::TwoDragonPungs);
    }

    // ═══════════════════════════════════════════════════════════════
    // 4-point fans
    // ═══════════════════════════════════════════════════════════════

    // ── 55. Outside Hand ──
    //
    // NOTE: The current check only inspects `meld_tile` (the start tile
    // of a chow).  For chows like 7-8-9 the terminal (9) is at the end
    // and not detected.  Fix when chow tile iteration is added.

    #[test]
    fn test_outside_hand_example1() {
        // Dot7-8-9, Char7-8-9, Bam7-8-9, Bam7-8-9 chows, pair Bam9
        // Each chow contains a terminal (9), so Outside Hand should apply.
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Dot7, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            ],
            Tile::Bamboo9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::OutsideHand);
    }

    #[test]
    fn test_outside_hand_example2() {
        // One chow is a declared meld (exposed), breaking the symmetrical
        // tile pattern that would also qualify for Seven Pairs.
        let hand = declared_hand(
            vec![Meld::Chow(Sequence::new(Tile::Character1, false))],
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
            ],
            Tile::Red,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::OutsideHand);
    }

    #[test]
    fn test_outside_hand_example3() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Dot1, true)),
                Meld::Pung(Triplet::new(Tile::Character9, true)),
                Meld::Chow(Sequence::new(Tile::Dot7, true)),
            ],
            Tile::East,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::OutsideHand);
    }

    // ── 56. Fully Concealed ──

    #[test]
    fn test_fully_concealed() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character2, true)),
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Dot6, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo6, true)),
            ],
            Tile::Bamboo4,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.is_fully_concealed = true;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::FullyConcealed);
    }

    // ── 57. Two Melded Kongs ──

    #[test]
    fn test_two_melded_kongs_example1() {
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Dot4, false)),
                Meld::Kong(Quad::new(Tile::Character4, false)),
                Meld::Pung(Triplet::new(Tile::Red, false)),
                Meld::Pung(Triplet::new(Tile::Character1, false)),
            ],
            Tile::Character3,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::TwoMeldedKongs);
    }

    #[test]
    fn test_two_melded_kongs_example2() {
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Dot2, false)),
                Meld::Kong(Quad::new(Tile::White, false)),
                Meld::Pung(Triplet::new(Tile::Bamboo6, false)),
                Meld::Chow(Sequence::new(Tile::Dot3, false)),
            ],
            Tile::Bamboo4,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::TwoMeldedKongs);
    }

    // ═══════════════════════════════════════════════════════════════
    // 2-point fans
    // ═══════════════════════════════════════════════════════════════

    // ── 58. Last Tile ──

    #[test]
    fn test_last_tile() {
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
        static_ctx.is_last_tile_of_kind = true;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::LastTile);
    }

    // ── 59. Dragon Pung ──

    #[test]
    fn test_dragon_pung() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Red, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
            ],
            Tile::Character5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::DragonPung);
    }

    // ── 60. Prevalent Wind ──

    #[test]
    fn test_prevalent_wind_east() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::East, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
            ],
            Tile::Character5,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.prevalent_wind = Wind::West;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::PrevalentWind);
    }

    // ── 61. Seat Wind ──

    #[test]
    fn test_seat_wind_south() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::South, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
            ],
            Tile::Character5,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.seat_wind = Wind::South;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::SeatWind);
    }

    // ── 62. Concealed Hand ──

    #[test]
    fn test_concealed_hand() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character2, true)),
                Meld::Chow(Sequence::new(Tile::Character5, true)),
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Dot6, true)),
            ],
            Tile::Bamboo6,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.is_concealed = true;
        static_ctx.win_method = WinMethod::Discard;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ConcealedHand);
    }

    // ── 63. All Chows ──

    #[test]
    fn test_all_chows() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character2, true)),
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            ],
            Tile::Dot7,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::AllChows);
    }

    // ── 64. Tile Hog ──

    #[test]
    fn test_tile_hog() {
        // Dot7-8-9, Bam7-8-9, Wan7-8-9, Wan7 pung, pair Dot3
        // Wan7 appears in pung (3) + chow (1) = 4 times → Tile Hog
        let hand = all_declared_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Dot7, false)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, false)),
                Meld::Chow(Sequence::new(Tile::Character7, false)),
                Meld::Pung(Triplet::new(Tile::Character7, false)),
            ],
            Tile::Dot3,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::TileHog);
    }

    // ── 65. Double Pung ──

    #[test]
    fn test_double_pung() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Character2, true)),
                Meld::Pung(Triplet::new(Tile::Character5, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo5, true)),
                Meld::Pung(Triplet::new(Tile::Dot8, true)),
            ],
            Tile::Dot4,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::DoublePung);
    }

    // ── 66. Two Concealed Pungs ──

    #[test]
    fn test_two_concealed_pungs() {
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Bamboo9, true)),
                Meld::Kong(Quad::new(Tile::Dot9, true)),
                Meld::Chow(Sequence::new(Tile::Character7, false)),
                Meld::Chow(Sequence::new(Tile::Character1, false)),
            ],
            Tile::Character5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::TwoConcealedPungs);
    }

    // ── 67. Concealed Kong ──

    #[test]
    fn test_concealed_kong() {
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Bamboo9, true)),
                Meld::Pung(Triplet::new(Tile::Dot9, false)),
                Meld::Chow(Sequence::new(Tile::Character7, false)),
                Meld::Chow(Sequence::new(Tile::Character1, false)),
            ],
            Tile::Character5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ConcealedKong);
    }

    // ── 68. All Simples ──

    #[test]
    fn test_all_simples() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character2, true)),
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo7, true)),
            ],
            Tile::Dot4,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::AllSimples);
    }

    // ═══════════════════════════════════════════════════════════════
    // 1-point fans
    // ═══════════════════════════════════════════════════════════════

    // ── 69. Pure Double Chow ──

    #[test]
    fn test_pure_double_chow() {
        // Two identical chows in same suit: Wan1-2-3 ×2
        // Plus Wan7-8-9 and a Dot8 pung, pair Dot9
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Pung(Triplet::new(Tile::Dot8, true)),
            ],
            Tile::Dot9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::PureDoubleChow);
    }

    // ── 70. Mixed Double Chow ──

    #[test]
    fn test_mixed_double_chow() {
        // Two chows of same numbers in different suits: Wan1-2-3, Dot1-2-3
        // Plus Wan7-8-9 and a Dot8 pung, pair Dot9
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Dot1, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Pung(Triplet::new(Tile::Dot8, true)),
            ],
            Tile::Dot9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MixedDoubleChow);
    }

    // ── 71. Short Straight ──

    #[test]
    fn test_short_straight() {
        // Two chows in same suit forming 1-2-3 + 4-5-6
        // Plus Wan7-8-9 and a Dot8 pung, pair Dot9
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Pung(Triplet::new(Tile::Dot8, true)),
            ],
            Tile::Dot9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ShortStraight);
    }

    // ── 72. Two Terminal Chows ──

    #[test]
    fn test_two_terminal_chows() {
        // Chows of 1-2-3 and 7-8-9 in same suit (Character)
        // Plus Wan4-5-6 and a Dot8 pung, pair Dot9
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Pung(Triplet::new(Tile::Dot8, true)),
            ],
            Tile::Dot9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::TwoTerminalChows);
    }

    // ── 72. Two Terminal Chows ──

    #[test]
    fn test_two_terminal_chows_example2() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Pung(Triplet::new(Tile::Dot8, true)),
            ],
            Tile::Dot9,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::TwoTerminalChows);
    }

    // ── 73. Pung of Terminals or Honors ──

    #[test]
    fn test_pung_of_terminals_or_honors() {
        let hand = concealed_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::Character1, true)),
                Meld::Pung(Triplet::new(Tile::East, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
            ],
            Tile::Character5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::PungOfTerminalsOrHonors);
    }

    // ── 74. Melded Kong ──

    #[test]
    fn test_melded_kong() {
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Dot1, false)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, false)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, false)),
                Meld::Chow(Sequence::new(Tile::Dot4, false)),
            ],
            Tile::Bamboo2,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::MeldedKong);
    }

    // ── 75. One Voided Suit ──

    #[test]
    fn test_one_voided_suit() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Pung(Triplet::new(Tile::East, true)),
            ],
            Tile::Character5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::OneVoidedSuit);
    }

    // ── 76. No Honors ──
    //
    // Full Flush excludes No Honors per MCR rules, so we use a hand
    // with all three suits (so Full Flush doesn't apply) and no honors.

    #[test]
    fn test_no_honors() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character2, true)),
                Meld::Chow(Sequence::new(Tile::Dot2, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo2, true)),
                Meld::Pung(Triplet::new(Tile::Bamboo5, true)),
            ],
            Tile::Character5,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::NoHonors);
    }

    // ── 77. Edge Wait ──

    #[test]
    fn test_edge_wait() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Chow(Sequence::new(Tile::Dot2, true)),
            ],
            Tile::Bamboo3,
        );
        let static_ctx = default_static_ctx();
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::EdgeWait);
    }

    // ── 78. Closed Wait ──

    #[test]
    fn test_closed_wait() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character2, true)),
                Meld::Chow(Sequence::new(Tile::Dot3, true)),
                Meld::Chow(Sequence::new(Tile::Dot6, true)),
                Meld::Chow(Sequence::new(Tile::Bamboo6, true)),
            ],
            Tile::Bamboo4,
        );
        let static_ctx = default_static_ctx();
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::ClosedWait);
    }

    // ── 79. Single Wait ──

    #[test]
    fn test_single_wait() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Pung(Triplet::new(Tile::East, true)),
            ],
            Tile::Character5,
        );
        let static_ctx = default_static_ctx();
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::SingleWait);
    }

    // ── 80. Self-Drawn ──

    #[test]
    fn test_self_drawn() {
        let hand = all_declared_hand(
            vec![
                Meld::Kong(Quad::new(Tile::Dot1, false)),
                Meld::Chow(Sequence::new(Tile::Bamboo4, false)),
                Meld::Chow(Sequence::new(Tile::Bamboo7, false)),
                Meld::Chow(Sequence::new(Tile::Dot4, false)),
            ],
            Tile::Bamboo2,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.win_method = WinMethod::SelfDraw;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::SelfDrawn);
    }

    // ── 81. Flower Tiles ──

    #[test]
    fn test_flower_tiles() {
        let hand = concealed_hand(
            vec![
                Meld::Chow(Sequence::new(Tile::Character1, true)),
                Meld::Chow(Sequence::new(Tile::Character4, true)),
                Meld::Chow(Sequence::new(Tile::Character7, true)),
                Meld::Chow(Sequence::new(Tile::Character1, true)),
            ],
            Tile::Character5,
        );
        let mut static_ctx = default_static_ctx();
        static_ctx.flower_count = 2;
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::FlowerTiles);
    }

    // ═══════════════════════════════════════════════════════════════
    // Exclusion rule integration tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_big_four_winds_excludes_big_three_winds() {
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
        assert_not_contains_fan(&result, FanType::BigThreeWinds);
    }

    #[test]
    fn test_big_three_dragons_excludes_dragon_pung() {
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
        assert_not_contains_fan(&result, FanType::DragonPung);
        assert_not_contains_fan(&result, FanType::TwoDragonPungs);
    }

    #[test]
    fn test_four_concealed_pungs_excludes_all_pungs() {
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
    }

    // ═══════════════════════════════════════════════════════════════
    // Non-winning hand tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_non_winning_hand_returns_none() {
        // 4 declared melds, but concealed has only 1 tile (not a pair)
        let mut hand = Hand::default();
        hand.melds.push(Meld::Pung(Triplet::new(Tile::East, false)));
        hand.melds
            .push(Meld::Pung(Triplet::new(Tile::South, false)));
        hand.melds.push(Meld::Pung(Triplet::new(Tile::West, false)));
        hand.melds
            .push(Meld::Pung(Triplet::new(Tile::North, false)));
        hand.concealed.insert(Tile::Character1); // only 1 tile — not a pair
        let static_ctx = default_static_ctx();
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        assert!(results.is_empty(), "non-winning hand should return empty");
    }

    #[test]
    fn test_empty_hand_returns_none() {
        let hand = Hand::default();
        let static_ctx = default_static_ctx();
        let dynamic_ctx = default_dynamic_ctx();
        let results = solve_fan(&hand, &static_ctx, &dynamic_ctx).unwrap();
        assert!(results.is_empty(), "empty hand should return empty");
    }

    // ═══════════════════════════════════════════════════════════════
    // Declared meld hand test (uses all_declared_hand for explicit melds)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_big_four_winds_with_declared_melds() {
        // Same hand as example 1 but with all 4 pungs declared (exposed).
        // This validates that declared melds also work with the fan rules.
        let hand = all_declared_hand(
            vec![
                Meld::Pung(Triplet::new(Tile::East, false)),
                Meld::Pung(Triplet::new(Tile::South, false)),
                Meld::Pung(Triplet::new(Tile::West, false)),
                Meld::Pung(Triplet::new(Tile::North, false)),
            ],
            Tile::Red,
        );
        let results = solve_default(&hand);
        let result = results.first().unwrap();
        assert_contains_fan(&result, FanType::BigFourWinds);
    }
}
