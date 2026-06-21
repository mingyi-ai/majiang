use crate::array_vec::ArrayVec;
use crate::structs::{Meld, Pair, Tile};

/// A complete decomposition of a winning hand.
///
/// Fan rule extractors pattern-match on this type.
/// The solver produces one or more `Decomposition` values for a given hand;
/// each represents a valid way to interpret the tiles as a winning pattern.
pub enum Decomposition {
    /// 4 sets + 1 pair (or fewer sets with declared melds).
    Standard {
        pair: Pair,
        sets: ArrayVec<Meld, 4>,
    },
    SevenPairs {
        pairs: [Pair; 7],
    },
    ThirteenOrphans {
        pair: Pair,
        tiles: [Tile; 13],
    },
    GreaterHonorsAndKnittedTiles {
        honors: [Tile; 7],
        knitted: [Tile; 7],
    },
    MediumHonorsAndKnittedTiles {
        honors: [Tile; 6],
        knitted: [Tile; 8],
    },
    LesserHonorsAndKnittedTiles {
        honors: [Tile; 5],
        knitted: [Tile; 9],
    },
    KnittedStraight {
        pair: Pair,
        tiles: [Tile; 9],
        set: Meld,
    },
}
