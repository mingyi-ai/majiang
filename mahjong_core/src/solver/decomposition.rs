use crate::array_vec::ArrayVec;
use crate::structs::{Meld, Pair};

/// A complete decomposition of a winning hand.
///
/// Fan rule extractors pattern-match on this type through the
/// `HandProfile` view, not directly.  The variants here hold
/// the canonical tile data; the profile flattens it for rules.
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
    },
}
