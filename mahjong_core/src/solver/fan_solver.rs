// ═══════════════════════════════════════════════════════════════
// Search kernel — maximum-weight compatible fan subset
//
// Extracted from fan.rs.  Given a set of FanCandidates (each with
// an exclusion mask, used-set mask, and score), finds the
// maximum-total-score subset respecting:
//   - mutual exclusion (FanExclusionSet)
//   - non-repeat (same sig_key cannot appear twice)
//   - score-monotonic ordering (non-increasing scores)
//
// Returns all solutions achieving the maximum score (ties).
// ═══════════════════════════════════════════════════════════════

use crate::solver::{
    FanInstance,
    rules::{FanCandidate, FanExclusionSet, FanType},
};

/// Result of a max-score search: total score + collected fan instances.
#[derive(Debug, Clone, Default)]
pub(crate) struct FanSolveResult {
    pub(crate) total_score: u16,
    pub(crate) fans: Vec<FanInstance>,
}

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

        let mut order: Vec<usize> = (0..instances.len()).collect();
        order.sort_by_key(|&i| {
            let inst = &instances[i];
            ((inst.score as i16).wrapping_neg(), inst.id)
        });

        Self { instances, order }
    }
}

// ── Eligibility check ──

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
    if (excluded_mask.0 >> (inst.fan_type as u16)) & 1 == 1 {
        return false;
    }
    for &sk in used_sig_keys[..used_count].iter() {
        if sk == inst.sig_key {
            return false;
        }
    }
    true
}

/// Get sig_key from a FanInstance (for deduplication).
fn sig_key_of(inst: &FanInstance) -> SigKey {
    ((inst.fan_type as u16 as u64) << 17)
        | ((inst.uses_pair as u64) << 16)
        | (inst.used_set_mask & 0xFFFF)
}

// ── Search ──

/// Find the maximum-score subset of the given fan candidates.
///
/// Uses iterative DFS with explicit stack. Candidates are ordered by
/// descending score; the `max_allowed_score` monotonicity constraint
/// (scores must be non-increasing) eliminates redundant permutations.
///
/// Returns all solutions achieving the maximum score (ties).
pub fn solve_max_score(candidates: Vec<FanCandidate>) -> Vec<FanSolveResult> {
    if candidates.is_empty() {
        return vec![FanSolveResult::default()];
    }

    let space = SearchSpace::build(candidates);
    let instances = &space.instances;
    let order = &space.order;

    let mut path: Vec<usize> = Vec::with_capacity(8);
    let mut used_sig_keys: [SigKey; 32] = [0; 32];
    let mut used_count: usize = 0;

    let mut excluded_mask = FanExclusionSet::default();
    let mut max_allowed_score = instances[order[0]].score;
    let mut total_score: u16 = 0;

    let mut best_score: u16 = 0;
    let mut best_paths: Vec<Vec<usize>> = Vec::new();

    let mut stack: Vec<Frame> = Vec::new();
    let mut pos: usize = 0;

    loop {
        let mut picked = false;
        let mut scan = pos;
        while scan < order.len() {
            let inst_id = order[scan];
            let inst = &instances[inst_id];

            if !is_eligible(
                inst,
                excluded_mask,
                max_allowed_score,
                &used_sig_keys,
                used_count,
            ) {
                scan += 1;
                continue;
            }

            stack.push(Frame {
                resume_pos: scan + 1,
                max_allowed_score,
                excluded_mask,
                total_score,
                path_len: path.len(),
            });

            path.push(inst_id);
            used_sig_keys[used_count] = inst.sig_key;
            used_count += 1;

            excluded_mask =
                FanExclusionSet(excluded_mask.0 | inst.excludes_mask.0);
            max_allowed_score = inst.score;
            total_score += inst.score as u16;

            pos = scan + 1;
            picked = true;
            break;
        }

        if !picked {
            if total_score > best_score {
                best_score = total_score;
                best_paths.clear();
                best_paths.push(path.clone());
            } else if total_score == best_score && total_score > 0 {
                best_paths.push(path.clone());
            }

            if let Some(frame) = stack.pop() {
                pos = frame.resume_pos;
                max_allowed_score = frame.max_allowed_score;
                excluded_mask = frame.excluded_mask;
                total_score = frame.total_score;

                while path.len() > frame.path_len {
                    path.pop();
                    used_count -= 1;
                }
            } else {
                break;
            }
        }
    }

    let results: Vec<FanSolveResult> = best_paths
        .into_iter()
        .map(|bp| {
            let fans: Vec<FanInstance> = bp
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
            // Deduplicate by sig_key — the search kernel prevents identical
            // sig_keys from being selected in one path, but different paths
            // may produce the same set of instances with different ordering.
            let mut seen = std::collections::HashSet::new();
            let mut unique = Vec::new();
            for f in fans {
                let key = sig_key_of(&f);
                if seen.insert(key) {
                    unique.push(f);
                }
            }
            FanSolveResult {
                total_score: best_score,
                fans: unique,
            }
        })
        .collect();

    if results.is_empty() {
        vec![FanSolveResult::default()]
    } else {
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        fan_type: FanType,
        used_set_mask: u64,
        uses_pair: bool,
    ) -> FanCandidate {
        let mut excludes_mask = FanExclusionSet::default();
        match fan_type {
            FanType::BigFourWinds => {
                excludes_mask.set_bit(FanType::AllPungs.bit_index());
                excludes_mask.set_bit(FanType::LittleFourWinds.bit_index());
            }
            FanType::LittleFourWinds => {
                excludes_mask.set_bit(FanType::BigFourWinds.bit_index());
            }
            FanType::AllHonors => {
                excludes_mask
                    .set_bit(FanType::AllTerminalsAndHonors.bit_index());
            }
            FanType::AllTerminalsAndHonors => {
                excludes_mask.set_bit(FanType::AllHonors.bit_index());
            }
            _ => {}
        }
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
        let results = solve_max_score(vec![]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].total_score, 0);
        assert!(results[0].fans.is_empty());
    }

    #[test]
    fn test_solve_max_score_single() {
        let results = solve_max_score(vec![candidate(
            FanType::BigFourWinds,
            0b1111,
            false,
        )]);
        assert_eq!(results[0].total_score, 88);
        assert_eq!(results[0].fans.len(), 1);
    }

    #[test]
    fn test_solve_max_score_compatible() {
        let results = solve_max_score(vec![
            candidate(FanType::BigFourWinds, 0b0011, false),
            candidate(FanType::AllTerminalsAndHonors, 0b1111, true),
            candidate(FanType::HalfFlush, 0b1111, true),
        ]);
        assert_eq!(results[0].total_score, 88 + 32 + 6);
        assert_eq!(results[0].fans.len(), 3);
    }

    #[test]
    fn test_solve_max_score_excludes() {
        let results = solve_max_score(vec![
            candidate(FanType::BigFourWinds, 0b1111, false),
            candidate(FanType::LittleFourWinds, 0b0111, true),
        ]);
        assert_eq!(results[0].total_score, 88);
        assert_eq!(results[0].fans.len(), 1);
        assert_eq!(results[0].fans[0].fan_type, FanType::BigFourWinds);
    }

    #[test]
    fn test_solve_max_score_excludes_reverse() {
        let results = solve_max_score(vec![
            candidate(FanType::AllTerminalsAndHonors, 0b1111, true),
            candidate(FanType::AllHonors, 0b1111, true),
        ]);
        assert_eq!(results[0].total_score, 64);
        assert_eq!(results[0].fans.len(), 1);
        assert_eq!(results[0].fans[0].fan_type, FanType::AllHonors);
    }

    #[test]
    fn test_solve_max_score_non_repeat() {
        let results = solve_max_score(vec![
            candidate(FanType::AllPungs, 0b1111, true),
            candidate(FanType::AllPungs, 0b1111, true),
        ]);
        assert_eq!(results[0].total_score, 6);
        assert_eq!(results[0].fans.len(), 1);
    }

    #[test]
    fn test_solve_max_score_complex() {
        let results = solve_max_score(vec![
            candidate(FanType::BigFourWinds, 0b1111, false),
            candidate(FanType::AllTerminalsAndHonors, 0b1111, true),
            candidate(FanType::HalfFlush, 0b1111, true),
            candidate(FanType::AllPungs, 0b1111, true),
        ]);
        assert_eq!(results[0].total_score, 88 + 32 + 6);
        assert_eq!(results[0].fans.len(), 3);
    }
}
