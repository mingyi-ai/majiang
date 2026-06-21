use crate::structs::{Meld, Pair, Tile};

/// A complete decomposition of a winning hand.
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

/// Internal: decomposition of concealed tiles only (no declared melds).
///
/// Uses a fixed-size array instead of `Vec` — stack-allocated, no heap.
#[derive(Clone, Copy)]
pub(crate) struct ConcealedDecomp {
    pub(crate) pair_tile: Tile,
    pub(crate) sets: [Option<Meld>; 4],
    pub(crate) len: u8,
}
