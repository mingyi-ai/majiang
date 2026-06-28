// mod decompose_special;
// mod decompose_standard;
mod rules;
// pub(crate) mod fan_solver;
// mod view;

use crate::{
    array_vec::ArrayVec,
    solver::rules::FanType,
    structs::{Hand, Meld, Pair, Tile, Wind},
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
    pub fans: Vec<FanInstance>,
}

impl FanResult {
    pub fn total_score(&self) -> u8 {
        self.fans.iter().map(|f| f.score).sum::<u8>()
    }
}

pub(crate) fn solve_fan(
    hand: &Hand,
    static_ctx: &StaticFanContext,
    dynamic_ctx: &DynamicFanContext,
) -> Result<Vec<FanResult>, SolverError> {
    let results = decompose_hand(hand)?.flat_map(|decomp| {
        score_decomposition(&decomp, static_ctx, dynamic_ctx)
    });
    Ok(keep_highest_score(results))
}

fn keep_highest_score(
    results: impl Iterator<Item = FanResult>,
) -> Vec<FanResult> {
    results
        .fold(None, |best: Option<(Vec<FanResult>, u8)>, candidate| {
            let score = candidate.total_score();
            match best {
                None => Some((vec![candidate], score)),
                Some((mut best_fans, best_score)) => {
                    match score.cmp(&best_score) {
                        std::cmp::Ordering::Greater => {
                            Some((vec![candidate], score))
                        }
                        std::cmp::Ordering::Equal => {
                            best_fans.push(candidate);
                            Some((best_fans, best_score))
                        }
                        std::cmp::Ordering::Less => {
                            Some((best_fans, best_score))
                        }
                    }
                }
            }
        })
        .map_or(Vec::new(), |(fans, _)| fans)
}

/// Compatibility stub for the old `is_hu` function.
pub(crate) fn is_hu(hand: &Hand) -> Result<bool, SolverError> {
    let mut decompositions = decompose_hand(hand)?;
    Ok(decompositions.next().is_some())
}

pub(crate) enum Decomposition {
    /// 4 sets + 1 pair (or fewer sets with declared melds).
    Standard {
        pair: Pair,
        sets: ArrayVec<Meld, 4>,
    },
    SevenPairs {
        pairs: [Pair; 7],
    },
    ThirteenOrphans {
        pair: Pair,
    },
    // Knitted tiles...
}

pub(crate) struct DecomposeResult {
    pub(crate) decompositions: Decomposition,
    pub(crate) wait_type: WaitType,
}

/// Enumerate all valid full-hand decompositions for fan scoring.
fn decompose_hand(
    _hand: &Hand,
) -> Result<impl Iterator<Item = DecomposeResult>, SolverError> {
    Ok(std::iter::empty())
}

/// Score a fully decomposed hand using the rule registry and search kernel.
/// Returns all max-score solutions (multiple in case of ties).
fn score_decomposition(
    _decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
) -> Vec<FanResult> {
    unimplemented!()
}
