use super::*;

#[derive(Clone, Copy)]
pub struct Sequence {
    pub(crate) start: Tile,
    pub is_concealed: bool,
}

impl Sequence {
    pub fn new(start: Tile, is_concealed: bool) -> Self {
        if !start.get_type().is_suit() {
            panic!("Sequence must start with a suit tile");
        }
        Self {
            start,
            is_concealed,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Triplet {
    pub(crate) tile: Tile,
    pub is_concealed: bool,
}

impl Triplet {
    pub fn new(tile: Tile, is_concealed: bool) -> Self {
        if matches!(tile.get_type(), TileType::Flower) {
            panic!("Triplet cannot be a flower tile");
        }
        Self { tile, is_concealed }
    }
}

#[derive(Clone, Copy)]
pub struct Quad {
    pub(crate) tile: Tile,
    pub is_concealed: bool,
}

impl Quad {
    pub fn new(tile: Tile, is_concealed: bool) -> Self {
        if matches!(tile.get_type(), TileType::Flower) {
            panic!("Quad cannot be a flower tile");
        }
        Self { tile, is_concealed }
    }
}

#[derive(Clone, Copy)]
pub enum Meld {
    Chow(Sequence),
    Pung(Triplet),
    Kong(Quad),
}

#[derive(Clone, Copy)]
pub struct Pair {
    pub(crate) tile: Tile,
}

impl Pair {
    pub fn new(tile: Tile) -> Self {
        if matches!(tile.get_type(), TileType::Flower) {
            panic!("Pair cannot be a flower tile");
        }
        Self { tile }
    }
}
