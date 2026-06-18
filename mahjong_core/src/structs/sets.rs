use super::*;

#[derive(Clone, Copy)]
pub struct Sequence {
    start: Tile,
    is_concealed: bool,
}

impl Sequence {
    pub fn new(start: Tile, is_concealed: bool) -> Self {
        // A valid chow must start at rank 1-7 within the suit (so that
        // start.next() and start.next().next() stay in the same suit).
        let v = start as u8;
        if !matches!(v, 0..=24 | 64..=88 | 128..=152) {
            panic!(
                "Sequence must have a valid chow start (suit tile rank 1-7)"
            );
        }
        Self {
            start,
            is_concealed,
        }
    }

    pub fn start(&self) -> Tile {
        self.start
    }

    pub fn tiles(&self) -> [Tile; 3] {
        [self.start, self.start.next(), self.start.next().next()]
    }

    pub fn is_concealed(&self) -> bool {
        self.is_concealed
    }
}

#[derive(Clone, Copy)]
pub struct Triplet {
    tile: Tile,
    is_concealed: bool,
}

impl Triplet {
    pub fn new(tile: Tile, is_concealed: bool) -> Self {
        if matches!(tile.get_type(), TileType::Flower) {
            panic!("Triplet cannot be a flower tile");
        }
        Self { tile, is_concealed }
    }

    pub fn tile(&self) -> Tile {
        self.tile
    }

    pub fn tiles(&self) -> [Tile; 3] {
        [self.tile, self.tile, self.tile]
    }

    pub fn is_concealed(&self) -> bool {
        self.is_concealed
    }
}

#[derive(Clone, Copy)]
pub struct Quad {
    tile: Tile,
    is_concealed: bool,
}

impl Quad {
    pub fn new(tile: Tile, is_concealed: bool) -> Self {
        if matches!(tile.get_type(), TileType::Flower) {
            panic!("Quad cannot be a flower tile");
        }
        Self { tile, is_concealed }
    }

    pub fn tile(&self) -> Tile {
        self.tile
    }

    pub fn tiles(&self) -> [Tile; 4] {
        [self.tile, self.tile, self.tile, self.tile]
    }

    pub fn is_concealed(&self) -> bool {
        self.is_concealed
    }
}

#[derive(Clone, Copy)]
pub enum Meld {
    Chow(Sequence),
    Pung(Triplet),
    Kong(Quad),
}

impl Meld {
    pub fn tiles(&self) -> Vec<Tile> {
        match self {
            Meld::Chow(s) => s.tiles().to_vec(),
            Meld::Pung(t) => t.tiles().to_vec(),
            Meld::Kong(q) => q.tiles().to_vec(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Pair {
    tile: Tile,
}

impl Pair {
    pub fn new(tile: Tile) -> Self {
        if matches!(tile.get_type(), TileType::Flower) {
            panic!("Pair cannot be a flower tile");
        }
        Self { tile }
    }

    pub fn tile(&self) -> Tile {
        self.tile
    }

    pub fn tiles(&self) -> [Tile; 2] {
        [self.tile, self.tile]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tile groups — source of truth is the enum variant name.
    const CHOW_STARTS: [Tile; 21] = [
        Tile::Character1,
        Tile::Character2,
        Tile::Character3,
        Tile::Character4,
        Tile::Character5,
        Tile::Character6,
        Tile::Character7,
        Tile::Dot1,
        Tile::Dot2,
        Tile::Dot3,
        Tile::Dot4,
        Tile::Dot5,
        Tile::Dot6,
        Tile::Dot7,
        Tile::Bamboo1,
        Tile::Bamboo2,
        Tile::Bamboo3,
        Tile::Bamboo4,
        Tile::Bamboo5,
        Tile::Bamboo6,
        Tile::Bamboo7,
    ];
    const SUIT_TILES: [Tile; 27] = [
        Tile::Character1,
        Tile::Character2,
        Tile::Character3,
        Tile::Character4,
        Tile::Character5,
        Tile::Character6,
        Tile::Character7,
        Tile::Character8,
        Tile::Character9,
        Tile::Dot1,
        Tile::Dot2,
        Tile::Dot3,
        Tile::Dot4,
        Tile::Dot5,
        Tile::Dot6,
        Tile::Dot7,
        Tile::Dot8,
        Tile::Dot9,
        Tile::Bamboo1,
        Tile::Bamboo2,
        Tile::Bamboo3,
        Tile::Bamboo4,
        Tile::Bamboo5,
        Tile::Bamboo6,
        Tile::Bamboo7,
        Tile::Bamboo8,
        Tile::Bamboo9,
    ];
    const HONOR_TILES: [Tile; 7] = [
        Tile::East,
        Tile::South,
        Tile::West,
        Tile::North,
        Tile::Red,
        Tile::Green,
        Tile::White,
    ];

    // --- Sequence ---

    #[test]
    fn sequence_new_succeeds_for_valid_chow_starts() {
        for &tile in &CHOW_STARTS {
            let seq = Sequence::new(tile, true);
            assert_eq!(seq.start(), tile, "{:?}", tile);
            assert!(seq.is_concealed());

            let seq = Sequence::new(tile, false);
            assert_eq!(seq.start(), tile, "{:?}", tile);
            assert!(!seq.is_concealed());
        }
    }

    #[test]
    fn sequence_tiles_returns_three_consecutive_tiles() {
        let seq = Sequence::new(Tile::Character3, false);
        assert_eq!(
            seq.tiles(),
            [Tile::Character3, Tile::Character4, Tile::Character5]
        );

        let seq = Sequence::new(Tile::Dot5, false);
        assert_eq!(seq.tiles(), [Tile::Dot5, Tile::Dot6, Tile::Dot7]);

        let seq = Sequence::new(Tile::Bamboo7, false);
        assert_eq!(seq.tiles(), [Tile::Bamboo7, Tile::Bamboo8, Tile::Bamboo9]);
    }

    #[test]
    #[should_panic(expected = "Sequence must have a valid chow start")]
    fn sequence_new_panics_for_rank_eight() {
        Sequence::new(Tile::Character8, false);
    }

    #[test]
    #[should_panic(expected = "Sequence must have a valid chow start")]
    fn sequence_new_panics_for_rank_nine() {
        Sequence::new(Tile::Dot9, false);
    }

    #[test]
    #[should_panic(expected = "Sequence must have a valid chow start")]
    fn sequence_new_panics_for_wind() {
        Sequence::new(Tile::East, false);
    }

    #[test]
    #[should_panic(expected = "Sequence must have a valid chow start")]
    fn sequence_new_panics_for_dragon() {
        Sequence::new(Tile::Red, false);
    }

    #[test]
    #[should_panic(expected = "Sequence must have a valid chow start")]
    fn sequence_new_panics_for_flower() {
        Sequence::new(Tile::Plum, false);
    }

    // --- Triplet ---

    #[test]
    fn triplet_new_succeeds_for_suit_tiles() {
        for &tile in &SUIT_TILES {
            let t = Triplet::new(tile, true);
            assert_eq!(t.tile(), tile, "{:?}", tile);
            assert!(t.is_concealed());
        }
    }

    #[test]
    fn triplet_new_succeeds_for_honor_tiles() {
        for &tile in &HONOR_TILES {
            let t = Triplet::new(tile, false);
            assert_eq!(t.tile(), tile, "{:?}", tile);
            assert!(!t.is_concealed());
        }
    }

    #[test]
    fn triplet_tiles_returns_three_copies() {
        let t = Triplet::new(Tile::East, false);
        assert_eq!(t.tiles(), [Tile::East, Tile::East, Tile::East]);
    }

    #[test]
    #[should_panic(expected = "Triplet cannot be a flower tile")]
    fn triplet_new_panics_for_flower() {
        Triplet::new(Tile::Orchid, false);
    }

    // --- Quad ---

    #[test]
    fn quad_new_succeeds_for_suit_tiles() {
        for &tile in &SUIT_TILES {
            let q = Quad::new(tile, true);
            assert_eq!(q.tile(), tile, "{:?}", tile);
            assert!(q.is_concealed());
        }
    }

    #[test]
    fn quad_new_succeeds_for_honor_tiles() {
        for &tile in &HONOR_TILES {
            let q = Quad::new(tile, false);
            assert_eq!(q.tile(), tile, "{:?}", tile);
            assert!(!q.is_concealed());
        }
    }

    #[test]
    fn quad_tiles_returns_four_copies() {
        let q = Quad::new(Tile::White, false);
        assert_eq!(
            q.tiles(),
            [Tile::White, Tile::White, Tile::White, Tile::White]
        );
    }

    #[test]
    #[should_panic(expected = "Quad cannot be a flower tile")]
    fn quad_new_panics_for_flower() {
        Quad::new(Tile::BambooF, false);
    }

    // --- Pair ---

    #[test]
    fn pair_new_succeeds_for_suit_tiles() {
        for &tile in &SUIT_TILES {
            let p = Pair::new(tile);
            assert_eq!(p.tile(), tile, "{:?}", tile);
        }
    }

    #[test]
    fn pair_new_succeeds_for_honor_tiles() {
        for &tile in &HONOR_TILES {
            let p = Pair::new(tile);
            assert_eq!(p.tile(), tile, "{:?}", tile);
        }
    }

    #[test]
    fn pair_tiles_returns_two_copies() {
        let p = Pair::new(Tile::Character5);
        assert_eq!(p.tiles(), [Tile::Character5, Tile::Character5]);
    }

    #[test]
    #[should_panic(expected = "Pair cannot be a flower tile")]
    fn pair_new_panics_for_flower() {
        Pair::new(Tile::Spring);
    }

    // --- Meld enum ---

    #[test]
    fn meld_tiles_for_chow() {
        let m = Meld::Chow(Sequence::new(Tile::Character3, true));
        assert_eq!(
            m.tiles(),
            vec![Tile::Character3, Tile::Character4, Tile::Character5]
        );
    }

    #[test]
    fn meld_tiles_for_pung() {
        let m = Meld::Pung(Triplet::new(Tile::East, false));
        assert_eq!(m.tiles(), vec![Tile::East, Tile::East, Tile::East]);
    }

    #[test]
    fn meld_tiles_for_kong() {
        let m = Meld::Kong(Quad::new(Tile::Red, true));
        assert_eq!(
            m.tiles(),
            vec![Tile::Red, Tile::Red, Tile::Red, Tile::Red]
        );
    }
}
