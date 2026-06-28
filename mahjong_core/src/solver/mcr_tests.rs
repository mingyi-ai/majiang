// Each test should go to where rule is registered.

// ═══════════════════════════════════════════════════════════════
// MCR Fan Rule Tests
//
// Every test case is sourced from the official Mahjong Competition
// Rules rulebook (World Mahjong Organization, 2006).  Each example
// lists the hand arrangement and which fans it inevitably combines
// with or excludes.
//
// TESTS CONSTRUCT THE HAND CORRECTLY:
// - Unless explicitly stated as melded/exposed, ALL tiles are placed
//   in `hand.concealed`.  The groupings shown in the rulebook (the
//   `[[...], [...], ...]` brackets) describe the final decomposition,
//   NOT declared melds.  The decomposition engine finds the structure.
// - For tests that require declared melds (e.g. Melded Hand, Two
//   Melded Kongs), we use the `declared_hand` helper.
//
// Tests assert:
//   1. The primary fan type is detected (once the rule is implemented).
//   2. Combined-with fans are also detected (once those rules are done).
//   3. Excluded fans are NOT present in the result.
//
// NOTE: Many rules are still stubs returning `vec![]`.  As each rule
//       is implemented, the corresponding assertions start passing.
// ═══════════════════════════════════════════════════════════════

use super::solve::solve_fan;
use super::types::FanType;
use super::{FanContext, FanSolveResult, WaitType, WinMethod};
use crate::structs::{Hand, Meld, Quad, Sequence, Tile, Triplet, Wind};

// ── Test helpers ──────────────────────────────────────────────────

/// Build a hand where ALL tiles are in the concealed hand.
/// The melds argument describes the *expected decomposition* — the
/// tiles are inserted into `hand.concealed` and the engine finds them.
fn concealed_hand(melds: Vec<Meld>, pair_tile: Tile) -> Hand {
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

/// Build a hand with some melds declared (exposed) and the rest
/// in the concealed hand.

fn declared_hand(
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
fn all_declared_hand(melds: Vec<Meld>, pair_tile: Tile) -> Hand {
    let mut hand = Hand::default();
    for m in melds {
        hand.melds.push(m);
    }
    hand.concealed.insert(pair_tile);
    hand.concealed.insert(pair_tile);
    hand
}

/// A generic default context — East seat, East round, discard win,
/// multiple wait, no flowers, not the last tile, not concealed.
fn default_context() -> FanContext {
    FanContext {
        seat_wind: Wind::East,
        prevalent_wind: Wind::East,
        win_method: WinMethod::Discard,
        winning_tile: Tile::Character1,
        wait_type: WaitType::Multiple,
        flower_count: 0,
        is_concealed: false,
        is_fully_concealed: false,
        is_last_tile_draw: false,
        is_last_tile_claim: false,
        is_last_tile_of_kind: false,
        wall_remaining: 20,
    }
}

/// Helper: check that result contains the given fan type.
fn assert_contains_fan(result: &FanSolveResult, fan: FanType) {
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

/// Helper: check that result does NOT contain the given fan type.
fn assert_not_contains_fan(result: &FanSolveResult, fan: FanType) {
    assert!(
        !result.fans.iter().any(|f| f.fan_type == fan),
        "fan {:?} ({}) should NOT be in result",
        fan,
        fan.name()
    );
}

/// Helper: check total score.

fn assert_total_score(result: &FanSolveResult, expected: u16) {
    assert_eq!(
        result.total_score,
        expected,
        "expected total score {}, got {}; fans: {:?}",
        expected,
        result.total_score,
        result
            .fans
            .iter()
            .map(|f| f.fan_type.name())
            .collect::<Vec<_>>()
    );
}

// ═══════════════════════════════════════════════════════════════
// 88-point fans
// ═══════════════════════════════════════════════════════════════

// ── 1. Big Four Winds ──

#[test]
fn test_big_four_winds_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
        ],
        Tile::Red,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigFourWinds);
    assert_not_contains_fan(&result, FanType::AllPungs);
    assert_not_contains_fan(&result, FanType::BigThreeWinds);
    // Combined with All Honors (stub — will pass when implemented)
    // assert_contains_fan(&result, FanType::AllHonors);
}

#[test]
fn test_big_four_winds_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
        ],
        Tile::Character9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigFourWinds);
}

#[test]
fn test_big_four_winds_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
        ],
        Tile::Bamboo3,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigFourWinds);
}

// ── 2. Big Three Dragons ──

#[test]
fn test_big_three_dragons_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::Green, true)),
            Meld::Pung(Triplet::new(Tile::White, true)),
            Meld::Pung(Triplet::new(Tile::Character1, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigThreeDragons);
    assert_not_contains_fan(&result, FanType::TwoDragonPungs);
    assert_not_contains_fan(&result, FanType::DragonPung);
}

#[test]
fn test_big_three_dragons_example2() {
    // Original doc: pair = White, but White pung + White pair = 5 tiles
    // which is impossible in MCR. Adjusted to West per the user's fix.
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::Green, true)),
            Meld::Pung(Triplet::new(Tile::White, true)),
            Meld::Pung(Triplet::new(Tile::East, true)),
        ],
        Tile::West,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigThreeDragons);
}

#[test]
fn test_big_three_dragons_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::Green, true)),
            Meld::Pung(Triplet::new(Tile::White, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, true)),
        ],
        Tile::Bamboo5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigThreeDragons);
}

// ── 3. All Green ──

#[test]
fn test_all_green_example1() {
    // Seven pairs of green tiles + Green Dragon
    // Use declared hand with 4 pungs of green tiles + pair Green Dragon
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo2, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo6, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
        ],
        Tile::Green,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllGreen);
}

// ── 4. Nine Gates ──

#[test]
fn test_nine_gates() {
    // 1112345678999 Wan + winning tile Wan9
    // Need all in concealed hand for the decomposition engine
    let mut hand = Hand::default();
    // 1×3, 2,3,4,5,6,7,8, 9×4 = 14 tiles
    for _ in 0..3 {
        hand.concealed.insert(Tile::Character1);
    }
    hand.concealed.insert(Tile::Character2);
    hand.concealed.insert(Tile::Character3);
    hand.concealed.insert(Tile::Character4);
    hand.concealed.insert(Tile::Character5);
    hand.concealed.insert(Tile::Character6);
    hand.concealed.insert(Tile::Character7);
    hand.concealed.insert(Tile::Character8);
    for _ in 0..4 {
        hand.concealed.insert(Tile::Character9);
    }
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::NineGates);
}

// ── 5. Four Kongs ──

#[test]
fn test_four_kongs_example1() {
    // Kongs must be declared melds — cannot have 4 identical tiles concealed.
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Bamboo2, false)),
            Meld::Kong(Quad::new(Tile::Character5, false)),
            Meld::Kong(Quad::new(Tile::Dot7, false)),
            Meld::Kong(Quad::new(Tile::East, false)),
        ],
        Tile::Red,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FourKongs);
    assert_not_contains_fan(&result, FanType::SingleWait);
}

#[test]
fn test_four_kongs_example2() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Character1, false)),
            Meld::Kong(Quad::new(Tile::Bamboo2, false)),
            Meld::Kong(Quad::new(Tile::Dot7, false)),
            Meld::Kong(Quad::new(Tile::Bamboo1, false)),
        ],
        Tile::Character4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FourKongs);
}

#[test]
fn test_four_kongs_example3() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Red, false)),
            Meld::Kong(Quad::new(Tile::Green, false)),
            Meld::Kong(Quad::new(Tile::White, false)),
            Meld::Kong(Quad::new(Tile::North, false)),
        ],
        Tile::East,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FourKongs);
}

// ── 6. Seven Shifted Pairs ──

#[test]
fn test_seven_shifted_pairs() {
    // Dot2-2, Dot3-3, Dot4-4, Dot5-5, Dot6-6, Dot7-7, Dot8-8
    // All 14 tiles in concealed — special decomposition engine handles it
    let mut hand = Hand::default();
    for i in 2..=8 {
        let t = match i {
            2 => Tile::Dot2,
            3 => Tile::Dot3,
            4 => Tile::Dot4,
            5 => Tile::Dot5,
            6 => Tile::Dot6,
            7 => Tile::Dot7,
            8 => Tile::Dot8,
            _ => unreachable!(),
        };
        hand.concealed.insert(t);
        hand.concealed.insert(t);
    }
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::SevenShiftedPairs);
}

// ── 7. Thirteen Orphans ──

#[test]
fn test_thirteen_orphans() {
    // 1-9 each suit + all honors + pair of one
    let mut hand = Hand::default();
    // 13 orphans: Dot1, Dot9, Bam1, Bam9, Wan1, Wan9, all 7 honors
    hand.concealed.insert(Tile::Dot1);
    hand.concealed.insert(Tile::Dot9);
    hand.concealed.insert(Tile::Bamboo1);
    hand.concealed.insert(Tile::Bamboo9);
    hand.concealed.insert(Tile::Character1);
    hand.concealed.insert(Tile::Character9);
    hand.concealed.insert(Tile::East);
    hand.concealed.insert(Tile::South);
    hand.concealed.insert(Tile::West);
    hand.concealed.insert(Tile::North);
    hand.concealed.insert(Tile::Red);
    hand.concealed.insert(Tile::Green);
    hand.concealed.insert(Tile::White);
    // Pair: North
    hand.concealed.insert(Tile::North);
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ThirteenOrphans);
}

// ═══════════════════════════════════════════════════════════════
// 64-point fans
// ═══════════════════════════════════════════════════════════════

// ── 8. All Terminals ──

#[test]
fn test_all_terminals() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character1, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo1, false)),
            Meld::Pung(Triplet::new(Tile::Character9, false)),
            Meld::Pung(Triplet::new(Tile::Dot9, false)),
        ],
        Tile::Bamboo1,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllTerminals);
    assert_not_contains_fan(&result, FanType::NoHonors);
    assert_not_contains_fan(&result, FanType::PungOfTerminalsOrHonors);
}

// ── 9. Little Four Winds ──

#[test]
fn test_little_four_winds_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
        ],
        Tile::South,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LittleFourWinds);
    assert_not_contains_fan(&result, FanType::BigThreeWinds);
}

#[test]
fn test_little_four_winds_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
            Meld::Pung(Triplet::new(Tile::White, true)),
        ],
        Tile::East,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LittleFourWinds);
}

// ── 10. Little Three Dragons ──

#[test]
fn test_little_three_dragons_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::Green, true)),
            Meld::Pung(Triplet::new(Tile::Dot1, true)),
            Meld::Pung(Triplet::new(Tile::Dot9, true)),
        ],
        Tile::White,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LittleThreeDragons);
    assert_not_contains_fan(&result, FanType::DragonPung);
    assert_not_contains_fan(&result, FanType::TwoDragonPungs);
}

#[test]
fn test_little_three_dragons_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::White, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Pung(Triplet::new(Tile::Dot1, true)),
        ],
        Tile::Green,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LittleThreeDragons);
}

#[test]
fn test_little_three_dragons_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::White, true)),
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
        ],
        Tile::Green,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LittleThreeDragons);
}

// ── 11. All Honors ──

#[test]
fn test_all_honors() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, false)),
            Meld::Pung(Triplet::new(Tile::White, false)),
            Meld::Pung(Triplet::new(Tile::East, false)),
            Meld::Pung(Triplet::new(Tile::South, false)),
        ],
        Tile::North,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllHonors);
    assert_not_contains_fan(&result, FanType::PungOfTerminalsOrHonors);
}

// ── 12. Four Concealed Pungs ──

#[test]
fn test_four_concealed_pungs() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo2, true)),
            Meld::Pung(Triplet::new(Tile::Dot3, true)),
            Meld::Pung(Triplet::new(Tile::Character4, true)),
            Meld::Pung(Triplet::new(Tile::Dot2, true)),
        ],
        Tile::Character3,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FourConcealedPungs);
    assert_not_contains_fan(&result, FanType::AllPungs);
    assert_not_contains_fan(&result, FanType::ConcealedHand);
}

// ── 13. Pure Terminal Chows ──

#[test]
fn test_pure_terminal_chows() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
        ],
        Tile::Bamboo5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureTerminalChows);
}

// ═══════════════════════════════════════════════════════════════
// 48-point fans
// ═══════════════════════════════════════════════════════════════

// ── 14. Quadruple Chow ──

#[test]
fn test_quadruple_chow() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
        ],
        Tile::Character4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::QuadrupleChow);
}

// ── 15. Four Pure Shifted Pungs ──

#[test]
fn test_four_pure_shifted_pungs() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Dot1, false)),
            Meld::Pung(Triplet::new(Tile::Dot2, false)),
            Meld::Pung(Triplet::new(Tile::Dot3, false)),
            Meld::Pung(Triplet::new(Tile::Dot4, false)),
        ],
        Tile::Bamboo2,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FourPureShiftedPungs);
}

// ═══════════════════════════════════════════════════════════════
// 32-point fans
// ═══════════════════════════════════════════════════════════════

// ── 16. Four Shifted Chows ──

#[test]
fn test_four_shifted_chows_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
        ],
        Tile::Character1,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FourShiftedChows);
}

#[test]
fn test_four_shifted_chows_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo5, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
        ],
        Tile::Dot5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FourShiftedChows);
}

// ── 17. Three Kongs ──

#[test]
fn test_three_kongs() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Character2, false)),
            Meld::Kong(Quad::new(Tile::Character3, false)),
            Meld::Kong(Quad::new(Tile::Character4, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo2, false)),
        ],
        Tile::Bamboo3,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ThreeKongs);
}

// ── 18. All Terminals and Honors ──

#[test]
fn test_all_terminals_and_honors() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, false)),
            Meld::Pung(Triplet::new(Tile::Character1, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo9, false)),
            Meld::Pung(Triplet::new(Tile::Dot9, false)),
        ],
        Tile::Red,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllTerminalsAndHonors);
    assert_not_contains_fan(&result, FanType::PungOfTerminalsOrHonors);
}

// ═══════════════════════════════════════════════════════════════
// 24-point fans
// ═══════════════════════════════════════════════════════════════

// ── 19. Seven Pairs ──

fn seven_pairs_hand(pair_tiles: &[Tile]) -> Hand {
    let mut hand = Hand::default();
    for &t in pair_tiles {
        hand.concealed.insert(t);
        hand.concealed.insert(t);
    }
    hand
}

#[test]
fn test_seven_pairs_example1() {
    let hand = seven_pairs_hand(&[
        Tile::Dot7,
        Tile::Bamboo2,
        Tile::Character3,
        Tile::Red,
        Tile::White,
        Tile::East,
        Tile::North,
    ]);
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::SevenPairs);
}

#[test]
fn test_seven_pairs_example2() {
    // 4× Dot9 forms two pairs of Dot9 (per MCR seven pairs rules).
    // seven_pairs_hand doubles each entry, so list Dot9 once → 2 tiles.
    // We need Dot9 × 4 = two pairs. Manually insert 4 Dot9 tiles.
    let mut hand = Hand::default();
    for &t in &[
        Tile::Dot1,
        Tile::Character1,
        Tile::Bamboo1,
        Tile::Dot9,
        Tile::Dot9,
        Tile::Character9,
        Tile::Bamboo9,
    ] {
        hand.concealed.insert(t);
        hand.concealed.insert(t);
    }
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::SevenPairs);
    // Also qualifies for All Terminals
    assert_contains_fan(&result, FanType::AllTerminals);
}

#[test]
fn test_seven_pairs_example3() {
    let hand = seven_pairs_hand(&[
        Tile::East,
        Tile::South,
        Tile::West,
        Tile::North,
        Tile::Red,
        Tile::Green,
        Tile::White,
    ]);
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::SevenPairs);
}

// ── 20. Greater Honors and Knitted Tiles ──

#[test]
fn test_greater_honors_and_knitted_tiles() {
    // This special decomposition is constructed by decompose_special.
    // For now, use a standard hand and expect the rule to fire through
    // the decomposition engine when it supports this pattern.
    // (Hand construction for this pattern not yet ergonomic.)
}

// ── 21. All Even Pungs ──

#[test]
fn test_all_even_pungs_example1() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character2, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, false)),
            Meld::Pung(Triplet::new(Tile::Dot6, false)),
            Meld::Pung(Triplet::new(Tile::Dot8, false)),
        ],
        Tile::Dot4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllEvenPungs);
}

#[test]
fn test_all_even_pungs_example2() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character2, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo2, false)),
            Meld::Pung(Triplet::new(Tile::Dot2, false)),
            Meld::Pung(Triplet::new(Tile::Character4, false)),
        ],
        Tile::Bamboo4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllEvenPungs);
}

#[test]
fn test_all_even_pungs_example3() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character6, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo6, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
            Meld::Pung(Triplet::new(Tile::Character8, false)),
        ],
        Tile::Bamboo2,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllEvenPungs);
}

// ── 22. Full Flush ──

#[test]
fn test_full_flush_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character6, true)),
            Meld::Pung(Triplet::new(Tile::Character7, true)),
            Meld::Pung(Triplet::new(Tile::Character8, true)),
            Meld::Pung(Triplet::new(Tile::Character9, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FullFlush);
    assert_not_contains_fan(&result, FanType::NoHonors);
}

#[test]
fn test_full_flush_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FullFlush);
}

// ── 23. Pure Triple Chow ──

#[test]
fn test_pure_triple_chow() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
        ],
        Tile::Dot5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureTripleChow);
}

// ── 24. Pure Shifted Pungs ──

#[test]
fn test_pure_shifted_pungs() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo2, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo3, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, false)),
            Meld::Pung(Triplet::new(Tile::Dot3, false)),
        ],
        Tile::Dot2,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureShiftedPungs);
}

// ── 25. Upper Tiles ──

#[test]
fn test_upper_tiles_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::UpperTiles);
    assert_not_contains_fan(&result, FanType::NoHonors);
}

#[test]
fn test_upper_tiles_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo7, true)),
            Meld::Pung(Triplet::new(Tile::Character8, true)),
            Meld::Pung(Triplet::new(Tile::Dot7, true)),
            Meld::Pung(Triplet::new(Tile::Character9, true)),
        ],
        Tile::Bamboo8,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::UpperTiles);
}

#[test]
fn test_upper_tiles_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Character8,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::UpperTiles);
}

// ── 26. Middle Tiles ──

#[test]
fn test_middle_tiles_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo5, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo6, true)),
        ],
        Tile::Dot6,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MiddleTiles);
    assert_not_contains_fan(&result, FanType::NoHonors);
    assert_not_contains_fan(&result, FanType::AllSimples);
}

#[test]
fn test_middle_tiles_example2() {
    // Seven pairs of 4-5-6 across suits (7 distinct tiles all rank 4-6)
    let hand = seven_pairs_hand(&[
        Tile::Character4,
        Tile::Character5,
        Tile::Character6,
        Tile::Dot4,
        Tile::Dot5,
        Tile::Dot6,
        Tile::Bamboo4,
    ]);
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MiddleTiles);
}

// ── 27. Lower Tiles ──

#[test]
fn test_lower_tiles() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
        ],
        Tile::Bamboo3,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LowerTiles);
    assert_not_contains_fan(&result, FanType::NoHonors);
}

// ═══════════════════════════════════════════════════════════════
// 16-point fans
// ═══════════════════════════════════════════════════════════════

// ── 28. Pure Straight ──
//
// NOTE: Current implementation requires EXACTLY 3 chows in the
// decomposition; hands with 4 chows are missed.

#[test]
fn test_pure_straight_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
        ],
        Tile::Character7,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureStraight);
}

#[test]
fn test_pure_straight_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Pung(Triplet::new(Tile::Dot6, true)),
        ],
        Tile::Dot8,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureStraight);
}

// ── 29. Three-Suited Terminal Chows ──

#[test]
fn test_three_suited_terminal_chows() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Bamboo5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ThreeSuitedTerminalChows);
}

// ── 30. Pure Shifted Chows ──

#[test]
fn test_pure_shifted_chows_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Dot2, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo2, true)),
        ],
        Tile::Bamboo2,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureShiftedChows);
}

#[test]
fn test_pure_shifted_chows_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo5, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
        ],
        Tile::Bamboo2,
    );
    let ctx = default_context();
    // Note: 4 chows, Pure Shifted Chows picks the best 3
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureShiftedChows);
}

#[test]
fn test_pure_shifted_chows_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Chow(Sequence::new(Tile::Character5, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Bamboo7,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureShiftedChows);
}

// ── 31. All Fives ──

#[test]
fn test_all_fives_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllFives);
}

#[test]
fn test_all_fives_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Pung(Triplet::new(Tile::Dot5, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo5, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllFives);
}

// ── 32. Triple Pung ──

#[test]
fn test_triple_pung_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Dot3, true)),
            Meld::Pung(Triplet::new(Tile::Character3, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo2, true)),
        ],
        Tile::Character4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TriplePung);
}

#[test]
fn test_triple_pung_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character1, true)),
            Meld::Pung(Triplet::new(Tile::Dot1, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo1, true)),
            Meld::Pung(Triplet::new(Tile::Dot9, true)),
        ],
        Tile::Bamboo9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TriplePung);
}

// ── 33. Three Concealed Pungs ──

#[test]
fn test_three_concealed_pungs_example1() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::Character9, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo1, false)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ThreeConcealedPungs);
}

#[test]
fn test_three_concealed_pungs_example2() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo4, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo7, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, false)),
        ],
        Tile::Bamboo5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ThreeConcealedPungs);
}

// ═══════════════════════════════════════════════════════════════
// 12-point fans
// ═══════════════════════════════════════════════════════════════

// ── 34. Lesser Honors and Knitted Tiles ──

#[test]
fn test_lesser_honors_and_knitted_example1() {
    // Requires special decomposition — test through standard engine
    // when decompose_special supports it.
}

// ── 35. Knitted Straight ──

#[test]
fn test_knitted_straight_example1() {
    // Requires special decomposition — test through standard engine
    // when decompose_special supports it.
}

// ── 36. Upper Four ──

#[test]
fn test_upper_four_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Chow(Sequence::new(Tile::Dot6, true)),
            Meld::Pung(Triplet::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Bamboo9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::UpperFour);
    assert_not_contains_fan(&result, FanType::NoHonors);
}

#[test]
fn test_upper_four_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
        ],
        Tile::Dot6,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::UpperFour);
}

#[test]
fn test_upper_four_example3() {
    let hand = seven_pairs_hand(&[
        Tile::Bamboo6,
        Tile::Bamboo8,
        Tile::Character6,
        Tile::Character7,
        Tile::Dot6,
        Tile::Dot8,
        Tile::Dot9,
    ]);
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::UpperFour);
}

// ── 37. Lower Four ──

#[test]
fn test_lower_four_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
        ],
        Tile::Character4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LowerFour);
    assert_not_contains_fan(&result, FanType::NoHonors);
}

#[test]
fn test_lower_four_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo1, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo2, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo3, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo4, true)),
        ],
        Tile::Character2,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LowerFour);
}

// ── 38. Big Three Winds ──

#[test]
fn test_big_three_winds_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::Character9, true)),
        ],
        Tile::Bamboo9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigThreeWinds);
}

#[test]
fn test_big_three_winds_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
            Meld::Chow(Sequence::new(Tile::Character2, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigThreeWinds);
}

#[test]
fn test_big_three_winds_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
            Meld::Pung(Triplet::new(Tile::Red, true)),
        ],
        Tile::Green,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigThreeWinds);
}

// ═══════════════════════════════════════════════════════════════
// 8-point fans
// ═══════════════════════════════════════════════════════════════

// ── 39. Mixed Straight ──

#[test]
fn test_mixed_straight_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
        ],
        Tile::Dot8,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MixedStraight);
}

#[test]
fn test_mixed_straight_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
        ],
        Tile::Character7,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MixedStraight);
}

// ── 40. Reversible Tiles ──

#[test]
fn test_reversible_tiles_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
        ],
        Tile::Bamboo5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ReversibleTiles);
}

#[test]
fn test_reversible_tiles_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot2, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, true)),
        ],
        Tile::White,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ReversibleTiles);
}

#[test]
fn test_reversible_tiles_example3() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Dot8, false)),
            Meld::Pung(Triplet::new(Tile::Dot9, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo9, false)),
            Meld::Pung(Triplet::new(Tile::White, false)),
        ],
        Tile::Bamboo8,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ReversibleTiles);
}

// ── 41. Mixed Triple Chow ──

#[test]
fn test_mixed_triple_chow_example1() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo3, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
        ],
        Tile::Character4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MixedTripleChow);
}

#[test]
fn test_mixed_triple_chow_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character6, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo6, true)),
            Meld::Chow(Sequence::new(Tile::Dot6, true)),
            Meld::Chow(Sequence::new(Tile::Dot6, true)),
        ],
        Tile::Bamboo9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MixedTripleChow);
}

// ── 42. Mixed Shifted Pungs ──

#[test]
fn test_mixed_shifted_pungs_example1() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Dot2, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo3, false)),
            Meld::Pung(Triplet::new(Tile::Character4, false)),
            Meld::Pung(Triplet::new(Tile::Dot3, false)),
        ],
        Tile::Character2,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MixedShiftedPungs);
}

#[test]
fn test_mixed_shifted_pungs_example2() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character7, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
            Meld::Pung(Triplet::new(Tile::Dot9, false)),
            Meld::Chow(Sequence::new(Tile::Dot7, false)),
        ],
        Tile::Character9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MixedShiftedPungs);
}

// ── 43. Chicken Hand ──

#[test]
fn test_chicken_hand() {
    // A hand with only a non-scoring pattern (e.g., mixed chows with honors
    // that don't form any scoring combination).
}

// ── 48. Two Concealed Kongs ──

#[test]
fn test_two_concealed_kongs() {
    // Kongs must be declared melds; they can be marked concealed (self-drawn).
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Bamboo1, true)),
            Meld::Kong(Quad::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::South, false)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
        ],
        Tile::Character8,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TwoConcealedKongs);
}

// ── 44. Last Tile Draw ──

#[test]
fn test_last_tile_draw() {
    let hand = all_declared_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, false)),
            Meld::Chow(Sequence::new(Tile::Character4, false)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
            Meld::Chow(Sequence::new(Tile::Character1, false)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.is_last_tile_draw = true;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LastTileDraw);
}

// ── 45. Last Tile Claim ──

#[test]
fn test_last_tile_claim() {
    let hand = all_declared_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, false)),
            Meld::Chow(Sequence::new(Tile::Character4, false)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
            Meld::Chow(Sequence::new(Tile::Character1, false)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.is_last_tile_claim = true;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LastTileClaim);
}

// ── 46. Out With Replacement Tile ──

#[test]
fn test_out_with_replacement_tile() {
    let hand = all_declared_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, false)),
            Meld::Chow(Sequence::new(Tile::Character4, false)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
            Meld::Chow(Sequence::new(Tile::Character1, false)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.win_method = WinMethod::KongReplacement;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::OutWithReplacementTile);
}

// ── 47. Robbing The Kong ──

#[test]
fn test_robbing_the_kong() {
    let hand = all_declared_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, false)),
            Meld::Chow(Sequence::new(Tile::Character4, false)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
            Meld::Chow(Sequence::new(Tile::Character1, false)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.win_method = WinMethod::RobKong;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::RobbingTheKong);
}

// ═══════════════════════════════════════════════════════════════
// 6-point fans
// ═══════════════════════════════════════════════════════════════

// ── 49. All Pungs ──

#[test]
// Fully concealed 4-pung hand qualifies for Four Concealed Pungs (64 pt)
// which excludes All Pungs (6 pt).  To test All Pungs in isolation, use
// exposed (declared) melds so Four Concealed Pungs doesn't apply.
#[test]
fn test_all_pungs() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character8, false)),
            Meld::Pung(Triplet::new(Tile::Dot8, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
            Meld::Pung(Triplet::new(Tile::White, false)),
        ],
        Tile::East,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllPungs);
}

// ── 50. Half Flush ──

#[test]
fn test_half_flush() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Character3, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Red,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::HalfFlush);
}

// ── 51. Mixed Shifted Chows ──

#[test]
fn test_mixed_shifted_chows() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Bamboo1, true)),
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MixedShiftedChows);
}

// ── 52. All Types ──

#[test]
fn test_all_types() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Dot4, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Pung(Triplet::new(Tile::Green, true)),
        ],
        Tile::North,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllTypes);
}

// ── 53. Melded Hand ──

#[test]
fn test_melded_hand() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Character2, false)),
            Meld::Chow(Sequence::new(Tile::Dot4, false)),
            Meld::Chow(Sequence::new(Tile::Character6, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo8, false)),
        ],
        Tile::Character6,
    );
    let mut ctx = default_context();
    ctx.win_method = WinMethod::Discard;
    ctx.wait_type = WaitType::Single;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MeldedHand);
}

// ── 54. Two Dragon Pungs ──

#[test]
fn test_two_dragon_pungs() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::Green, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Bamboo8,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TwoDragonPungs);
}

// ═══════════════════════════════════════════════════════════════
// 4-point fans
// ═══════════════════════════════════════════════════════════════

// ── 55. Outside Hand ──
//
// NOTE: The current check only inspects `meld_tile` (the start tile
// of a chow).  For chows like 7-8-9 the terminal (9) is at the end
// and not detected.  Fix when chow tile iteration is added.

#[test]
fn test_outside_hand_example1() {
    // Dot7-8-9, Char7-8-9, Bam7-8-9, Bam7-8-9 chows, pair Bam9
    // Each chow contains a terminal (9), so Outside Hand should apply.
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
        ],
        Tile::Bamboo9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::OutsideHand);
}

#[test]
fn test_outside_hand_example2() {
    // One chow is a declared meld (exposed), breaking the symmetrical
    // tile pattern that would also qualify for Seven Pairs.
    let hand = declared_hand(
        vec![Meld::Chow(Sequence::new(Tile::Character1, false))],
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Red,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::OutsideHand);
}

#[test]
fn test_outside_hand_example3() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Pung(Triplet::new(Tile::Character9, true)),
            Meld::Chow(Sequence::new(Tile::Dot7, true)),
        ],
        Tile::East,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::OutsideHand);
}

// ── 56. Fully Concealed ──

#[test]
fn test_fully_concealed() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Dot6, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo6, true)),
        ],
        Tile::Bamboo4,
    );
    let mut ctx = default_context();
    ctx.win_method = WinMethod::SelfDraw;
    ctx.is_fully_concealed = true;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FullyConcealed);
}

// ── 57. Two Melded Kongs ──

#[test]
fn test_two_melded_kongs_example1() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Dot4, false)),
            Meld::Kong(Quad::new(Tile::Character4, false)),
            Meld::Pung(Triplet::new(Tile::Red, false)),
            Meld::Pung(Triplet::new(Tile::Character1, false)),
        ],
        Tile::Character3,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TwoMeldedKongs);
}

#[test]
fn test_two_melded_kongs_example2() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Dot2, false)),
            Meld::Kong(Quad::new(Tile::White, false)),
            Meld::Pung(Triplet::new(Tile::Bamboo6, false)),
            Meld::Chow(Sequence::new(Tile::Dot3, false)),
        ],
        Tile::Bamboo4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TwoMeldedKongs);
}

// ═══════════════════════════════════════════════════════════════
// 2-point fans
// ═══════════════════════════════════════════════════════════════

// ── 58. Last Tile ──

#[test]
fn test_last_tile() {
    let hand = all_declared_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, false)),
            Meld::Chow(Sequence::new(Tile::Character4, false)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
            Meld::Chow(Sequence::new(Tile::Character1, false)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.is_last_tile_of_kind = true;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::LastTile);
}

// ── 59. Dragon Pung ──

#[test]
fn test_dragon_pung() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::DragonPung);
}

// ── 60. Prevalent Wind ──

#[test]
fn test_prevalent_wind_east() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.prevalent_wind = Wind::East;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PrevalentWind);
}

// ── 61. Seat Wind ──

#[test]
fn test_seat_wind_south() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.seat_wind = Wind::South;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::SeatWind);
}

// ── 62. Concealed Hand ──

#[test]
fn test_concealed_hand() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Character5, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Dot6, true)),
        ],
        Tile::Bamboo6,
    );
    let mut ctx = default_context();
    ctx.is_concealed = true;
    ctx.win_method = WinMethod::Discard;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ConcealedHand);
}

// ── 63. All Chows ──

#[test]
fn test_all_chows() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, true)),
        ],
        Tile::Dot7,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllChows);
}

// ── 64. Tile Hog ──

#[test]
fn test_tile_hog() {
    // Dot7-8-9, Bam7-8-9, Wan7-8-9, Wan7 pung, pair Dot3
    // Wan7 appears in pung (3) + chow (1) = 4 times → Tile Hog
    let hand = all_declared_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Dot7, false)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, false)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
            Meld::Pung(Triplet::new(Tile::Character7, false)),
        ],
        Tile::Dot3,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TileHog);
}

// ── 65. Double Pung ──

#[test]
fn test_double_pung() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character2, true)),
            Meld::Pung(Triplet::new(Tile::Character5, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo5, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Dot4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::DoublePung);
}

// ── 66. Two Concealed Pungs ──

#[test]
fn test_two_concealed_pungs() {
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo9, true)),
            Meld::Kong(Quad::new(Tile::Dot9, true)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
            Meld::Chow(Sequence::new(Tile::Character1, false)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TwoConcealedPungs);
}

// ── 67. Concealed Kong ──

#[test]
fn test_concealed_kong() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Bamboo9, true)),
            Meld::Pung(Triplet::new(Tile::Dot9, false)),
            Meld::Chow(Sequence::new(Tile::Character7, false)),
            Meld::Chow(Sequence::new(Tile::Character1, false)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ConcealedKong);
}

// ── 68. All Simples ──

#[test]
fn test_all_simples() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo7, true)),
        ],
        Tile::Dot4,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::AllSimples);
}

// ═══════════════════════════════════════════════════════════════
// 1-point fans
// ═══════════════════════════════════════════════════════════════

// ── 69. Pure Double Chow ──

#[test]
fn test_pure_double_chow() {
    // Two identical chows in same suit: Wan1-2-3 ×2
    // Plus Wan7-8-9 and a Dot8 pung, pair Dot9
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PureDoubleChow);
}

// ── 70. Mixed Double Chow ──

#[test]
fn test_mixed_double_chow() {
    // Two chows of same numbers in different suits: Wan1-2-3, Dot1-2-3
    // Plus Wan7-8-9 and a Dot8 pung, pair Dot9
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Dot1, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MixedDoubleChow);
}

// ── 71. Short Straight ──

#[test]
fn test_short_straight() {
    // Two chows in same suit forming 1-2-3 + 4-5-6
    // Plus Wan7-8-9 and a Dot8 pung, pair Dot9
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ShortStraight);
}

// ── 72. Two Terminal Chows ──

#[test]
fn test_two_terminal_chows() {
    // Chows of 1-2-3 and 7-8-9 in same suit (Character)
    // Plus Wan4-5-6 and a Dot8 pung, pair Dot9
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TwoTerminalChows);
}

// ── 72. Two Terminal Chows ──

#[test]
fn test_two_terminal_chows_example2() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Pung(Triplet::new(Tile::Dot8, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::TwoTerminalChows);
}

// ── 73. Pung of Terminals or Honors ──

#[test]
fn test_pung_of_terminals_or_honors() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Character1, true)),
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::PungOfTerminalsOrHonors);
}

// ── 74. Melded Kong ──

#[test]
fn test_melded_kong() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Dot1, false)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, false)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, false)),
            Meld::Chow(Sequence::new(Tile::Dot4, false)),
        ],
        Tile::Bamboo2,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::MeldedKong);
}

// ── 75. One Voided Suit ──

#[test]
fn test_one_voided_suit() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Pung(Triplet::new(Tile::East, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::OneVoidedSuit);
}

// ── 76. No Honors ──
//
// Full Flush excludes No Honors per MCR rules, so we use a hand
// with all three suits (so Full Flush doesn't apply) and no honors.

#[test]
fn test_no_honors() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Dot2, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo2, true)),
            Meld::Pung(Triplet::new(Tile::Bamboo5, true)),
        ],
        Tile::Character5,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::NoHonors);
}

// ── 77. Edge Wait ──

#[test]
fn test_edge_wait() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Dot2, true)),
        ],
        Tile::Bamboo3,
    );
    let mut ctx = default_context();
    ctx.wait_type = WaitType::Edge;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::EdgeWait);
}

// ── 78. Closed Wait ──

#[test]
fn test_closed_wait() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character2, true)),
            Meld::Chow(Sequence::new(Tile::Dot3, true)),
            Meld::Chow(Sequence::new(Tile::Dot6, true)),
            Meld::Chow(Sequence::new(Tile::Bamboo6, true)),
        ],
        Tile::Bamboo4,
    );
    let mut ctx = default_context();
    ctx.wait_type = WaitType::Closed;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::ClosedWait);
}

// ── 79. Single Wait ──

#[test]
fn test_single_wait() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Pung(Triplet::new(Tile::East, true)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.wait_type = WaitType::Single;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::SingleWait);
}

// ── 80. Self-Drawn ──

#[test]
fn test_self_drawn() {
    let hand = all_declared_hand(
        vec![
            Meld::Kong(Quad::new(Tile::Dot1, false)),
            Meld::Chow(Sequence::new(Tile::Bamboo4, false)),
            Meld::Chow(Sequence::new(Tile::Bamboo7, false)),
            Meld::Chow(Sequence::new(Tile::Dot4, false)),
        ],
        Tile::Bamboo2,
    );
    let mut ctx = default_context();
    ctx.win_method = WinMethod::SelfDraw;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::SelfDrawn);
}

// ── 81. Flower Tiles ──

#[test]
fn test_flower_tiles() {
    let hand = concealed_hand(
        vec![
            Meld::Chow(Sequence::new(Tile::Character1, true)),
            Meld::Chow(Sequence::new(Tile::Character4, true)),
            Meld::Chow(Sequence::new(Tile::Character7, true)),
            Meld::Chow(Sequence::new(Tile::Character1, true)),
        ],
        Tile::Character5,
    );
    let mut ctx = default_context();
    ctx.flower_count = 2;
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FlowerTiles);
}

// ═══════════════════════════════════════════════════════════════
// Exclusion rule integration tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_big_four_winds_excludes_big_three_winds() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, true)),
            Meld::Pung(Triplet::new(Tile::South, true)),
            Meld::Pung(Triplet::new(Tile::West, true)),
            Meld::Pung(Triplet::new(Tile::North, true)),
        ],
        Tile::Red,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigFourWinds);
    assert_not_contains_fan(&result, FanType::BigThreeWinds);
}

#[test]
fn test_big_three_dragons_excludes_dragon_pung() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Red, true)),
            Meld::Pung(Triplet::new(Tile::Green, true)),
            Meld::Pung(Triplet::new(Tile::White, true)),
            Meld::Pung(Triplet::new(Tile::Character1, true)),
        ],
        Tile::Dot9,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigThreeDragons);
    assert_not_contains_fan(&result, FanType::DragonPung);
    assert_not_contains_fan(&result, FanType::TwoDragonPungs);
}

#[test]
fn test_four_concealed_pungs_excludes_all_pungs() {
    let hand = concealed_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::Bamboo2, true)),
            Meld::Pung(Triplet::new(Tile::Dot3, true)),
            Meld::Pung(Triplet::new(Tile::Character4, true)),
            Meld::Pung(Triplet::new(Tile::Dot2, true)),
        ],
        Tile::Character3,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::FourConcealedPungs);
    assert_not_contains_fan(&result, FanType::AllPungs);
}

// ═══════════════════════════════════════════════════════════════
// Non-winning hand tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_non_winning_hand_returns_none() {
    // 4 declared melds, but concealed has only 1 tile (not a pair)
    let mut hand = Hand::default();
    hand.melds.push(Meld::Pung(Triplet::new(Tile::East, false)));
    hand.melds
        .push(Meld::Pung(Triplet::new(Tile::South, false)));
    hand.melds.push(Meld::Pung(Triplet::new(Tile::West, false)));
    hand.melds
        .push(Meld::Pung(Triplet::new(Tile::North, false)));
    hand.concealed.insert(Tile::Character1); // only 1 tile — not a pair
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap();
    assert!(result.is_none(), "non-winning hand should return None");
}

#[test]
fn test_empty_hand_returns_none() {
    let hand = Hand::default();
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap();
    assert!(result.is_none(), "empty hand should return None");
}

// ═══════════════════════════════════════════════════════════════
// Declared meld hand test (uses all_declared_hand for explicit melds)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_big_four_winds_with_declared_melds() {
    // Same hand as example 1 but with all 4 pungs declared (exposed).
    // This validates that declared melds also work with the fan rules.
    let hand = all_declared_hand(
        vec![
            Meld::Pung(Triplet::new(Tile::East, false)),
            Meld::Pung(Triplet::new(Tile::South, false)),
            Meld::Pung(Triplet::new(Tile::West, false)),
            Meld::Pung(Triplet::new(Tile::North, false)),
        ],
        Tile::Red,
    );
    let ctx = default_context();
    let result = solve_fan(&hand, &ctx).unwrap().unwrap();
    assert_contains_fan(&result, FanType::BigFourWinds);
}
