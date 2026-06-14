mod hand;
mod sets;
mod tiles;

pub(crate) use hand::BitTileCounts;
pub use hand::Hand;
use rand::seq::SliceRandom;
pub use sets::{Meld, Pair, Quad, Sequence, Triplet};
pub use tiles::{Tile, TileType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

impl Wind {
    pub fn next(self) -> Self {
        match self {
            Wind::East => Wind::South,
            Wind::South => Wind::West,
            Wind::West => Wind::North,
            Wind::North => Wind::East,
        }
    }

    pub fn iter() -> impl Iterator<Item = Wind> {
        [Wind::East, Wind::South, Wind::West, Wind::North].into_iter()
    }
}

pub(crate) const WALL_SIZE: usize =
    Tile::NON_FLOWER_TILE_COUNT * 4 + Tile::FLOWER_TILE_COUNT;

#[derive(Clone, Copy)]
pub struct Wall {
    tiles: [Tile; WALL_SIZE],
    pointer: usize,
}

impl Wall {
    pub fn new_mcr() -> Self {
        let mut tiles: [Tile; WALL_SIZE] = [Tile::Character1; WALL_SIZE];
        let mut idx = 0;
        for tile in Tile::iter() {
            let count = if tile.is_flower() { 1 } else { 4 };
            for _ in 0..count {
                tiles[idx] = tile;
                idx += 1;
            }
        }
        // No shuffling for now; deterministic wall
        Self { tiles, pointer: 0 }
    }

    pub(crate) fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.tiles.shuffle(&mut rng);
    }

    /// Yields the next tile from the wall, if available.
    /// Advances the wall pointer.
    pub(crate) fn yield_tile(&mut self) -> Option<Tile> {
        if self.pointer >= WALL_SIZE {
            return None;
        }
        let tile = self.tiles[self.pointer];
        self.pointer += 1;
        Some(tile)
    }
}
