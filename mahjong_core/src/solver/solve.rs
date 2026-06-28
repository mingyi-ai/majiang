// ═══════════════════════════════════════════════════════════════
// solve_fan — highest-scoring fan result for a winning hand
// ═══════════════════════════════════════════════════════════════

use super::FanContext;
use super::FanSolveResult;
use super::{SolverError, decompose_hand, score_decomposition};
use crate::structs::Hand;

/// Find the highest-scoring fan result for a given hand and context.
///
/// Iterates all valid decompositions, scores each via the fan search
/// kernel, and returns the single best result (highest total score).
/// Returns `None` if the hand is not a winning hand or cannot be
/// decomposed.
pub fn solve_fan(
    hand: &Hand,
    ctx: &FanContext,
) -> Result<Option<FanSolveResult>, SolverError> {
    let decompositions = decompose_hand(hand)?;
    if decompositions.is_empty() {
        return Ok(None);
    }

    let mut best: Option<FanSolveResult> = None;
    for decomp in &decompositions {
        let results = score_decomposition(decomp, ctx);
        for r in results {
            let is_better = match &best {
                None => true,
                Some(b) => r.total_score > b.total_score,
            };
            if is_better {
                best = Some(r);
            }
        }
    }
    Ok(best)
}
