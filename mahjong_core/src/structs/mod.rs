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

    pub(crate) fn to_tile(self) -> Tile {
        match self {
            Wind::East => Tile::East,
            Wind::South => Tile::South,
            Wind::West => Tile::West,
            Wind::North => Tile::North,
        }
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
        Self { tiles, pointer: 0 }
    }

    pub(crate) fn shuffle<R: rand::Rng>(&mut self, rng: &mut R) {
        self.tiles.shuffle(rng);
    }

    /// Replace a tile at a specific position. Used by tests to
    /// inject flowers at known positions in an otherwise deterministic
    /// wall.
    #[cfg(test)]
    pub(crate) fn set_tile(&mut self, index: usize, tile: Tile) {
        self.tiles[index] = tile;
    }

    /// Yields the next tile from the wall, if available.
    /// Advances the wall pointer.
    pub(crate) fn yield_tile(&mut self) -> Option<Tile> {
        if self.is_empty() {
            return None;
        }
        let tile = self.tiles[self.pointer];
        self.pointer += 1;
        Some(tile)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pointer >= WALL_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Wall::new_mcr ──

    #[test]
    fn new_wall_is_not_empty() {
        let wall = Wall::new_mcr();
        assert!(!wall.is_empty());
    }

    // ── yield_tile / is_empty ──

    #[test]
    fn yield_tile_returns_tiles_in_initial_order() {
        let mut wall = Wall::new_mcr();

        // The wall is built in Tile::iter() order:
        // Character1 ×4, Character2 ×4, ..., Winter ×1
        for _ in 0..4 {
            assert_eq!(wall.yield_tile(), Some(Tile::Character1));
        }
        for _ in 0..4 {
            assert_eq!(wall.yield_tile(), Some(Tile::Character2));
        }
        // Spot-check a middle tile
        for _ in 0..4 {
            assert_eq!(wall.yield_tile(), Some(Tile::Character3));
        }
        // After 12 tiles, wall should not be empty
        assert!(!wall.is_empty());
    }

    #[test]
    fn yield_tile_exhaustion_returns_none() {
        let mut wall = Wall::new_mcr();

        // Yield all tiles
        let mut count = 0usize;
        while wall.yield_tile().is_some() {
            count += 1;
        }

        assert_eq!(count, WALL_SIZE);
        assert!(wall.is_empty());
        // Subsequent calls return None
        assert_eq!(wall.yield_tile(), None);
        assert_eq!(wall.yield_tile(), None);
    }

    #[test]
    fn yield_tile_after_partial_drain() {
        let mut wall = Wall::new_mcr();

        // Drain half the wall
        for _ in 0..(WALL_SIZE / 2) {
            assert!(wall.yield_tile().is_some());
            assert!(!wall.is_empty());
        }

        // Drain the rest
        while wall.yield_tile().is_some() {}

        assert!(wall.is_empty());
    }
}
