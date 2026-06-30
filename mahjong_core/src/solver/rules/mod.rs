use super::FanInstance;
use super::{DecomposeResult, DynamicFanContext, StaticFanContext, WaitType};

// ============================================================================
// Type aliases
// ============================================================================

/// Signature for a rule check function.
pub(crate) type RuleFn = fn(
    &DecomposeResult,
    &StaticFanContext,
    &DynamicFanContext,
    WaitType,
) -> Vec<FanInstance>;

// ============================================================================
// Macro: mcr_rules! — single-source rule registry for all 81 MCR fan types
//
// Generates:
//   - FanType enum (81 variants, #[repr(u16)], discriminants 0..80)
//   - impl FanType { fn points(), fn name(), fn excludes_mask(), fn bit_index() }
//   - RuleEntry struct + const ALL_RULES
//   - fn check_all(profile, ctx) -> Vec<FanInstance>
// ============================================================================

macro_rules! mcr_rules {
    (
        $(
            $( #[$attr:meta] )*
            $name:ident ( $points:expr ) / $display:literal
            excludes [ $( $excl:ident ),* $(,)? ]
            => $check:path
        ),* $(,)?
    ) => {

        // ── 1. FanType enum ──
        /// All 81 MCR fan types. Discriminants are compiler-assigned 0..80
        /// and serve as stable bit indices for `FanExclusionSet`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum FanType {
            $(
                $( #[$attr] )*
                $name,
            )*
        }

        // ── 2. Associated data ──
        impl FanType {
            /// Base point value for this fan type.
            pub fn points(self) -> u8 {
                match self {
                    $( Self::$name => $points, )*
                }
            }

            /// Human-readable display name.
            pub fn name(self) -> &'static str {
                match self {
                    $( Self::$name => $display, )*
                }
            }

            /// Precomputed exclusion mask: bits set for all mutually-exclusive
            /// fan types.
            pub fn excludes_mask(self) -> FanExclusionSet {
                match self {
                    $(
                        Self::$name => {
                            let mut mask = FanExclusionSet::default();
                            $( mask.set_bit(Self::$excl as u16 as usize); )*
                            mask
                        }
                    )*
                }
            }

            /// Stable bit index for use in `FanExclusionSet`.
            #[inline]
            pub fn bit_index(self) -> usize {
                self as u16 as usize
            }
        }

        // ── 3. Registry ──
        /// A registered rule: its fan type and the check function.
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct RuleEntry {
            pub(crate) fan_type: FanType,
            pub(crate) check: RuleFn,
        }

        /// All 81 MCR rules in point-descending order (the canonical MCR order).
        pub(crate) const ALL_RULES: &[RuleEntry] = &[
            $( RuleEntry { fan_type: FanType::$name, check: $check }, )*
        ];

        /// Run all registered rules against a decomposition and context.
        /// Returns candidates with precomputed exclusion masks.
        pub(crate) fn check_all(
            decomp: &DecomposeResult,
            static_ctx: &StaticFanContext,
            dynamic_ctx: &DynamicFanContext,
            wait_type: WaitType,
        ) -> Vec<FanInstance> {
            ALL_RULES
                .iter()
                .flat_map(|entry| {
                    (entry.check)(decomp, static_ctx, dynamic_ctx, wait_type)
                })
                .collect()
        }
    };
}

// ============================================================================
// Rule registry invocation — all 81 MCR rules
//
// NOTE: Every rule currently uses `empty_rule` (the no-op stub) because none of
// the rule submodules (high, mid_high, mid_low, low) are activated yet.
// To activate a rule:
//   1. Uncomment the corresponding `mod` in the submodules section below
//   2. Change `empty_rule` → `high::big_four_winds` (etc.) in this invocation
// ============================================================================

mcr_rules! {

    // ═══════════════════════════════════════════════════════════════
    // 88-point fans
    // ═══════════════════════════════════════════════════════════════

    BigFourWinds(88) / "Big Four Winds"
        excludes [BigThreeWinds, AllPungs, PrevalentWind, SeatWind, PungOfTerminalsOrHonors]
        => high::big_four_winds,

    BigThreeDragons(88) / "Big Three Dragons"
        excludes [TwoDragonPungs, DragonPung]
        => high::big_three_dragons,

    AllGreen(88) / "All Green"
        excludes []
        => high::all_green,

    NineGates(88) / "Nine Gates"
        excludes [FullFlush, ConcealedHand, PungOfTerminalsOrHonors]
        => high::nine_gates,

    FourKongs(88) / "Four Kongs"
        excludes [SingleWait]
        => high::four_kongs,

    SevenShiftedPairs(88) / "Seven Shifted Pairs"
        excludes [FullFlush, ConcealedHand, SingleWait]
        => high::seven_shifted_pairs,

    ThirteenOrphans(88) / "Thirteen Orphans"
        excludes [AllTypes, ConcealedHand, SingleWait]
        => high::thirteen_orphans,

    // ═══════════════════════════════════════════════════════════════
    // 64-point fans
    // ═══════════════════════════════════════════════════════════════

    AllTerminals(64) / "All Terminals"
        excludes [AllPungs, OutsideHand, PungOfTerminalsOrHonors, NoHonors]
        => high::all_terminals,

    LittleFourWinds(64) / "Little Four Winds"
        excludes [BigThreeWinds, PungOfTerminalsOrHonors]
        => high::little_four_winds,

    LittleThreeDragons(64) / "Little Three Dragons"
        excludes [DragonPung, TwoDragonPungs]
        => high::little_three_dragons,

    AllHonors(64) / "All Honors"
        excludes [AllPungs, OutsideHand, PungOfTerminalsOrHonors]
        => high::all_honors,

    FourConcealedPungs(64) / "Four Concealed Pungs"
        excludes [AllPungs, ConcealedHand]
        => high::four_concealed_pungs,

    PureTerminalChows(64) / "Pure Terminal Chows"
        excludes [SevenPairs, FullFlush, AllChows, PureDoubleChow, TwoTerminalChows]
        => high::pure_terminal_chows,

    // ═══════════════════════════════════════════════════════════════
    // 48-point fans
    // ═══════════════════════════════════════════════════════════════

    QuadrupleChow(48) / "Quadruple Chow"
        excludes [PureShiftedPungs, TileHog, PureDoubleChow]
        => high::quadruple_chow,

    FourPureShiftedPungs(48) / "Four Pure Shifted Pungs"
        excludes [PureTripleChow, AllPungs]
        => high::four_pure_shifted_pungs,

    // ═══════════════════════════════════════════════════════════════
    // 32-point fans
    // ═══════════════════════════════════════════════════════════════

    FourShiftedChows(32) / "Four Shifted Chows"
        excludes [ShortStraight]
        => mid_high::four_shifted_chows,

    ThreeKongs(32) / "Three Kongs"
        excludes []
        => mid_high::three_kongs,

    AllTerminalsAndHonors(32) / "All Terminals and Honors"
        excludes [AllPungs, PungOfTerminalsOrHonors]
        => mid_high::all_terminals_and_honors,

    // ═══════════════════════════════════════════════════════════════
    // 24-point fans
    // ═══════════════════════════════════════════════════════════════

    SevenPairs(24) / "Seven Pairs"
        excludes [ConcealedHand, SingleWait]
        => mid_high::seven_pairs,

    GreaterHonorsAndKnittedTiles(24) / "Greater Honors and Knitted Tiles"
        excludes []
        => empty_rule,

    AllEvenPungs(24) / "All Even Pungs"
        excludes [AllPungs, AllSimples]
        => mid_high::all_even_pungs,

    FullFlush(24) / "Full Flush"
        excludes [NoHonors]
        => mid_high::full_flush,

    PureTripleChow(24) / "Pure Triple Chow"
        excludes [PureShiftedPungs, PureDoubleChow]
        => mid_high::pure_triple_chow,

    PureShiftedPungs(24) / "Pure Shifted Pungs"
        excludes [PureTripleChow]
        => mid_high::pure_shifted_pungs,

    UpperTiles(24) / "Upper Tiles"
        excludes [NoHonors]
        => mid_high::upper_tiles,

    MiddleTiles(24) / "Middle Tiles"
        excludes [NoHonors, AllSimples]
        => mid_high::middle_tiles,

    LowerTiles(24) / "Lower Tiles"
        excludes [NoHonors]
        => mid_high::lower_tiles,

    // ═══════════════════════════════════════════════════════════════
    // 16-point fans
    // ═══════════════════════════════════════════════════════════════

    PureStraight(16) / "Pure Straight"
        excludes []
        => mid_high::pure_straight,

    ThreeSuitedTerminalChows(16) / "Three-Suited Terminal Chows"
        excludes [PureDoubleChow, TwoTerminalChows, NoHonors, AllChows]
        => mid_high::three_suited_terminal_chows,

    PureShiftedChows(16) / "Pure Shifted Chows"
        excludes []
        => mid_high::pure_shifted_chows,

    AllFives(16) / "All Fives"
        excludes [AllSimples]
        => mid_high::all_fives,

    TriplePung(16) / "Triple Pung"
        excludes []
        => mid_high::triple_pung,

    ThreeConcealedPungs(16) / "Three Concealed Pungs"
        excludes []
        => mid_high::three_concealed_pungs,

    // ═══════════════════════════════════════════════════════════════
    // 12-point fans
    // ═══════════════════════════════════════════════════════════════

    LesserHonorsAndKnittedTiles(12) / "Lesser Honors and Knitted Tiles"
        excludes [AllTypes, ConcealedHand]
        => mid_low::lesser_honors_and_knitted_tiles,

    KnittedStraight(12) / "Knitted Straight"
        excludes []
        => mid_low::knitted_straight,

    UpperFour(12) / "Upper Four"
        excludes [NoHonors]
        => mid_low::upper_four,

    LowerFour(12) / "Lower Four"
        excludes [NoHonors]
        => mid_low::lower_four,

    BigThreeWinds(12) / "Big Three Winds"
        excludes []
        => mid_low::big_three_winds,

    // ═══════════════════════════════════════════════════════════════
    // 8-point fans
    // ═══════════════════════════════════════════════════════════════

    MixedStraight(8) / "Mixed Straight"
        excludes []
        => mid_low::mixed_straight,

    ReversibleTiles(8) / "Reversible Tiles"
        excludes [OneVoidedSuit]
        => mid_low::reversible_tiles,

    MixedTripleChow(8) / "Mixed Triple Chow"
        excludes []
        => mid_low::mixed_triple_chow,

    MixedShiftedPungs(8) / "Mixed Shifted Pungs"
        excludes []
        => mid_low::mixed_shifted_pungs,

    ChickenHand(8) / "Chicken Hand"
        excludes []
        => mid_low::chicken_hand,

    LastTileDraw(8) / "Last Tile Draw"
        excludes []
        => mid_low::last_tile_draw,

    LastTileClaim(8) / "Last Tile Claim"
        excludes []
        => mid_low::last_tile_claim,

    OutWithReplacementTile(8) / "Out With Replacement Tile"
        excludes []
        => mid_low::out_with_replacement_tile,

    RobbingTheKong(8) / "Robbing The Kong"
        excludes []
        => mid_low::robbing_the_kong,

    TwoConcealedKongs(8) / "Two Concealed Kongs"
        excludes []
        => mid_low::two_concealed_kongs,

    // ═══════════════════════════════════════════════════════════════
    // 6-point fans
    // ═══════════════════════════════════════════════════════════════

    AllPungs(6) / "All Pungs"
        excludes []
        => mid_low::all_pungs,

    HalfFlush(6) / "Half Flush"
        excludes []
        => mid_low::half_flush,

    MixedShiftedChows(6) / "Mixed Shifted Chows"
        excludes []
        => mid_low::mixed_shifted_chows,

    AllTypes(6) / "All Types"
        excludes []
        => mid_low::all_types,

    MeldedHand(6) / "Melded Hand"
        excludes [SingleWait]
        => mid_low::melded_hand,

    TwoDragonPungs(6) / "Two Dragon Pungs"
        excludes []
        => mid_low::two_dragon_pungs,

    // ═══════════════════════════════════════════════════════════════
    // 4-point fans
    // ═══════════════════════════════════════════════════════════════

    OutsideHand(4) / "Outside Hand"
        excludes []
        => low::outside_hand,

    FullyConcealed(4) / "Fully Concealed"
        excludes []
        => low::fully_concealed,

    TwoMeldedKongs(4) / "Two Melded Kongs"
        excludes []
        => low::two_melded_kongs,

    LastTile(4) / "Last Tile"
        excludes []
        => low::last_tile,

    // ═══════════════════════════════════════════════════════════════
    // 2-point fans
    // ═══════════════════════════════════════════════════════════════

    DragonPung(2) / "Dragon Pung"
        excludes []
        => low::dragon_pung,

    PrevalentWind(2) / "Prevalent Wind"
        excludes []
        => low::prevalent_wind,

    SeatWind(2) / "Seat Wind"
        excludes []
        => low::seat_wind,

    ConcealedHand(2) / "Concealed Hand"
        excludes []
        => low::concealed_hand,

    AllChows(2) / "All Chows"
        excludes []
        => low::all_chows,

    TileHog(2) / "Tile Hog"
        excludes []
        => low::tile_hog,

    DoublePung(2) / "Double Pung"
        excludes []
        => low::double_pung,

    TwoConcealedPungs(2) / "Two Concealed Pungs"
        excludes []
        => low::two_concealed_pungs,

    ConcealedKong(2) / "Concealed Kong"
        excludes []
        => low::concealed_kong,

    AllSimples(2) / "All Simples"
        excludes []
        => low::all_simples,

    // ═══════════════════════════════════════════════════════════════
    // 1-point fans
    // ═══════════════════════════════════════════════════════════════

    PureDoubleChow(1) / "Pure Double Chow"
        excludes []
        => low::pure_double_chow,

    MixedDoubleChow(1) / "Mixed Double Chow"
        excludes []
        => low::mixed_double_chow,

    ShortStraight(1) / "Short Straight"
        excludes []
        => low::short_straight,

    TwoTerminalChows(1) / "Two Terminal Chows"
        excludes []
        => low::two_terminal_chows,

    PungOfTerminalsOrHonors(1) / "Pung of Terminals or Honors"
        excludes []
        => low::pung_of_terminals_or_honors,

    MeldedKong(1) / "Melded Kong"
        excludes []
        => low::melded_kong,

    OneVoidedSuit(1) / "One Voided Suit"
        excludes []
        => low::one_voided_suit,

    NoHonors(1) / "No Honors"
        excludes []
        => low::no_honors,

    EdgeWait(1) / "Edge Wait"
        excludes []
        => low::edge_wait,

    ClosedWait(1) / "Closed Wait"
        excludes []
        => low::closed_wait,

    SingleWait(1) / "Single Wait"
        excludes []
        => low::single_wait,

    SelfDrawn(1) / "Self-Drawn"
        excludes []
        => low::self_drawn,

    FlowerTiles(1) / "Flower Tiles"
        excludes []
        => low::flower_tiles,
}

// ============================================================================
// Supporting types (defined after FanType exists)
// ============================================================================

/// Bitset for fan-exclusion checks. 81 bits used (u128 is sufficient).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FanExclusionSet(pub u128);

impl FanExclusionSet {
    pub fn set_bit(&mut self, bit: usize) {
        self.0 |= 1u128 << bit;
    }
}

// ============================================================================
// Stub — placeholder check function for unimplemented / not-yet-activated rules
// ============================================================================

/// No-op stub for rules not yet implemented. Always returns empty.
pub(crate) fn empty_rule(
    _decomp: &DecomposeResult,
    _static_ctx: &StaticFanContext,
    _dynamic_ctx: &DynamicFanContext,
    _wait_type: WaitType,
) -> Vec<FanInstance> {
    vec![]
}

// ============================================================================
// Rule submodules (commented out — activate one at a time)
//
// To activate: 1) uncomment the `mod`, 2) update the check paths in mcr_rules! above
// ============================================================================

mod helpers;
mod high;
mod low;
mod mid_high;
mod mid_low;
mod test_helpers;
