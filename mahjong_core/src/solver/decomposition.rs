use crate::structs::{Meld, Pair, Tile};

/// A complete decomposition of a winning hand into 4 sets and 1 pair.
///
/// Combines declared melds (from `Hand.melds`) with the decomposition
/// of the concealed tiles. Fan rule extractors operate on this type.
pub enum Decomposition {
    Standard {
        pair: Pair,
        sets: [Meld; 4],
    },
    SevenPairs {
        pairs: [Pair; 7],
    },
    ThirteenOrphans {
        pair: Pair,
        tiles: [Tile; 12],
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
