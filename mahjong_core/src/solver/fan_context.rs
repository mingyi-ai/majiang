use crate::structs::{Tile, TileType, Wind};

/// How the winning tile was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinMethod {
    SelfDraw,
    Discard,
    KongReplacement,
    RobKong,
}

/// The type of wait before the winning tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitType {
    /// None / not applicable (multiple winning tiles possible).
    Multiple,
    /// Waiting for 3 to complete 1-2-3, or 7 to complete 7-8-9.
    Edge,
    /// Waiting for a tile in the middle of a chow (e.g., 4 for 3-4-5).
    Closed,
    /// Waiting for a single tile to complete the pair.
    Single,
}

/// Game-state context needed for fan rule evaluation.
///
/// Many MCR fan types depend on how the hand was won, the player's
/// position, and other transition-dependent information that cannot
/// be derived from the tile decomposition alone.
#[derive(Debug, Clone)]
pub struct FanContext {
    pub seat_wind: Wind,
    pub prevalent_wind: Wind,
    pub win_method: WinMethod,
    pub winning_tile: Tile,
    pub wait_type: WaitType,
    pub flower_count: u8,
    pub is_concealed: bool,
    pub is_fully_concealed: bool,
    pub is_last_tile_draw: bool,
    pub is_last_tile_claim: bool,
    pub is_last_tile_of_kind: bool,
    pub wall_remaining: usize,
}

impl FanContext {
    pub fn prevalent_wind_to_tile(&self) -> Tile {
        wind_to_tile(self.prevalent_wind)
    }

    pub fn seat_wind_to_tile(&self) -> Tile {
        wind_to_tile(self.seat_wind)
    }
}

fn wind_to_tile(w: Wind) -> Tile {
    match w {
        Wind::East => Tile::East,
        Wind::South => Tile::South,
        Wind::West => Tile::West,
        Wind::North => Tile::North,
    }
}
