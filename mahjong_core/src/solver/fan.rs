// ═══════════════════════════════════════════════════════════════
// Scoring orchestration
//
// Coordinates the scoring pipeline for one decomposition:
//   1. Build HandProfile (pre-computed view)
//   2. Check all rules → candidates with exclusion lists
//   3. Embed exclusion masks into candidates
//   4. Search for max-score compatible subset
//   5. Apply Chicken Hand fallback
// ═══════════════════════════════════════════════════════════════

use super::decomposition::Decomposition;
use super::fan_context::FanContext;
use super::rules;
use super::search;
use super::types::{
    FanCandidate, FanExclusionSet, FanInstance, FanSolveResult, FanType,
};
use super::view::HandProfile;

/// Run the full scoring pipeline: build profile, extract candidates,
/// embed exclusion masks, search.
pub fn score_hand(
    decomp: &Decomposition,
    ctx: &FanContext,
) -> Vec<FanSolveResult> {
    let profile = HandProfile::from_decomposition(decomp);
    let raw = rules::check_all(&profile, ctx);

    let candidates: Vec<FanCandidate> = raw
        .into_iter()
        .map(|(mut c, excludes)| {
            let mut mask = FanExclusionSet::default();
            for &ft in excludes {
                mask.set(ft);
            }
            c.excludes_mask = mask;
            c
        })
        .collect();

    let mut results = search::solve_max_score(candidates);

    // Chicken Hand (43): if the hand would score 0 points (excluding
    // flower tiles), award 8 points.  The solver returns an empty
    // result when no candidates are found, which is the 0-score case.
    for r in &mut results {
        if r.total_score == 0 && r.fans.is_empty() {
            r.fans.push(FanInstance {
                fan_type: FanType::ChickenHand,
                used_set_mask: 0,
                uses_pair: false,
                score: 8,
            });
            r.total_score = 8;
        }
    }
    results
}
