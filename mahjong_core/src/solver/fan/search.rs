use super::fan_types::{
    FanCandidate, FanExclusionSet, FanInstance, FanSolveResult, FanType,
};

// ── Internal types ──

/// Compact signature for deduplication: two candidates with the same
/// (fan_type, uses_pair, used_set_mask) are considered the same fan
/// instance under MCR's non-repeat principle and cannot both be selected.
type SigKey = u64;

/// Preprocessed candidate instance for the search.
#[derive(Debug, Clone)]
struct SearchInstance {
    id: usize,
    fan_type: FanType,
    score: u8,
    used_set_mask: u64,
    uses_pair: bool,
    sig_key: SigKey,
    excludes_mask: FanExclusionSet,
}

/// A frame on the explicit DFS stack.
struct Frame {
    resume_pos: usize,
    max_allowed_score: u8,
    excluded_mask: FanExclusionSet,
    total_score: u16,
    path_len: usize,
}

// ── Search space construction ──

struct SearchSpace {
    instances: Vec<SearchInstance>,
    order: Vec<usize>, // sorted by (-score, id) for deterministic pruning
}

impl SearchSpace {
    fn build(candidates: Vec<FanCandidate>) -> Self {
        // SigKey layout: [discriminant (16 bits)][uses_pair (1 bit)][used_set_mask (16 bits)]
        let instances: Vec<SearchInstance> = candidates
            .into_iter()
            .enumerate()
            .map(|(id, c)| {
                let sig_key = ((c.fan_type as u16 as u64) << 17)
                    | ((c.uses_pair as u64) << 16)
                    | (c.used_set_mask & 0xFFFF);
                SearchInstance {
                    id,
                    fan_type: c.fan_type,
                    score: c.score,
                    used_set_mask: c.used_set_mask,
                    uses_pair: c.uses_pair,
                    sig_key,
                    excludes_mask: c.excludes_mask,
                }
            })
            .collect();

        // Sort by descending score, then ascending id for determinism.
        let mut order: Vec<usize> = (0..instances.len()).collect();
        order.sort_by_key(|&i| {
            let inst = &instances[i];
            ((inst.score as i16).wrapping_neg(), inst.id)
        });

        Self { instances, order }
    }
}

// ── Eligibility check ──

/// Returns true if `inst` can be added to the current selection.
#[inline]
fn is_eligible(
    inst: &SearchInstance,
    excluded_mask: FanExclusionSet,
    max_allowed_score: u8,
    used_sig_keys: &[SigKey; 32],
    used_count: usize,
) -> bool {
    if inst.score > max_allowed_score {
        return false;
    }
    // Check exclusion: is this fan's bit set in the excluded mask?
    if (excluded_mask.0 >> (inst.fan_type as u16)) & 1 == 1 {
        return false;
    }
    // Check sig_key uniqueness (non-repeat principle)
    for &sk in used_sig_keys[..used_count].iter() {
        if sk == inst.sig_key {
            return false;
        }
    }
    true
}

// ── Search ──

/// Find the maximum-score subset of the given fan candidates respecting
/// the MCR non-repeat principle and exclusion rules.
///
/// Uses iterative DFS with explicit stack, ordering candidates by
/// descending score.  The `max_allowed_score` monotonicity constraint
/// (scores must be non-increasing) eliminates redundant permutations.
pub fn solve_max_score(candidates: Vec<FanCandidate>) -> FanSolveResult {
    if candidates.is_empty() {
        return FanSolveResult::default();
    }

    let space = SearchSpace::build(candidates);
    let instances = &space.instances;
    let order = &space.order;

    // Mutable search state
    let mut path: Vec<usize> = Vec::with_capacity(8);
    let mut used_sig_keys: [SigKey; 32] = [0; 32];
    let mut used_count: usize = 0;

    let mut excluded_mask = FanExclusionSet::default();
    let mut max_allowed_score = instances[order[0]].score;
    let mut total_score: u16 = 0;

    let mut best_score: u16 = 0;
    let mut best_path: Vec<usize> = Vec::new();

    let mut stack: Vec<Frame> = Vec::new();
    let mut pos: usize = 0;

    loop {
        // Try to pick the next eligible instance from `pos` onwards.
        let mut picked = false;
        for idx in pos..order.len() {
            let inst_id = order[idx];
            let inst = &instances[inst_id];

            if !is_eligible(
                inst,
                excluded_mask,
                max_allowed_score,
                &used_sig_keys,
                used_count,
            ) {
                continue;
            }

            // Save frame for backtracking: "try next candidate at this depth"
            stack.push(Frame {
                resume_pos: idx + 1,
                max_allowed_score,
                excluded_mask,
                total_score,
                path_len: path.len(),
            });

            // Commit this choice
            path.push(inst_id);
            used_sig_keys[used_count] = inst.sig_key;
            used_count += 1;

            excluded_mask =
                FanExclusionSet(excluded_mask.0 | inst.excludes_mask.0);
            max_allowed_score = inst.score; // non-increasing
            total_score += inst.score as u16;

            pos = idx + 1;
            picked = true;
            break;
        }

        if !picked {
            // Leaf: reached the end of the candidate list at this depth.
            if total_score > best_score {
                best_score = total_score;
                best_path = path.clone();
            }

            // Backtrack to previous decision point.
            if let Some(frame) = stack.pop() {
                pos = frame.resume_pos;
                max_allowed_score = frame.max_allowed_score;
                excluded_mask = frame.excluded_mask;
                total_score = frame.total_score;

                // Unwind path and sig_keys back to the saved frame's depth.
                while path.len() > frame.path_len {
                    path.pop();
                    used_count -= 1;
                }
            } else {
                break; // stack empty: search complete
            }
        }
    }

    // Convert best_path to the output type.
    let fans: Vec<FanInstance> = best_path
        .into_iter()
        .map(|id| {
            let inst = &instances[id];
            FanInstance {
                fan_type: inst.fan_type,
                used_set_mask: inst.used_set_mask,
                uses_pair: inst.uses_pair,
                score: inst.score,
            }
        })
        .collect();

    FanSolveResult {
        total_score: best_score,
        fans,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::fan::fan_types::FanType;

    fn candidate(
        fan_type: FanType,
        used_set_mask: u64,
        uses_pair: bool,
    ) -> FanCandidate {
        let excludes_mask = FanExclusionSet::for_fan(fan_type);
        FanCandidate {
            fan_type,
            used_set_mask,
            uses_pair,
            score: fan_type.points(),
            excludes_mask,
        }
    }

    #[test]
    fn test_solve_max_score_empty() {
        let result = solve_max_score(vec![]);
        assert_eq!(result.total_score, 0);
        assert!(result.fans.is_empty());
    }

    #[test]
    fn test_solve_max_score_single() {
        // Big Four Winds (88) alone should score 88
        let candidates = vec![candidate(FanType::BigFourWinds, 0b1111, false)];
        let result = solve_max_score(candidates);
        assert_eq!(result.total_score, 88);
        assert_eq!(result.fans.len(), 1);
    }

    #[test]
    fn test_solve_max_score_compatible() {
        // Big Four Winds (88) + All Terminals and Honors (32) + Half Flush (6)
        // These are all compatible (no exclusions between them, different sig keys)
        let candidates = vec![
            candidate(FanType::BigFourWinds, 0b0011, false),
            candidate(FanType::AllTerminalsAndHonors, 0b1111, true),
            candidate(FanType::HalfFlush, 0b1111, true),
        ];
        let result = solve_max_score(candidates);
        assert_eq!(result.total_score, 88 + 32 + 6);
        assert_eq!(result.fans.len(), 3);
    }

    #[test]
    fn test_solve_max_score_excludes() {
        // Big Four Winds (88) excludes Little Four Winds (64) and All Pungs (6)
        // Little Four Winds excludes Big Four Winds
        // So picking Big Four Winds means we cannot also pick Little Four Winds.
        // But we CAN pick Little Four Winds alone (64).
        let candidates = vec![
            candidate(FanType::BigFourWinds, 0b1111, false),
            candidate(FanType::LittleFourWinds, 0b0111, true),
        ];
        let result = solve_max_score(candidates);
        // big-four 88 beats little-four 64, so result should be 88
        assert_eq!(result.total_score, 88);
        assert_eq!(result.fans.len(), 1);
        assert_eq!(result.fans[0].fan_type, FanType::BigFourWinds);
    }

    #[test]
    fn test_solve_max_score_excludes_reverse() {
        // AllHonors (64) excludes AllTerminalsAndHonors (32)
        // AllTerminalsAndHonors excludes AllHonors
        // Best score: 64 (AllHonors) is better than 32 (AllTerminalsAndHonors)
        let candidates = vec![
            candidate(FanType::AllTerminalsAndHonors, 0b1111, true),
            candidate(FanType::AllHonors, 0b1111, true),
        ];
        let result = solve_max_score(candidates);
        assert_eq!(result.total_score, 64);
        assert_eq!(result.fans.len(), 1);
        assert_eq!(result.fans[0].fan_type, FanType::AllHonors);
    }

    #[test]
    fn test_solve_max_score_non_repeat() {
        // Two identical candidates should not both be selectable (same sig_key)
        let candidates = vec![
            candidate(FanType::AllPungs, 0b1111, true),
            candidate(FanType::AllPungs, 0b1111, true),
        ];
        let result = solve_max_score(candidates);
        assert_eq!(result.total_score, 6);
        assert_eq!(result.fans.len(), 1);
    }

    #[test]
    fn test_solve_max_score_complex() {
        // Big Four Winds (88, uses sets 0-3) + All Terminals (32, uses all sets + pair)
        // + Half Flush (6, uses all sets + pair) + All Pungs (6, uses all sets + pair)
        // Big Four Winds excludes All Pungs. So best is 88 + 32 + 6 = 126.
        let candidates = vec![
            candidate(FanType::BigFourWinds, 0b1111, false),
            candidate(FanType::AllTerminalsAndHonors, 0b1111, true),
            candidate(FanType::HalfFlush, 0b1111, true),
            candidate(FanType::AllPungs, 0b1111, true),
        ];
        let result = solve_max_score(candidates);
        assert_eq!(result.total_score, 88 + 32 + 6);
        assert_eq!(result.fans.len(), 3);
    }
}
