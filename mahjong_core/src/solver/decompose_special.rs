use crate::structs::BitTileCounts;

use super::decomposition::Decomposition;

/// Detect a Seven Pairs hand.
///
/// Seven distinct pairs = 14 tiles where each tile appears exactly twice.
/// The pair tiles must be distinct (no four-of-a-kind split into two pairs
/// under standard MCR rules).
pub(crate) fn detect_seven_pairs(
    _counts: &BitTileCounts,
) -> Option<Decomposition> {
    // TODO: implement
    None
}

/// Detect a Thirteen Orphans hand.
///
/// One copy of each terminal (1,9) in all three suits plus one copy of
/// each honor (7 total), plus a duplicate of any one of those 13 tiles
/// to form the pair. Total: 14 tiles.
pub(crate) fn detect_thirteen_orphans(
    _counts: &BitTileCounts,
) -> Option<Decomposition> {
    // TODO: implement
    None
}

// ── Knitted / honor-knitted patterns ──
// TODO: implement when MCR rules for these are settled
