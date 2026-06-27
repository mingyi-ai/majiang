// ═══════════════════════════════════════════════════════════════
// HandProfile — pre-computed decomposition view
//
// Converts a Decomposition into a flat, field-accessible form so
// that every rule can access meld/pair properties without
// pattern-matching through the Decomposition tree.
//
// Building a HandProfile is O(n_sets) — done once per decomposition.
// Without it, each of the 81 rules independently re-parses the same
// Meld variants to extract rank, suit, kind, etc.
// ═══════════════════════════════════════════════════════════════

use super::decomposition::Decomposition;
use crate::array_vec::ArrayVec;
use crate::structs::{Meld, Pair, Tile};

// ── Kinds ──────────────────────────────────────────────────────

/// Which form of hand this profile represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProfileKind {
    Standard,
    SevenPairs,
    ThirteenOrphans,
}

// ── Meld info ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
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
    /// The constituent tiles (first 3 valid for pung/chow, 4 for kong, rest sentinel).
    /// For chows: sequential tiles; for pungs/kongs: copies of the tile.
    pub tiles: [Tile; 4],
}

// ── Pair info ──────────────────────────────────────────────────

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

// ── HandProfile ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct HandProfile {
    pub kind: ProfileKind,
    /// For Standard: the 4 melds (n_sets indicates how many are valid).
    pub melds: [MeldInfo; 4],
    /// How many melds are actually present (1-4 for Standard).
    pub n_sets: u8,
    /// Pair information.
    pub pair: PairInfo,
    /// For SevenPairs: the 7 pair tiles.
    pub pair_tiles: [Tile; 7],
    pub all_pungs: bool,
    pub all_chows: bool,
    pub n_kongs: u8,
    pub has_honors: bool,
}

// ── Helpers ────────────────────────────────────────────────────

pub(crate) fn rank_of(t: Tile) -> u8 {
    let v = t as u8;
    match v {
        0..=32 => v / 4 + 1,
        64..=96 => (v - 64) / 4 + 1,
        128..=160 => (v - 128) / 4 + 1,
        _ => 0, // honors have rank 0
    }
}

pub(crate) fn suit_of(t: Tile) -> u8 {
    match t as u8 {
        0..=32 => 0,
        64..=96 => 1,
        128..=160 => 2,
        _ => 3,
    }
}

pub(crate) fn is_terminal_tile(t: Tile) -> bool {
    let r = rank_of(t);
    r == 1 || r == 9
}

pub(crate) fn is_honor_tile(t: Tile) -> bool {
    suit_of(t) == 3
}

pub(crate) fn is_dragon_tile(t: Tile) -> bool {
    matches!(t, Tile::Red | Tile::Green | Tile::White)
}

pub(crate) fn is_wind_tile(t: Tile) -> bool {
    matches!(t, Tile::East | Tile::South | Tile::West | Tile::North)
}

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

pub(crate) fn is_even_rank(t: Tile) -> bool {
    let r = rank_of(t);
    r > 0 && r.is_multiple_of(2)
}

// ── MeldInfo ───────────────────────────────────────────────────

fn chow_tiles(start: Tile) -> [Tile; 4] {
    let idx = start as u8;
    [
        start,
        unsafe { std::mem::transmute::<u8, Tile>(idx + 4) },
        unsafe { std::mem::transmute::<u8, Tile>(idx + 8) },
        start,
    ] // sentinel
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

/// Build PairInfo from a Pair reference.
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

// ── HandProfile construction ───────────────────────────────────

const EMPTY_MELD: MeldInfo = MeldInfo {
    kind: MeldKind::Pung,
    tile: Tile::Character1,
    rank: 0,
    suit: 0,
    is_concealed: false,
    is_terminal: false,
    is_honor: false,
    is_dragon: false,
    is_wind: false,
    tiles: [Tile::Character1; 4],
};

const EMPTY_PAIR: PairInfo = PairInfo {
    tile: Tile::Character1,
    rank: 0,
    suit: 0,
    is_terminal: false,
    is_honor: false,
    is_dragon: false,
    is_wind: false,
};

fn profile_standard(sets: &ArrayVec<Meld, 4>, pair: &Pair) -> HandProfile {
    let n = sets.len() as u8;
    let mut melds = [EMPTY_MELD; 4];
    let mut n_kongs: u8 = 0;
    let mut has_honors = false;

    for (i, m) in sets.iter().enumerate() {
        let info = meld_info(m);
        if info.is_honor {
            has_honors = true;
        }
        if matches!(info.kind, MeldKind::Kong) {
            n_kongs += 1;
        }
        melds[i] = info;
    }

    let pi = pair_info(pair);
    if pi.is_honor {
        has_honors = true;
    }

    let all_pungs = n > 0
        && melds
            .iter()
            .take(n as usize)
            .all(|m| matches!(m.kind, MeldKind::Pung | MeldKind::Kong));
    let all_chows = n > 0
        && melds
            .iter()
            .take(n as usize)
            .all(|m| matches!(m.kind, MeldKind::Chow));

    HandProfile {
        kind: ProfileKind::Standard,
        melds,
        n_sets: n,
        pair: pi,
        pair_tiles: [Tile::Character1; 7],
        all_pungs,
        all_chows,
        n_kongs,
        has_honors,
    }
}

fn profile_seven_pairs(pairs: &[Pair; 7]) -> HandProfile {
    let mut pair_tiles = [Tile::Character1; 7];
    for (i, p) in pairs.iter().enumerate() {
        pair_tiles[i] = p.tile();
    }

    HandProfile {
        kind: ProfileKind::SevenPairs,
        melds: [EMPTY_MELD; 4],
        n_sets: 0,
        pair: EMPTY_PAIR,
        pair_tiles,
        all_pungs: false,
        all_chows: false,
        n_kongs: 0,
        has_honors: pairs.iter().any(|p| is_honor_tile(p.tile())),
    }
}

fn empty_standard() -> HandProfile {
    HandProfile {
        kind: ProfileKind::Standard,
        melds: [EMPTY_MELD; 4],
        n_sets: 0,
        pair: EMPTY_PAIR,
        pair_tiles: [Tile::Character1; 7],
        all_pungs: false,
        all_chows: false,
        n_kongs: 0,
        has_honors: false,
    }
}

// ── Public API ─────────────────────────────────────────────────

impl HandProfile {
    pub(crate) fn from_decomposition(decomp: &Decomposition) -> Self {
        match decomp {
            Decomposition::Standard { pair, sets } => {
                profile_standard(sets, pair)
            }
            Decomposition::SevenPairs { pairs } => profile_seven_pairs(pairs),
            Decomposition::ThirteenOrphans { pair } => {
                let mut p = empty_standard();
                p.kind = ProfileKind::ThirteenOrphans;
                p.pair = pair_info(pair);
                p
            }
        }
    }

    pub(crate) fn is_standard(&self) -> bool {
        self.kind == ProfileKind::Standard
    }

    /// Check whether all tiles in this profile are suit tiles
    /// (no honors), for rules like NoHonors.
    pub(crate) fn all_suit_tiles(&self) -> bool {
        !self.has_honors
    }
}
