mod decompose_special;
mod decompose_standard;
mod fan_solver;
mod rules;

use crate::{
    array_vec::ArrayVec,
    solver::rules::FanType,
    structs::{Hand, Meld, Pair, Tile, Wind},
};

#[derive(Debug)]
pub enum SolverError {
    InvalidHand,
}

/// How the winning tile was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinMethod {
    SelfDraw,
    Discard,
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

#[derive(Debug, Clone, Copy)]
pub struct StaticFanContext {
    pub seat_wind: Wind,
    pub prevalent_wind: Wind,
    pub win_method: WinMethod,
    pub winning_tile: Tile,
    pub flower_count: u8,
    pub is_concealed: bool,
    pub is_fully_concealed: bool,
    pub is_last_tile_draw: bool,
    pub is_last_tile_claim: bool,
    pub is_last_tile_of_kind: bool,
    pub wall_remaining: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct DynamicFanContext {
    pub is_kong_replacement: bool,
    pub is_rob_kong: bool,
}

/// A selected fan instance in the result.
#[derive(Debug, Clone)]
pub struct FanInstance {
    pub fan_type: FanType,
    pub(super) used_set_mask: u64,
    pub(super) uses_pair: bool,
}

/// Result of a fan search.
#[derive(Debug, Clone, Default)]
pub struct FanResult {
    pub fans: Vec<FanInstance>,
}

impl FanResult {
    pub fn total_score(&self) -> u8 {
        self.fans.iter().map(|f| f.fan_type.points()).sum::<u8>()
    }
}

pub fn solve_fan(
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
    match decompose_hand(hand) {
        Ok(mut iter) => Ok(iter.next().is_some()),
        Err(_) => Ok(false),
    }
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
    hand: &Hand,
) -> Result<impl Iterator<Item = DecomposeResult>, SolverError> {
    let counts = &hand.concealed;
    let declared = &hand.melds;
    let n_declared = declared.len();

    // ── Standard decompositions (concealed tiles only) ──
    let standard = decompose_standard::decompose_standard(counts);

    // ── Special hand detection ──
    let seven_pairs = decompose_special::detect_seven_pairs(counts);
    let thirteen_orphans = decompose_special::detect_thirteen_orphans(counts);

    // ── Build DecomposeResult iterator ──
    let mut results: Vec<DecomposeResult> = Vec::new();

    for concealed in &standard {
        let mut sets = ArrayVec::new();
        // Declared melds come first (lower indices for used_set_mask)
        for i in 0..n_declared {
            sets.push(declared[i]);
        }
        for i in 0..concealed.melds.len() {
            sets.push(concealed.melds[i]);
        }
        let pair = Pair::new(concealed.pair_tile);
        results.push(DecomposeResult {
            decompositions: Decomposition::Standard { pair, sets },
            wait_type: stub_wait_type(),
        });
    }

    if let Some(decomp) = seven_pairs {
        results.push(DecomposeResult {
            decompositions: decomp,
            wait_type: stub_wait_type(),
        });
    }
    if let Some(decomp) = thirteen_orphans {
        results.push(DecomposeResult {
            decompositions: decomp,
            wait_type: stub_wait_type(),
        });
    }

    Ok(results.into_iter())
}

/// Stub for wait-type computation. Returns Multiple until
/// the wait-type detection algorithm is implemented.
fn stub_wait_type() -> WaitType {
    WaitType::Multiple
}

/// Score a fully decomposed hand using the rule registry and search kernel.
/// Returns all max-score solutions (multiple in case of ties).
fn score_decomposition(
    decomp: &DecomposeResult,
    static_ctx: &StaticFanContext,
    dynamic_ctx: &DynamicFanContext,
) -> Vec<FanResult> {
    // Run all rules via the macro-generated registry
    let candidates =
        rules::check_all(decomp, static_ctx, dynamic_ctx, decomp.wait_type);

    // Run the search kernel to find the max-score compatible subset
    let solve_results = fan_solver::solve_max_score(candidates);

    // Convert FanSolveResult → FanResult
    solve_results
        .into_iter()
        .map(|sr| FanResult { fans: sr.fans })
        .collect()
}
