// mod helpers;
// mod high;
// mod low;
// mod mid_high;
// mod mid_low;

/// All 81 MCR fan types.
///
/// Discriminants serve as stable bit indices for `FanExclusionSet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum FanType {
    // ── 88 points ──
    BigFourWinds = 1,
    BigThreeDragons = 2,
    AllGreen = 3,
    NineGates = 4,
    FourKongs = 5,
    SevenShiftedPairs = 6,
    ThirteenOrphans = 7,
    // ── 64 points ──
    AllTerminals = 8,
    LittleFourWinds = 9,
    LittleThreeDragons = 10,
    AllHonors = 11,
    FourConcealedPungs = 12,
    PureTerminalChows = 13,
    // ── 48 points ──
    QuadrupleChow = 14,
    FourPureShiftedPungs = 15,
    // ── 32 points ──
    FourShiftedChows = 16,
    ThreeKongs = 17,
    AllTerminalsAndHonors = 18,
    // ── 24 points ──
    SevenPairs = 19,
    GreaterHonorsAndKnittedTiles = 20,
    AllEvenPungs = 21,
    FullFlush = 22,
    PureTripleChow = 23,
    PureShiftedPungs = 24,
    UpperTiles = 25,
    MiddleTiles = 26,
    LowerTiles = 27,
    // ── 16 points ──
    PureStraight = 28,
    ThreeSuitedTerminalChows = 29,
    PureShiftedChows = 30,
    AllFives = 31,
    TriplePung = 32,
    ThreeConcealedPungs = 33,
    // ── 12 points ──
    LesserHonorsAndKnittedTiles = 34,
    KnittedStraight = 35,
    UpperFour = 36,
    LowerFour = 37,
    BigThreeWinds = 38,
    // ── 8 points ──
    MixedStraight = 39,
    ReversibleTiles = 40,
    MixedTripleChow = 41,
    MixedShiftedPungs = 42,
    ChickenHand = 43,
    LastTileDraw = 44,
    LastTileClaim = 45,
    OutWithReplacementTile = 46,
    RobbingTheKong = 47,
    TwoConcealedKongs = 48,
    // ── 6 points ──
    AllPungs = 49,
    HalfFlush = 50,
    MixedShiftedChows = 51,
    AllTypes = 52,
    MeldedHand = 53,
    TwoDragonPungs = 54,
    // ── 4 points ──
    OutsideHand = 55,
    FullyConcealed = 56,
    TwoMeldedKongs = 57,
    LastTile = 58,
    // ── 2 points ──
    DragonPung = 59,
    PrevalentWind = 60,
    SeatWind = 61,
    ConcealedHand = 62,
    AllChows = 63,
    TileHog = 64,
    DoublePung = 65,
    TwoConcealedPungs = 66,
    ConcealedKong = 67,
    AllSimples = 68,
    // ── 1 point ──
    PureDoubleChow = 69,
    MixedDoubleChow = 70,
    ShortStraight = 71,
    TwoTerminalChows = 72,
    PungOfTerminalsOrHonors = 73,
    MeldedKong = 74,
    OneVoidedSuit = 75,
    NoHonors = 76,
    EdgeWait = 77,
    ClosedWait = 78,
    SingleWait = 79,
    SelfDrawn = 80,
    FlowerTiles = 81,
}

impl FanType {
    pub fn points(self) -> u8 {
        match self {
            Self::BigFourWinds
            | Self::BigThreeDragons
            | Self::AllGreen
            | Self::NineGates
            | Self::FourKongs
            | Self::SevenShiftedPairs
            | Self::ThirteenOrphans => 88,
            Self::AllTerminals
            | Self::LittleFourWinds
            | Self::LittleThreeDragons
            | Self::AllHonors
            | Self::FourConcealedPungs
            | Self::PureTerminalChows => 64,
            Self::QuadrupleChow | Self::FourPureShiftedPungs => 48,
            Self::FourShiftedChows
            | Self::ThreeKongs
            | Self::AllTerminalsAndHonors => 32,
            Self::SevenPairs
            | Self::GreaterHonorsAndKnittedTiles
            | Self::AllEvenPungs
            | Self::FullFlush
            | Self::PureTripleChow
            | Self::PureShiftedPungs
            | Self::UpperTiles
            | Self::MiddleTiles
            | Self::LowerTiles => 24,
            Self::PureStraight
            | Self::ThreeSuitedTerminalChows
            | Self::PureShiftedChows
            | Self::AllFives
            | Self::TriplePung
            | Self::ThreeConcealedPungs => 16,
            Self::LesserHonorsAndKnittedTiles
            | Self::KnittedStraight
            | Self::UpperFour
            | Self::LowerFour
            | Self::BigThreeWinds => 12,
            Self::MixedStraight
            | Self::ReversibleTiles
            | Self::MixedTripleChow
            | Self::MixedShiftedPungs
            | Self::ChickenHand
            | Self::LastTileDraw
            | Self::LastTileClaim
            | Self::OutWithReplacementTile
            | Self::RobbingTheKong
            | Self::TwoConcealedKongs => 8,
            Self::AllPungs
            | Self::HalfFlush
            | Self::MixedShiftedChows
            | Self::AllTypes
            | Self::MeldedHand
            | Self::TwoDragonPungs => 6,
            Self::OutsideHand
            | Self::FullyConcealed
            | Self::TwoMeldedKongs
            | Self::LastTile => 4,
            Self::DragonPung
            | Self::PrevalentWind
            | Self::SeatWind
            | Self::ConcealedHand
            | Self::AllChows
            | Self::TileHog
            | Self::DoublePung
            | Self::TwoConcealedPungs
            | Self::ConcealedKong
            | Self::AllSimples => 2,
            Self::PureDoubleChow
            | Self::MixedDoubleChow
            | Self::ShortStraight
            | Self::TwoTerminalChows
            | Self::PungOfTerminalsOrHonors
            | Self::MeldedKong
            | Self::OneVoidedSuit
            | Self::NoHonors
            | Self::EdgeWait
            | Self::ClosedWait
            | Self::SingleWait
            | Self::SelfDrawn
            | Self::FlowerTiles => 1,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::BigFourWinds => "Big Four Winds",
            Self::BigThreeDragons => "Big Three Dragons",
            Self::AllGreen => "All Green",
            Self::NineGates => "Nine Gates",
            Self::FourKongs => "Four Kongs",
            Self::SevenShiftedPairs => "Seven Shifted Pairs",
            Self::ThirteenOrphans => "Thirteen Orphans",
            Self::AllTerminals => "All Terminals",
            Self::LittleFourWinds => "Little Four Winds",
            Self::LittleThreeDragons => "Little Three Dragons",
            Self::AllHonors => "All Honors",
            Self::FourConcealedPungs => "Four Concealed Pungs",
            Self::PureTerminalChows => "Pure Terminal Chows",
            Self::QuadrupleChow => "Quadruple Chow",
            Self::FourPureShiftedPungs => "Four Pure Shifted Pungs",
            Self::FourShiftedChows => "Four Shifted Chows",
            Self::ThreeKongs => "Three Kongs",
            Self::AllTerminalsAndHonors => "All Terminals and Honors",
            Self::SevenPairs => "Seven Pairs",
            Self::GreaterHonorsAndKnittedTiles => {
                "Greater Honors and Knitted Tiles"
            }
            Self::AllEvenPungs => "All Even Pungs",
            Self::FullFlush => "Full Flush",
            Self::PureTripleChow => "Pure Triple Chow",
            Self::PureShiftedPungs => "Pure Shifted Pungs",
            Self::UpperTiles => "Upper Tiles",
            Self::MiddleTiles => "Middle Tiles",
            Self::LowerTiles => "Lower Tiles",
            Self::PureStraight => "Pure Straight",
            Self::ThreeSuitedTerminalChows => "Three-Suited Terminal Chows",
            Self::PureShiftedChows => "Pure Shifted Chows",
            Self::AllFives => "All Fives",
            Self::TriplePung => "Triple Pung",
            Self::ThreeConcealedPungs => "Three Concealed Pungs",
            Self::LesserHonorsAndKnittedTiles => {
                "Lesser Honors and Knitted Tiles"
            }
            Self::KnittedStraight => "Knitted Straight",
            Self::UpperFour => "Upper Four",
            Self::LowerFour => "Lower Four",
            Self::BigThreeWinds => "Big Three Winds",
            Self::MixedStraight => "Mixed Straight",
            Self::ReversibleTiles => "Reversible Tiles",
            Self::MixedTripleChow => "Mixed Triple Chow",
            Self::MixedShiftedPungs => "Mixed Shifted Pungs",
            Self::ChickenHand => "Chicken Hand",
            Self::LastTileDraw => "Last Tile Draw",
            Self::LastTileClaim => "Last Tile Claim",
            Self::OutWithReplacementTile => "Out With Replacement Tile",
            Self::RobbingTheKong => "Robbing The Kong",
            Self::TwoConcealedKongs => "Two Concealed Kongs",
            Self::AllPungs => "All Pungs",
            Self::HalfFlush => "Half Flush",
            Self::MixedShiftedChows => "Mixed Shifted Chows",
            Self::AllTypes => "All Types",
            Self::MeldedHand => "Melded Hand",
            Self::TwoDragonPungs => "Two Dragon Pungs",
            Self::OutsideHand => "Outside Hand",
            Self::FullyConcealed => "Fully Concealed",
            Self::TwoMeldedKongs => "Two Melded Kongs",
            Self::LastTile => "Last Tile",
            Self::DragonPung => "Dragon Pung",
            Self::PrevalentWind => "Prevalent Wind",
            Self::SeatWind => "Seat Wind",
            Self::ConcealedHand => "Concealed Hand",
            Self::AllChows => "All Chows",
            Self::TileHog => "Tile Hog",
            Self::DoublePung => "Double Pung",
            Self::TwoConcealedPungs => "Two Concealed Pungs",
            Self::ConcealedKong => "Concealed Kong",
            Self::AllSimples => "All Simples",
            Self::PureDoubleChow => "Pure Double Chow",
            Self::MixedDoubleChow => "Mixed Double Chow",
            Self::ShortStraight => "Short Straight",
            Self::TwoTerminalChows => "Two Terminal Chows",
            Self::PungOfTerminalsOrHonors => "Pung of Terminals or Honors",
            Self::MeldedKong => "Melded Kong",
            Self::OneVoidedSuit => "One Voided Suit",
            Self::NoHonors => "No Honors",
            Self::EdgeWait => "Edge Wait",
            Self::ClosedWait => "Closed Wait",
            Self::SingleWait => "Single Wait",
            Self::SelfDrawn => "Self-Drawn",
            Self::FlowerTiles => "Flower Tiles",
        }
    }

    #[inline]
    pub fn bit_index(self) -> usize {
        self as u16 as usize
    }
}

// ── Exclusion bitset ──

/// Bitset for fan-exclusion checks. 81 bits used (u128 is sufficient).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FanExclusionSet(pub u128);

impl FanExclusionSet {
    #[inline]
    pub fn set(&mut self, fan: FanType) {
        self.0 |= 1u128 << fan.bit_index();
    }
}

// ── Search types ──

/// A candidate fan instance extracted from a decomposition.
#[derive(Debug, Clone)]
pub struct FanCandidate {
    pub fan_type: FanType,
    /// Which sets this fan uses (bit 0 = sets[0], etc.).
    pub used_set_mask: u64,
    /// Whether the pair is involved.
    pub uses_pair: bool,
    pub score: u8,
    pub excludes_mask: FanExclusionSet,
}

// /// A registered rule: a check function and its exclusion list.
// struct RuleEntry {
//     check: fn(&HandProfile, &FanContext) -> Vec<FanCandidate>,
//     excludes: &'static [FanType],
// }

// /// All 81 MCR rules in the standard order (descending by point value).
// const ALL_RULES: &[RuleEntry] = &[
//     // ── 88 points ──
//     RuleEntry {
//         check: high::big_four_winds,
//         excludes: high::BIG_FOUR_WINDS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::big_three_dragons,
//         excludes: high::BIG_THREE_DRAGONS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::all_green,
//         excludes: high::ALL_GREEN_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::nine_gates,
//         excludes: high::NINE_GATES_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::four_kongs,
//         excludes: high::FOUR_KONGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::seven_shifted_pairs,
//         excludes: high::SEVEN_SHIFTED_PAIRS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::thirteen_orphans,
//         excludes: high::THIRTEEN_ORPHANS_EXCLUDES,
//     },
//     // ── 64 points ──
//     RuleEntry {
//         check: high::all_terminals,
//         excludes: high::ALL_TERMINALS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::little_four_winds,
//         excludes: high::LITTLE_FOUR_WINDS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::little_three_dragons,
//         excludes: high::LITTLE_THREE_DRAGONS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::all_honors,
//         excludes: high::ALL_HONORS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::four_concealed_pungs,
//         excludes: high::FOUR_CONCEALED_PUNGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::pure_terminal_chows,
//         excludes: high::PURE_TERMINAL_CHOWS_EXCLUDES,
//     },
//     // ── 48 points ──
//     RuleEntry {
//         check: high::quadruple_chow,
//         excludes: high::QUADRUPLE_CHOW_EXCLUDES,
//     },
//     RuleEntry {
//         check: high::four_pure_shifted_pungs,
//         excludes: high::FOUR_PURE_SHIFTED_PUNGS_EXCLUDES,
//     },
//     // ── 32 points ──
//     RuleEntry {
//         check: mid_high::four_shifted_chows,
//         excludes: mid_high::FOUR_SHIFTED_CHOWS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::three_kongs,
//         excludes: mid_high::THREE_KONGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::all_terminals_and_honors,
//         excludes: mid_high::ALL_TERMINALS_AND_HONORS_EXCLUDES,
//     },
//     // ── 24 points ──
//     RuleEntry {
//         check: mid_high::seven_pairs,
//         excludes: mid_high::SEVEN_PAIRS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::all_even_pungs,
//         excludes: mid_high::ALL_EVEN_PUNGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::full_flush,
//         excludes: mid_high::FULL_FLUSH_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::pure_triple_chow,
//         excludes: mid_high::PURE_TRIPLE_CHOW_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::pure_shifted_pungs,
//         excludes: mid_high::PURE_SHIFTED_PUNGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::upper_tiles,
//         excludes: mid_high::UPPER_TILES_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::middle_tiles,
//         excludes: mid_high::MIDDLE_TILES_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::lower_tiles,
//         excludes: mid_high::LOWER_TILES_EXCLUDES,
//     },
//     // ── 16 points ──
//     RuleEntry {
//         check: mid_high::pure_straight,
//         excludes: mid_high::PURE_STRAIGHT_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::three_suited_terminal_chows,
//         excludes: mid_high::THREE_SUITED_TERMINAL_CHOWS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::pure_shifted_chows,
//         excludes: mid_high::PURE_SHIFTED_CHOWS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::all_fives,
//         excludes: mid_high::ALL_FIVES_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::triple_pung,
//         excludes: mid_high::TRIPLE_PUNG_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_high::three_concealed_pungs,
//         excludes: mid_high::THREE_CONCEALED_PUNGS_EXCLUDES,
//     },
//     // ── 12 points ──
//     RuleEntry {
//         check: mid_low::upper_four,
//         excludes: mid_low::UPPER_FOUR_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::lower_four,
//         excludes: mid_low::LOWER_FOUR_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::big_three_winds,
//         excludes: mid_low::BIG_THREE_WINDS_EXCLUDES,
//     },
//     // ── 8 points ──
//     RuleEntry {
//         check: mid_low::mixed_straight,
//         excludes: mid_low::MIXED_STRAIGHT_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::reversible_tiles,
//         excludes: mid_low::REVERSIBLE_TILES_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::mixed_triple_chow,
//         excludes: mid_low::MIXED_TRIPLE_CHOW_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::mixed_shifted_pungs,
//         excludes: mid_low::MIXED_SHIFTED_PUNGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::chicken_hand,
//         excludes: mid_low::CHICKEN_HAND_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::last_tile_draw,
//         excludes: mid_low::LAST_TILE_DRAW_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::last_tile_claim,
//         excludes: mid_low::LAST_TILE_CLAIM_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::out_with_replacement_tile,
//         excludes: mid_low::OUT_WITH_REPLACEMENT_TILE_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::robbing_the_kong,
//         excludes: mid_low::ROBBING_THE_KONG_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::two_concealed_kongs,
//         excludes: mid_low::TWO_CONCEALED_KONGS_EXCLUDES,
//     },
//     // ── 6 points ──
//     RuleEntry {
//         check: mid_low::all_pungs,
//         excludes: mid_low::ALL_PUNGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::half_flush,
//         excludes: mid_low::HALF_FLUSH_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::mixed_shifted_chows,
//         excludes: mid_low::MIXED_SHIFTED_CHOWS_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::all_types,
//         excludes: mid_low::ALL_TYPES_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::melded_hand,
//         excludes: mid_low::MELDED_HAND_EXCLUDES,
//     },
//     RuleEntry {
//         check: mid_low::two_dragon_pungs,
//         excludes: mid_low::TWO_DRAGON_PUNGS_EXCLUDES,
//     },
//     // ── 4 points ──
//     RuleEntry {
//         check: low::outside_hand,
//         excludes: low::OUTSIDE_HAND_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::fully_concealed,
//         excludes: low::FULLY_CONCEALED_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::two_melded_kongs,
//         excludes: low::TWO_MELDED_KONGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::last_tile,
//         excludes: low::LAST_TILE_EXCLUDES,
//     },
//     // ── 2 points ──
//     RuleEntry {
//         check: low::dragon_pung,
//         excludes: low::DRAGON_PUNG_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::prevalent_wind,
//         excludes: low::PREVALENT_WIND_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::seat_wind,
//         excludes: low::SEAT_WIND_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::concealed_hand,
//         excludes: low::CONCEALED_HAND_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::all_chows,
//         excludes: low::ALL_CHOWS_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::tile_hog,
//         excludes: low::TILE_HOG_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::double_pung,
//         excludes: low::DOUBLE_PUNG_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::two_concealed_pungs,
//         excludes: low::TWO_CONCEALED_PUNGS_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::concealed_kong,
//         excludes: low::CONCEALED_KONG_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::all_simples,
//         excludes: low::ALL_SIMPLES_EXCLUDES,
//     },
//     // ── 1 point ──
//     RuleEntry {
//         check: low::pure_double_chow,
//         excludes: low::PURE_DOUBLE_CHOW_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::mixed_double_chow,
//         excludes: low::MIXED_DOUBLE_CHOW_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::short_straight,
//         excludes: low::SHORT_STRAIGHT_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::two_terminal_chows,
//         excludes: low::TWO_TERMINAL_CHOWS_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::pung_of_terminals_or_honors,
//         excludes: low::PUNG_OF_TERMINALS_OR_HONORS_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::melded_kong,
//         excludes: low::MELDED_KONG_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::one_voided_suit,
//         excludes: low::ONE_VOIDED_SUIT_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::no_honors,
//         excludes: low::NO_HONORS_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::edge_wait,
//         excludes: low::EDGE_WAIT_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::closed_wait,
//         excludes: low::CLOSED_WAIT_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::single_wait,
//         excludes: low::SINGLE_WAIT_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::self_drawn,
//         excludes: low::SELF_DRAWN_EXCLUDES,
//     },
//     RuleEntry {
//         check: low::flower_tiles,
//         excludes: low::FLOWER_TILES_EXCLUDES,
//     },
// ];

// /// Check all rules against a decomposition profile + context.
// ///
// /// Returns candidates paired with their exclusion lists.
// pub(crate) fn check_all(
//     profile: &HandProfile,
//     ctx: &FanContext,
// ) -> Vec<(FanCandidate, &'static [FanType])> {
//     let mut result = Vec::new();
//     for entry in ALL_RULES {
//         let candidates = (entry.check)(profile, ctx);
//         if !candidates.is_empty() {
//             result.extend(candidates.into_iter().map(|c| (c, entry.excludes)));
//         }
//     }
//     result
// }
