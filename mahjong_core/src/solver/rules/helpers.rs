// ═══════════════════════════════════════════════════════════════
// Shared helpers for MCR rule checking
//
// Provides:
//   - MeldInfo / PairInfo — flat view of a meld/pair with pre-extracted fields
//   - meld_info(), pair_info() — converters from Meld/Pair
//   - Tile attribute helpers — rank_of, suit_of, is_terminal_tile, etc.
//   - Meld predicate helpers — is_pung_or_kong, is_chow, is_melded_kong
//   - cand() — FanCandidate factory
// ═══════════════════════════════════════════════════════════════

use super::{FanCandidate, FanType};
use crate::structs::{Meld, Pair, Tile};

// ── Meld metadata types ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeldKind {
    Chow,
    Pung,
    Kong,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MeldInfo {
    pub kind: MeldKind,
    pub tile: Tile,
    /// For chows: the rank of the start tile (1-7).
    /// For pungs/kongs: the rank of the tile (1-9), 0 for honors.
    pub rank: u8,
    /// 0=Characters, 1=Dots, 2=Bamboos, 3=Honors.
    pub suit: u8,
    pub is_concealed: bool,
    pub is_terminal: bool,
    pub is_honor: bool,
    pub is_dragon: bool,
    pub is_wind: bool,
    /// The constituent tiles (first 3 valid, 4 for kong, rest sentinel).
    pub tiles: [Tile; 4],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PairInfo {
    pub tile: Tile,
    pub rank: u8,
    pub suit: u8,
    pub is_terminal: bool,
    pub is_honor: bool,
    pub is_dragon: bool,
    pub is_wind: bool,
}

// ── Tile attribute helpers ─────────────────────────────────────

#[inline]
pub(crate) fn rank_of(t: Tile) -> u8 {
    let v = t as u8;
    match v {
        0..=32 => v / 4 + 1,
        64..=96 => (v - 64) / 4 + 1,
        128..=160 => (v - 128) / 4 + 1,
        _ => 0,
    }
}

#[inline]
pub(crate) fn suit_of(t: Tile) -> u8 {
    match t as u8 {
        0..=32 => 0,
        64..=96 => 1,
        128..=160 => 2,
        _ => 3,
    }
}

#[inline]
pub(crate) fn is_terminal_tile(t: Tile) -> bool {
    matches!(rank_of(t), 1 | 9)
}

#[inline]
pub(crate) fn is_honor_tile(t: Tile) -> bool {
    suit_of(t) == 3
}

#[inline]
pub(crate) fn is_dragon_tile(t: Tile) -> bool {
    matches!(t, Tile::Red | Tile::Green | Tile::White)
}

#[inline]
pub(crate) fn is_wind_tile(t: Tile) -> bool {
    matches!(t, Tile::East | Tile::South | Tile::West | Tile::North)
}

#[inline]
pub(crate) fn is_green_tile(t: Tile) -> bool {
    matches!(
        t,
        Tile::Bamboo2
            | Tile::Bamboo3
            | Tile::Bamboo4
            | Tile::Bamboo6
            | Tile::Bamboo8
            | Tile::Green
    )
}

#[inline]
pub(crate) fn is_reversible_tile(t: Tile) -> bool {
    matches!(
        t,
        Tile::Dot1
            | Tile::Dot2
            | Tile::Dot3
            | Tile::Dot4
            | Tile::Dot5
            | Tile::Dot8
            | Tile::Dot9
            | Tile::Bamboo2
            | Tile::Bamboo4
            | Tile::Bamboo5
            | Tile::Bamboo6
            | Tile::Bamboo8
            | Tile::Bamboo9
            | Tile::White
    )
}

#[inline]
pub(crate) fn is_even_rank(t: Tile) -> bool {
    let r = rank_of(t);
    r > 0 && r.is_multiple_of(2)
}

// ── Meld/Pair converters ───────────────────────────────────────

fn chow_tiles(start: Tile) -> [Tile; 4] {
    let idx = start as u8;
    [
        start,
        // SAFETY: Tile enum has 4-bit gaps per suit; idx+4 moves to next rank.
        unsafe { std::mem::transmute::<u8, Tile>(idx + 4) },
        unsafe { std::mem::transmute::<u8, Tile>(idx + 8) },
        start,
    ]
}

pub(crate) fn meld_info(m: &Meld) -> MeldInfo {
    match *m {
        Meld::Pung(p) => {
            let tile = p.tile();
            let r = rank_of(tile);
            let s = suit_of(tile);
            MeldInfo {
                kind: MeldKind::Pung,
                tile,
                rank: r,
                suit: s,
                is_concealed: p.is_concealed(),
                is_terminal: r == 1 || r == 9,
                is_honor: s == 3,
                is_dragon: is_dragon_tile(tile),
                is_wind: is_wind_tile(tile),
                tiles: [tile, tile, tile, tile],
            }
        }
        Meld::Chow(s) => {
            let start = s.start();
            let r = rank_of(start);
            let suit = suit_of(start);
            MeldInfo {
                kind: MeldKind::Chow,
                tile: start,
                rank: r,
                suit,
                is_concealed: s.is_concealed(),
                is_terminal: r == 1 || r == 7,
                is_honor: false,
                is_dragon: false,
                is_wind: false,
                tiles: chow_tiles(start),
            }
        }
        Meld::Kong(q) => {
            let tile = q.tile();
            let r = rank_of(tile);
            let s = suit_of(tile);
            MeldInfo {
                kind: MeldKind::Kong,
                tile,
                rank: r,
                suit: s,
                is_concealed: q.is_concealed(),
                is_terminal: r == 1 || r == 9,
                is_honor: s == 3,
                is_dragon: is_dragon_tile(tile),
                is_wind: is_wind_tile(tile),
                tiles: [tile, tile, tile, tile],
            }
        }
    }
}

pub(crate) fn pair_info(p: &Pair) -> PairInfo {
    let tile = p.tile();
    let r = rank_of(tile);
    let s = suit_of(tile);
    PairInfo {
        tile,
        rank: r,
        suit: s,
        is_terminal: r == 1 || r == 9,
        is_honor: s == 3,
        is_dragon: is_dragon_tile(tile),
        is_wind: is_wind_tile(tile),
    }
}

// ── Meld predicates (operate on `&Meld`, call `meld_info` internally) ──

#[inline]
pub(crate) fn is_melded_kong(m: &Meld) -> bool {
    let mi = meld_info(m);
    matches!(mi.kind, MeldKind::Kong) && !mi.is_concealed
}

#[inline]
pub(crate) fn is_pung_or_kong(m: &Meld) -> bool {
    matches!(meld_info(m).kind, MeldKind::Pung | MeldKind::Kong)
}

#[inline]
pub(crate) fn is_chow(m: &Meld) -> bool {
    matches!(meld_info(m).kind, MeldKind::Chow)
}

// ── FanCandidate factory ───────────────────────────────────────

/// Check if all melds and the pair satisfy a rank range.
pub(crate) fn all_in_range(sets: &[Meld], pair: &Pair, lo: u8, hi: u8) -> bool {
    sets.iter().all(|m| {
        let r = meld_info(m).rank;
        lo <= r && r <= hi
    }) && {
        let r = pair_info(pair).rank;
        r > 0 && lo <= r && r <= hi
    }
}

/// Create a FanCandidate with score set from FanType points.
#[inline]
pub(crate) fn cand(ft: FanType, mask: u64, uses_pair: bool) -> FanCandidate {
    FanCandidate {
        fan_type: ft,
        used_set_mask: mask,
        uses_pair,
        score: ft.points(),
        excludes_mask: Default::default(),
    }
}
