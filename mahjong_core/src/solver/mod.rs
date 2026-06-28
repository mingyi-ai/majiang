// mod decompose_special;
// mod decompose_standard;
mod decomposition;
// pub(crate) mod rules;
// pub(crate) mod search;
// pub(crate) mod solve;
mod types;
// mod view;

pub(crate) use decomposition::Decomposition;

use crate::{
    solver::types::FanType,
    structs::{Hand, Tile, Wind},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverError {
    InvalidHand,
}

/// How the winning tile was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WinMethod {
    SelfDraw,
    Discard,
}

/// The type of wait before the winning tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WaitType {
    /// None / not applicable (multiple winning tiles possible).
    Multiple,
    /// Waiting for 3 to complete 1-2-3, or 7 to complete 7-8-9.
    Edge,
    /// Waiting for a tile in the middle of a chow (e.g., 4 for 3-4-5).
    Closed,
    /// Waiting for a single tile to complete the pair.
    Single,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StaticFanContext {
    pub(crate) seat_wind: Wind,
    pub(crate) prevalent_wind: Wind,
    pub(crate) win_method: WinMethod,
    pub(crate) winning_tile: Tile,
    pub(crate) flower_count: u8,
    pub(crate) is_concealed: bool,
    pub(crate) is_fully_concealed: bool,
    pub(crate) is_last_tile_draw: bool,
    pub(crate) is_last_tile_claim: bool,
    pub(crate) is_last_tile_of_kind: bool,
    pub(crate) wall_remaining: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicFanContext {
    pub(crate) is_kong_replacement: bool,
    pub(crate) is_rob_kong: bool,
}

pub(crate) struct DecomposeResult {
    pub(crate) decompositions: Decomposition,
    pub(crate) wait_type: WaitType,
}

/// A selected fan instance in the result.
#[derive(Debug, Clone)]
pub struct FanInstance {
    pub fan_type: FanType,
    pub score: u8,
    pub(crate) used_set_mask: u64,
    pub(crate) uses_pair: bool,
}

/// Result of a fan search.
#[derive(Debug, Clone, Default)]
pub struct FanResult {
    pub total_score: u16,
    pub fans: Vec<FanInstance>,
}

pub(crate) fn solve_fan(
    hand: &Hand,
    static_ctx: &StaticFanContext,
    dynamic_ctx: &DynamicFanContext,
) -> Result<Vec<FanResult>, SolverError> {
    let decompositions = decompose_hand(hand)?;
    let mut results: Vec<FanResult> = Vec::new();
    for decomp in decompositions.iter() {
        let mut scored = score_decomposition(decomp, static_ctx, dynamic_ctx);
        results.append(&mut scored);
    }
    Ok(keep_highest_score(results))
}

fn keep_highest_score(_results: Vec<FanResult>) -> Vec<FanResult> {
    unimplemented!()
}

/// Compatibility stub for the old `is_hu` function.
pub(crate) fn is_hu(hand: &Hand) -> Result<bool, SolverError> {
    let decompositions = decompose_hand(hand)?;
    Ok(!decompositions.is_empty())
}

/// Enumerate all valid full-hand decompositions for fan scoring.
pub(crate) fn decompose_hand(
    _hand: &Hand,
) -> Result<Vec<DecomposeResult>, SolverError> {
    unimplemented!()
}

/// Score a fully decomposed hand using the rule registry and search kernel.
///
/// Returns all max-score solutions (multiple in case of ties).
pub(crate) fn score_decomposition(
    _decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
) -> Vec<FanResult> {
    unimplemented!()
}
