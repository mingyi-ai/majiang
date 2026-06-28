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
