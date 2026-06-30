use crate::solver::{
    FanInstance, FanResult,
    rules::{FanExclusionSet, FanType},
};

/// Dedup key: (fan_type, uses_pair, used_set_mask).
/// Two candidates with the same triple are the same fan instance
/// under MCR's non-repeat principle and cannot both be selected.
type DedupKey = (FanType, bool, u64);

/// A frame on the explicit DFS stack.
struct Frame {
    resume_pos: usize,
    max_allowed_score: u8,
    excluded_mask: FanExclusionSet,
    total_score: u16,
    path_len: usize,
    used_sets: u64,
    bridge_count: [u8; 4],
}

// ── Search space construction ──

struct SearchSpace {
    instances: Vec<FanInstance>,
    order: Vec<usize>, // sorted by (-points, index) for deterministic pruning
}

impl SearchSpace {
    fn build(candidates: Vec<FanInstance>) -> Self {
        let instances = candidates;
        let mut order: Vec<usize> = (0..instances.len()).collect();
        order.sort_by_key(|&i| {
            let inst = &instances[i];
            ((inst.fan_type.points() as i16).wrapping_neg(), i)
        });

        Self { instances, order }
    }
}

// ── Eligibility check ──

/// Check if a candidate can be added to the current selection.
/// Enforces:
///   1. Score monotonicity (non-increasing scores)
///   2. MCR mutual exclusion (excludes_mask)
///   3. Non-repeat (dedup key)
///   4. Account-Once Principle: a set can bridge to remaining sets at most once.
///      Only applies to structural fans (≥2 sets). Hand properties (mask = 0) and
///      single-set fans (mask has 1 bit) are exempt — they don't combine sets.
#[inline]
fn is_eligible(
    inst: &FanInstance,
    excluded_mask: FanExclusionSet,
    max_allowed_score: u8,
    used_keys: &[DedupKey],
    used_sets: u64,
    bridge_count: &[u8; 4],
) -> bool {
    if inst.fan_type.points() > max_allowed_score {
        return false;
    }
    if (excluded_mask.0 >> (inst.fan_type as u16)) & 1 == 1 {
        return false;
    }
    let key = (inst.fan_type, inst.uses_pair, inst.used_set_mask);
    if used_keys.contains(&key) {
        return false;
    }
    // Account-Once: only structural fans (≥2 sets, non-zero)
    let m = inst.used_set_mask;
    let n_sets = m.count_ones();
    if n_sets >= 2 && used_sets != 0 {
        let intersection = m & used_sets;
        let remaining = m & !used_sets;
        if intersection != 0 && remaining != 0 {
            // This fan bridges from already-used sets to new sets.
            // Each set can bridge at most once.
            for set in 0..4 {
                if (intersection >> set) & 1 == 1 && bridge_count[set] >= 1 {
                    return false;
                }
            }
        }
    }
    true
}

/// Build a dedup key from a FanInstance.
fn dedup_key_of(inst: &FanInstance) -> DedupKey {
    (inst.fan_type, inst.uses_pair, inst.used_set_mask)
}

// ── Search ──

/// Find the maximum-score subset of the given fan candidates.
///
/// Uses iterative DFS with explicit stack. Candidates are ordered by
/// descending score; the `max_allowed_score` monotonicity constraint
/// (scores must be non-increasing) eliminates redundant permutations.
///
/// Returns all solutions achieving the maximum score (ties).
pub fn solve_max_score(candidates: Vec<FanInstance>) -> Vec<FanResult> {
    if candidates.is_empty() {
        return vec![FanResult::default()];
    }

    let space = SearchSpace::build(candidates);
    let instances = &space.instances;
    let order = &space.order;

    let mut path: Vec<usize> = Vec::with_capacity(8);
    let mut used_keys: Vec<DedupKey> = Vec::with_capacity(32);

    let mut excluded_mask = FanExclusionSet::default();
    let mut max_allowed_score = instances[order[0]].fan_type.points();
    let mut total_score: u16 = 0;
    let mut used_sets: u64 = 0;
    let mut bridge_count: [u8; 4] = [0; 4];

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
                &used_keys,
                used_sets,
                &bridge_count,
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
                used_sets,
                bridge_count,
            });

            path.push(inst_id);
            used_keys.push((
                inst.fan_type,
                inst.uses_pair,
                inst.used_set_mask,
            ));

            excluded_mask = FanExclusionSet(
                excluded_mask.0 | inst.fan_type.excludes_mask().0,
            );
            max_allowed_score = inst.fan_type.points();
            total_score += inst.fan_type.points() as u16;

            // Update Account-Once state for structural fans
            let m = inst.used_set_mask;
            if m.count_ones() >= 2 {
                let intersection = m & used_sets;
                let remaining = m & !used_sets;
                if intersection != 0 && remaining != 0 {
                    for set in 0..4 {
                        if (intersection >> set) & 1 == 1 {
                            bridge_count[set] += 1;
                        }
                    }
                }
                used_sets |= m;
            }

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
                used_sets = frame.used_sets;
                bridge_count = frame.bridge_count;

                while path.len() > frame.path_len {
                    path.pop();
                    used_keys.pop();
                }
            } else {
                break;
            }
        }
    }

    let results: Vec<FanResult> = best_paths
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
                    }
                })
                .collect();
            // Deduplicate — the search kernel prevents identical dedup keys
            // from being selected in one path, but different paths
            // may produce the same set of instances with different ordering.
            let mut seen = std::collections::HashSet::new();
            let mut unique = Vec::new();
            for f in fans {
                let key = dedup_key_of(&f);
                if seen.insert(key) {
                    unique.push(f);
                }
            }
            FanResult { fans: unique }
        })
        .collect();

    if results.is_empty() {
        vec![FanResult::default()]
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
    ) -> FanInstance {
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
        FanInstance {
            fan_type,
            used_set_mask,
            uses_pair,
        }
    }

    #[test]
    fn test_solve_max_score_empty() {
        let results = solve_max_score(vec![]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].total_score(), 0);
        assert!(results[0].fans.is_empty());
    }

    #[test]
    fn test_solve_max_score_single() {
        let results = solve_max_score(vec![candidate(
            FanType::BigFourWinds,
            0b1111,
            false,
        )]);
        assert_eq!(results[0].total_score(), 88);
        assert_eq!(results[0].fans.len(), 1);
    }

    #[test]
    fn test_solve_max_score_compatible() {
        let results = solve_max_score(vec![
            candidate(FanType::BigFourWinds, 0b0011, false),
            candidate(FanType::AllTerminalsAndHonors, 0b1111, true),
            candidate(FanType::HalfFlush, 0b1111, true),
        ]);
        assert_eq!(results[0].total_score(), 88 + 32 + 6);
        assert_eq!(results[0].fans.len(), 3);
    }

    #[test]
    fn test_solve_max_score_excludes() {
        let results = solve_max_score(vec![
            candidate(FanType::BigFourWinds, 0b1111, false),
            candidate(FanType::LittleFourWinds, 0b0111, true),
        ]);
        assert_eq!(results[0].total_score(), 88);
        assert_eq!(results[0].fans.len(), 1);
        assert_eq!(results[0].fans[0].fan_type, FanType::BigFourWinds);
    }

    #[test]
    fn test_solve_max_score_excludes_reverse() {
        let results = solve_max_score(vec![
            candidate(FanType::AllTerminalsAndHonors, 0b1111, true),
            candidate(FanType::AllHonors, 0b1111, true),
        ]);
        assert_eq!(results[0].total_score(), 64);
        assert_eq!(results[0].fans.len(), 1);
        assert_eq!(results[0].fans[0].fan_type, FanType::AllHonors);
    }

    #[test]
    fn test_solve_max_score_non_repeat() {
        let results = solve_max_score(vec![
            candidate(FanType::AllPungs, 0b1111, true),
            candidate(FanType::AllPungs, 0b1111, true),
        ]);
        assert_eq!(results[0].total_score(), 6);
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
        assert_eq!(results[0].total_score(), 88 + 32 + 6);
        assert_eq!(results[0].fans.len(), 3);
    }
}
