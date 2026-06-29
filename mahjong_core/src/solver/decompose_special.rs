use crate::{
    array_vec::ArrayVec,
    solver::Decomposition,
    structs::{BitTileCounts, Pair, Tile},
};

/// Detect a Seven Pairs hand.
///
/// Seven pairs = 14 tiles. Each tile may appear 2 times (one pair) or
/// 4 times (two pairs of the same tile, per MCR rules).
pub(crate) fn detect_seven_pairs(
    counts: &BitTileCounts,
) -> Option<Decomposition> {
    // Collect pairs. A tile with count 2 gives 1 pair; count 4 gives 2 pairs.
    let mut pairs: ArrayVec<Pair, 7> = ArrayVec::new();
    for tile in Tile::iter() {
        if tile.is_flower() {
            continue;
        }
        let c = counts.count(tile);
        match c {
            0 => {}
            2 => {
                if pairs.len() >= 7 {
                    return None;
                }
                pairs.push(Pair::new(tile));
            }
            4 => {
                if pairs.len() >= 6 {
                    return None;
                }
                pairs.push(Pair::new(tile));
                pairs.push(Pair::new(tile));
            }
            _ => return None,
        }
    }
    if pairs.len() != 7 {
        return None;
    }
    Some(Decomposition::SevenPairs {
        pairs: [
            pairs[0], pairs[1], pairs[2], pairs[3], pairs[4], pairs[5],
            pairs[6],
        ],
    })
}

/// Detect a Thirteen Orphans hand.
///
/// One copy of each terminal (1,9) in all three suits plus one copy of
/// each honor (7 total), plus a duplicate of any one of those 13 tiles
/// to form the pair. Total: 14 tiles.
pub(crate) fn detect_thirteen_orphans(
    counts: &BitTileCounts,
) -> Option<Decomposition> {
    // The 13 orphan tiles in order
    const ORPHANS: [Tile; 13] = [
        Tile::Character1,
        Tile::Character9,
        Tile::Dot1,
        Tile::Dot9,
        Tile::Bamboo1,
        Tile::Bamboo9,
        Tile::East,
        Tile::South,
        Tile::West,
        Tile::North,
        Tile::Red,
        Tile::Green,
        Tile::White,
    ];
    let mut pair_tile: Option<Tile> = None;
    for &t in &ORPHANS {
        let c = counts.count(t);
        match c {
            1 => {}
            2 => {
                if pair_tile.is_some() {
                    return None; // only one tile can have count 2
                }
                pair_tile = Some(t);
            }
            _ => return None, // missing or too many
        }
    }
    let pair_tile = pair_tile?; // must have exactly one pair
    // Confirm total count = 14 (13 orphans + 1 duplicate for pair)
    if counts.total_count() != 14 {
        return None;
    }
    Some(Decomposition::ThirteenOrphans {
        pair: Pair::new(pair_tile),
    })
}
