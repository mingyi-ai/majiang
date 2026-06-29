// ═══════════════════════════════════════════════════════════════
// Shared test helpers for MCR fan rule tests
//
// Every test case is sourced from the official Mahjong Competition
// Rules rulebook (World Mahjong Organization, 2006).  Each example
// lists the hand arrangement and which fans it inevitably combines
// with or excludes.
// ═══════════════════════════════════════════════════════════════

#![cfg(test)]

use crate::solver::{
    rules::FanType, DynamicFanContext, FanResult, StaticFanContext, WaitType,
    WinMethod, solve_fan,
};
use crate::structs::{Hand, Meld, Tile, Wind};

/// Build a hand where ALL tiles are in the concealed hand.
/// The melds argument describes the *expected decomposition* — the
/// tiles are inserted into `hand.concealed` and the engine finds them.
pub(crate) fn concealed_hand(melds: Vec<Meld>, pair_tile: Tile) -> Hand {
    let mut hand = Hand::default();
    for m in melds {
        for t in m.tiles() {
            hand.concealed.insert(t);
        }
    }
    hand.concealed.insert(pair_tile);
    hand.concealed.insert(pair_tile);
    hand
}

/// Build a hand with some melds declared (exposed) and the rest concealed.
pub(crate) fn declared_hand(
    declared: Vec<Meld>,
    concealed: Vec<Meld>,
    pair_tile: Tile,
) -> Hand {
    let mut hand = Hand::default();
    for m in declared {
        hand.melds.push(m);
    }
    for m in concealed {
        for t in m.tiles() {
            hand.concealed.insert(t);
        }
    }
    hand.concealed.insert(pair_tile);
    hand.concealed.insert(pair_tile);
    hand
}

/// Build a hand where all melds are declared (exposed) and only the
/// pair is concealed.
pub(crate) fn all_declared_hand(melds: Vec<Meld>, pair_tile: Tile) -> Hand {
    let mut hand = Hand::default();
    for m in melds {
        hand.melds.push(m);
    }
    hand.concealed.insert(pair_tile);
    hand.concealed.insert(pair_tile);
    hand
}

/// A generic default static context — East seat, East round, discard win,
/// multiple wait, no flowers, not the last tile, not concealed.
pub(crate) fn default_static_ctx() -> StaticFanContext {
    StaticFanContext {
        seat_wind: Wind::East,
        prevalent_wind: Wind::East,
        win_method: WinMethod::Discard,
        winning_tile: Tile::Character1,
        flower_count: 0,
        is_concealed: false,
        is_fully_concealed: false,
        is_last_tile_draw: false,
        is_last_tile_claim: false,
        is_last_tile_of_kind: false,
        wall_remaining: 20,
    }
}

/// A default dynamic context — no kong replacement, no rob kong.
pub(crate) fn default_dynamic_ctx() -> DynamicFanContext {
    DynamicFanContext {
        is_kong_replacement: false,
        is_rob_kong: false,
    }
}

/// Solve a hand with default contexts.
pub(crate) fn solve_default(hand: &Hand) -> Vec<FanResult> {
    let static_ctx = default_static_ctx();
    let dynamic_ctx = default_dynamic_ctx();
    solve_fan(hand, &static_ctx, &dynamic_ctx).unwrap()
}

/// Helper: check that a result contains the given fan type.
pub(crate) fn assert_contains_fan(result: &FanResult, fan: FanType) {
    assert!(
        result.fans.iter().any(|f| f.fan_type == fan),
        "expected fan {:?} ({}) in result, got: {:?}",
        fan,
        fan.name(),
        result
            .fans
            .iter()
            .map(|f| f.fan_type.name())
            .collect::<Vec<_>>()
    );
}

/// Helper: check that a result does NOT contain the given fan type.
pub(crate) fn assert_not_contains_fan(result: &FanResult, fan: FanType) {
    assert!(
        !result.fans.iter().any(|f| f.fan_type == fan),
        "fan {:?} ({}) should NOT be in result",
        fan,
        fan.name()
    );
}


/// Build a hand from an array of 7 pair tiles (each inserted twice).
pub(crate) fn seven_pairs_hand(pair_tiles: &[Tile]) -> Hand {
    let mut hand = Hand::default();
    for &t in pair_tiles {
        hand.concealed.insert(t);
        hand.concealed.insert(t);
    }
    hand
}

/// Helper: check total score of a result.
pub(crate) fn assert_total_score(result: &FanResult, expected: u8) {
    let actual = result.total_score();
    assert_eq!(
        actual, expected,
        "expected total score {}, got {}; fans: {:?}",
        expected, actual,
        result
            .fans
            .iter()
            .map(|f| f.fan_type.name())
            .collect::<Vec<_>>()
    );
}
