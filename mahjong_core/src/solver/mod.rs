mod decompose_special;
mod decompose_standard;
mod decomposition;
pub(crate) mod fan;

pub(crate) use decomposition::Decomposition;
// No re-exports from fan/ until fan extractors exist.

use crate::structs::{BitTileCounts, Pair};

/// Check if the concealed tiles can form a winning hand (any pattern).
///
/// Returns `true` as soon as any valid decomposition is found.
/// Does **not** check minimum fan score — that's the caller's responsibility.
pub(crate) fn is_hu(counts: &BitTileCounts) -> bool {
    // Standard decomposition (4 sets + 1 pair, or fewer with declared melds)
    if !decompose_standard::decompose_standard(counts).is_empty() {
        return true;
    }
    // Special hand patterns
    if decompose_special::detect_seven_pairs(counts).is_some()
        || decompose_special::detect_thirteen_orphans(counts).is_some()
    {
        return true;
    }
    false
}

#[allow(dead_code)]
/// Enumerate all valid decompositions of the concealed tiles.
pub(crate) fn decompose(counts: &BitTileCounts) -> Vec<Decomposition> {
    let mut result: Vec<Decomposition> = Vec::new();

    // Standard decompositions
    for sd in decompose_standard::decompose_standard(counts) {
        result.push(Decomposition::Standard {
            pair: Pair::new(sd.pair_tile),
            sets: sd.melds,
        });
    }

    // Special hand patterns
    if let Some(d) = decompose_special::detect_seven_pairs(counts) {
        result.push(d);
    }
    if let Some(d) = decompose_special::detect_thirteen_orphans(counts) {
        result.push(d);
    }

    result
}
