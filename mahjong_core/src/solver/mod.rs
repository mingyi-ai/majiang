mod decompose_special;
mod decompose_standard;
mod decomposition;
pub(crate) mod fan;
mod fan_context;
pub(crate) mod rules;
pub(crate) mod solve;
#[cfg(test)]
mod mcr_tests;
mod types;

pub(crate) use decomposition::Decomposition;
pub(crate) use fan_context::FanContext;
pub use solve::solve_fan;
pub(crate) use types::FanSolveResult;

use crate::array_vec::ArrayVec;
use crate::structs::{Hand, Pair};

// ═══════════════════════════════════════════════════════════════
// Solver error type
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverError {
    #[allow(dead_code)]
    InvalidTileCount {
        declared_melds: usize,
        concealed: usize,
        total: usize,
        expected_concealed: usize,
    },
    TooManyMelds {
        declared: usize,
    },
}

// ═══════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════

/// Check if the full hand forms a winning hand.
pub(crate) fn is_hu(hand: &Hand) -> Result<bool, SolverError> {
    let n_declared = hand.melds.len();
    if n_declared > 4 {
        return Err(SolverError::TooManyMelds {
            declared: n_declared,
        });
    }

    let n_sets_needed = 4 - n_declared;
    for concealed in decompose_standard::decompose_standard(&hand.concealed) {
        if concealed.melds.len() == n_sets_needed {
            return Ok(true);
        }
    }

    if hand.melds.is_empty() {
        if decompose_special::detect_seven_pairs(&hand.concealed).is_some()
            || decompose_special::detect_thirteen_orphans(&hand.concealed)
                .is_some()
        {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Enumerate all valid full-hand decompositions for fan scoring.
#[allow(dead_code)]
pub(crate) fn decompose_hand(
    hand: &Hand,
) -> Result<Vec<Decomposition>, SolverError> {
    let n_declared = hand.melds.len();
    if n_declared > 4 {
        return Err(SolverError::TooManyMelds {
            declared: n_declared,
        });
    }

    let mut result: Vec<Decomposition> = Vec::new();

    for concealed in decompose_standard::decompose_standard(&hand.concealed) {
        let mut sets = ArrayVec::new();
        for i in 0..hand.melds.len() {
            sets.push(hand.melds[i]);
        }
        for i in 0..concealed.melds.len() {
            sets.push(concealed.melds[i]);
        }
        result.push(Decomposition::Standard {
            pair: Pair::new(concealed.pair_tile),
            sets,
        });
    }

    if hand.melds.is_empty() {
        if let Some(d) = decompose_special::detect_seven_pairs(&hand.concealed)
        {
            result.push(d);
        }
        if let Some(d) =
            decompose_special::detect_thirteen_orphans(&hand.concealed)
        {
            result.push(d);
        }
    }

    Ok(result)
}

/// Score a fully decomposed hand using the rule registry and search kernel.
///
/// Returns all max-score solutions (multiple in case of ties).
#[allow(dead_code)]
pub(crate) fn score_decomposition(
    decomp: &Decomposition,
    ctx: &FanContext,
) -> Vec<FanSolveResult> {
    fan::score_hand(decomp, ctx)
}
