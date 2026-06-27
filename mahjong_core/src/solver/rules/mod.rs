// ═══════════════════════════════════════════════════════════════
// MCR Fan Rule Registry
//
// A flat array of (check_fn, excludes) pairs replaces the old
// `define_rules!` macro.  Check functions now take a `&HandProfile`
// instead of `&Decomposition`, eliminating per-rule re-parsing.
// ═══════════════════════════════════════════════════════════════

mod helpers;
mod high;
mod low;
mod mid_high;
mod mid_low;

use super::fan_context::FanContext;
use super::types::{FanCandidate, FanType};
use super::view::HandProfile;

/// A registered rule: a check function and its exclusion list.
struct RuleEntry {
    check: fn(&HandProfile, &FanContext) -> Vec<FanCandidate>,
    excludes: &'static [FanType],
}

/// All 81 MCR rules in the standard order (descending by point value).
const ALL_RULES: &[RuleEntry] = &[
    // ── 88 points ──
    RuleEntry {
        check: high::big_four_winds,
        excludes: high::BIG_FOUR_WINDS_EXCLUDES,
    },
    RuleEntry {
        check: high::big_three_dragons,
        excludes: high::BIG_THREE_DRAGONS_EXCLUDES,
    },
    RuleEntry {
        check: high::all_green,
        excludes: high::ALL_GREEN_EXCLUDES,
    },
    RuleEntry {
        check: high::nine_gates,
        excludes: high::NINE_GATES_EXCLUDES,
    },
    RuleEntry {
        check: high::four_kongs,
        excludes: high::FOUR_KONGS_EXCLUDES,
    },
    RuleEntry {
        check: high::seven_shifted_pairs,
        excludes: high::SEVEN_SHIFTED_PAIRS_EXCLUDES,
    },
    RuleEntry {
        check: high::thirteen_orphans,
        excludes: high::THIRTEEN_ORPHANS_EXCLUDES,
    },
    // ── 64 points ──
    RuleEntry {
        check: high::all_terminals,
        excludes: high::ALL_TERMINALS_EXCLUDES,
    },
    RuleEntry {
        check: high::little_four_winds,
        excludes: high::LITTLE_FOUR_WINDS_EXCLUDES,
    },
    RuleEntry {
        check: high::little_three_dragons,
        excludes: high::LITTLE_THREE_DRAGONS_EXCLUDES,
    },
    RuleEntry {
        check: high::all_honors,
        excludes: high::ALL_HONORS_EXCLUDES,
    },
    RuleEntry {
        check: high::four_concealed_pungs,
        excludes: high::FOUR_CONCEALED_PUNGS_EXCLUDES,
    },
    RuleEntry {
        check: high::pure_terminal_chows,
        excludes: high::PURE_TERMINAL_CHOWS_EXCLUDES,
    },
    // ── 48 points ──
    RuleEntry {
        check: high::quadruple_chow,
        excludes: high::QUADRUPLE_CHOW_EXCLUDES,
    },
    RuleEntry {
        check: high::four_pure_shifted_pungs,
        excludes: high::FOUR_PURE_SHIFTED_PUNGS_EXCLUDES,
    },
    // ── 32 points ──
    RuleEntry {
        check: mid_high::four_shifted_chows,
        excludes: mid_high::FOUR_SHIFTED_CHOWS_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::three_kongs,
        excludes: mid_high::THREE_KONGS_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::all_terminals_and_honors,
        excludes: mid_high::ALL_TERMINALS_AND_HONORS_EXCLUDES,
    },
    // ── 24 points ──
    RuleEntry {
        check: mid_high::seven_pairs,
        excludes: mid_high::SEVEN_PAIRS_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::all_even_pungs,
        excludes: mid_high::ALL_EVEN_PUNGS_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::full_flush,
        excludes: mid_high::FULL_FLUSH_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::pure_triple_chow,
        excludes: mid_high::PURE_TRIPLE_CHOW_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::pure_shifted_pungs,
        excludes: mid_high::PURE_SHIFTED_PUNGS_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::upper_tiles,
        excludes: mid_high::UPPER_TILES_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::middle_tiles,
        excludes: mid_high::MIDDLE_TILES_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::lower_tiles,
        excludes: mid_high::LOWER_TILES_EXCLUDES,
    },
    // ── 16 points ──
    RuleEntry {
        check: mid_high::pure_straight,
        excludes: mid_high::PURE_STRAIGHT_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::three_suited_terminal_chows,
        excludes: mid_high::THREE_SUITED_TERMINAL_CHOWS_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::pure_shifted_chows,
        excludes: mid_high::PURE_SHIFTED_CHOWS_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::all_fives,
        excludes: mid_high::ALL_FIVES_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::triple_pung,
        excludes: mid_high::TRIPLE_PUNG_EXCLUDES,
    },
    RuleEntry {
        check: mid_high::three_concealed_pungs,
        excludes: mid_high::THREE_CONCEALED_PUNGS_EXCLUDES,
    },
    // ── 12 points ──
    RuleEntry {
        check: mid_low::upper_four,
        excludes: mid_low::UPPER_FOUR_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::lower_four,
        excludes: mid_low::LOWER_FOUR_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::big_three_winds,
        excludes: mid_low::BIG_THREE_WINDS_EXCLUDES,
    },
    // ── 8 points ──
    RuleEntry {
        check: mid_low::mixed_straight,
        excludes: mid_low::MIXED_STRAIGHT_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::reversible_tiles,
        excludes: mid_low::REVERSIBLE_TILES_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::mixed_triple_chow,
        excludes: mid_low::MIXED_TRIPLE_CHOW_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::mixed_shifted_pungs,
        excludes: mid_low::MIXED_SHIFTED_PUNGS_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::chicken_hand,
        excludes: mid_low::CHICKEN_HAND_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::last_tile_draw,
        excludes: mid_low::LAST_TILE_DRAW_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::last_tile_claim,
        excludes: mid_low::LAST_TILE_CLAIM_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::out_with_replacement_tile,
        excludes: mid_low::OUT_WITH_REPLACEMENT_TILE_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::robbing_the_kong,
        excludes: mid_low::ROBBING_THE_KONG_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::two_concealed_kongs,
        excludes: mid_low::TWO_CONCEALED_KONGS_EXCLUDES,
    },
    // ── 6 points ──
    RuleEntry {
        check: mid_low::all_pungs,
        excludes: mid_low::ALL_PUNGS_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::half_flush,
        excludes: mid_low::HALF_FLUSH_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::mixed_shifted_chows,
        excludes: mid_low::MIXED_SHIFTED_CHOWS_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::all_types,
        excludes: mid_low::ALL_TYPES_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::melded_hand,
        excludes: mid_low::MELDED_HAND_EXCLUDES,
    },
    RuleEntry {
        check: mid_low::two_dragon_pungs,
        excludes: mid_low::TWO_DRAGON_PUNGS_EXCLUDES,
    },
    // ── 4 points ──
    RuleEntry {
        check: low::outside_hand,
        excludes: low::OUTSIDE_HAND_EXCLUDES,
    },
    RuleEntry {
        check: low::fully_concealed,
        excludes: low::FULLY_CONCEALED_EXCLUDES,
    },
    RuleEntry {
        check: low::two_melded_kongs,
        excludes: low::TWO_MELDED_KONGS_EXCLUDES,
    },
    RuleEntry {
        check: low::last_tile,
        excludes: low::LAST_TILE_EXCLUDES,
    },
    // ── 2 points ──
    RuleEntry {
        check: low::dragon_pung,
        excludes: low::DRAGON_PUNG_EXCLUDES,
    },
    RuleEntry {
        check: low::prevalent_wind,
        excludes: low::PREVALENT_WIND_EXCLUDES,
    },
    RuleEntry {
        check: low::seat_wind,
        excludes: low::SEAT_WIND_EXCLUDES,
    },
    RuleEntry {
        check: low::concealed_hand,
        excludes: low::CONCEALED_HAND_EXCLUDES,
    },
    RuleEntry {
        check: low::all_chows,
        excludes: low::ALL_CHOWS_EXCLUDES,
    },
    RuleEntry {
        check: low::tile_hog,
        excludes: low::TILE_HOG_EXCLUDES,
    },
    RuleEntry {
        check: low::double_pung,
        excludes: low::DOUBLE_PUNG_EXCLUDES,
    },
    RuleEntry {
        check: low::two_concealed_pungs,
        excludes: low::TWO_CONCEALED_PUNGS_EXCLUDES,
    },
    RuleEntry {
        check: low::concealed_kong,
        excludes: low::CONCEALED_KONG_EXCLUDES,
    },
    RuleEntry {
        check: low::all_simples,
        excludes: low::ALL_SIMPLES_EXCLUDES,
    },
    // ── 1 point ──
    RuleEntry {
        check: low::pure_double_chow,
        excludes: low::PURE_DOUBLE_CHOW_EXCLUDES,
    },
    RuleEntry {
        check: low::mixed_double_chow,
        excludes: low::MIXED_DOUBLE_CHOW_EXCLUDES,
    },
    RuleEntry {
        check: low::short_straight,
        excludes: low::SHORT_STRAIGHT_EXCLUDES,
    },
    RuleEntry {
        check: low::two_terminal_chows,
        excludes: low::TWO_TERMINAL_CHOWS_EXCLUDES,
    },
    RuleEntry {
        check: low::pung_of_terminals_or_honors,
        excludes: low::PUNG_OF_TERMINALS_OR_HONORS_EXCLUDES,
    },
    RuleEntry {
        check: low::melded_kong,
        excludes: low::MELDED_KONG_EXCLUDES,
    },
    RuleEntry {
        check: low::one_voided_suit,
        excludes: low::ONE_VOIDED_SUIT_EXCLUDES,
    },
    RuleEntry {
        check: low::no_honors,
        excludes: low::NO_HONORS_EXCLUDES,
    },
    RuleEntry {
        check: low::edge_wait,
        excludes: low::EDGE_WAIT_EXCLUDES,
    },
    RuleEntry {
        check: low::closed_wait,
        excludes: low::CLOSED_WAIT_EXCLUDES,
    },
    RuleEntry {
        check: low::single_wait,
        excludes: low::SINGLE_WAIT_EXCLUDES,
    },
    RuleEntry {
        check: low::self_drawn,
        excludes: low::SELF_DRAWN_EXCLUDES,
    },
    RuleEntry {
        check: low::flower_tiles,
        excludes: low::FLOWER_TILES_EXCLUDES,
    },
];

/// Check all rules against a decomposition profile + context.
///
/// Returns candidates paired with their exclusion lists.
pub(crate) fn check_all(
    profile: &HandProfile,
    ctx: &FanContext,
) -> Vec<(FanCandidate, &'static [FanType])> {
    let mut result = Vec::new();
    for entry in ALL_RULES {
        let candidates = (entry.check)(profile, ctx);
        if !candidates.is_empty() {
            result.extend(candidates.into_iter().map(|c| (c, entry.excludes)));
        }
    }
    result
}
