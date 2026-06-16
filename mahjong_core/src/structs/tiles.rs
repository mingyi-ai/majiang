#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    Character,
    Dot,
    Bamboo,
    Wind,
    Dragon,
    Flower,
}

impl TileType {
    pub(crate) fn is_suit(self) -> bool {
        matches!(self, Self::Character | Self::Dot | Self::Bamboo)
    }

    pub(crate) fn is_honor(self) -> bool {
        matches!(self, Self::Wind | Self::Dragon)
    }

    pub(crate) fn is_flower(self) -> bool {
        self == Self::Flower
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Mahjong tiles represented with sparse patterns
/// optimized for bitwise operations.
pub enum Tile {
    // Characters (Wan) - Row 0
    Character1 = 0,
    Character2 = 4,
    Character3 = 8,
    Character4 = 12,
    Character5 = 16,
    Character6 = 20,
    Character7 = 24,
    Character8 = 28,
    Character9 = 32,

    // Dots (Tong) - Row 1 (Offset 64)
    Dot1 = 64,
    Dot2 = 68,
    Dot3 = 72,
    Dot4 = 76,
    Dot5 = 80,
    Dot6 = 84,
    Dot7 = 88,
    Dot8 = 92,
    Dot9 = 96,

    // Bamboo (Tiao) - Row 2 (Offset 128)
    Bamboo1 = 128,
    Bamboo2 = 132,
    Bamboo3 = 136,
    Bamboo4 = 140,
    Bamboo5 = 144,
    Bamboo6 = 148,
    Bamboo7 = 152,
    Bamboo8 = 156,
    Bamboo9 = 160,

    // Winds - Row 3 (Offset 192)
    East = 192,
    South = 196,
    West = 200,
    North = 204,

    // Dragons - Row 3
    Red = 208,
    Green = 212,
    White = 216,

    // Flowers - Row 3 (Offset 220, 4 bits each)
    Plum = 220,
    Orchid = 224,
    BambooF = 228,
    Chrysanthemum = 232,
    Spring = 236,
    Summer = 240,
    Autumn = 244,
    Winter = 248,
}

impl Tile {
    pub const NON_FLOWER_TILE_COUNT: usize = 34;
    pub const FLOWER_TILE_COUNT: usize = 8;

    pub const COUNT: usize =
        Self::NON_FLOWER_TILE_COUNT + Self::FLOWER_TILE_COUNT;

    pub const ALL: [Tile; Tile::COUNT] = [
        // Characters
        Tile::Character1,
        Tile::Character2,
        Tile::Character3,
        Tile::Character4,
        Tile::Character5,
        Tile::Character6,
        Tile::Character7,
        Tile::Character8,
        Tile::Character9,
        // Dots
        Tile::Dot1,
        Tile::Dot2,
        Tile::Dot3,
        Tile::Dot4,
        Tile::Dot5,
        Tile::Dot6,
        Tile::Dot7,
        Tile::Dot8,
        Tile::Dot9,
        // Bamboo
        Tile::Bamboo1,
        Tile::Bamboo2,
        Tile::Bamboo3,
        Tile::Bamboo4,
        Tile::Bamboo5,
        Tile::Bamboo6,
        Tile::Bamboo7,
        Tile::Bamboo8,
        Tile::Bamboo9,
        // Winds
        Tile::East,
        Tile::South,
        Tile::West,
        Tile::North,
        // Dragons
        Tile::Red,
        Tile::Green,
        Tile::White,
        // Flowers
        Tile::Plum,
        Tile::Orchid,
        Tile::BambooF,
        Tile::Chrysanthemum,
        Tile::Spring,
        Tile::Summer,
        Tile::Autumn,
        Tile::Winter,
    ];

    pub fn iter() -> impl Iterator<Item = Tile> {
        Self::ALL.iter().copied()
    }

    pub(crate) fn from_repr(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }

    pub fn next(self) -> Tile {
        let v = self as u8;
        match v {
            0..=32 => Tile::from_repr(v + 4),
            64..=96 => Tile::from_repr(v + 4),
            128..=160 => Tile::from_repr(v + 4),
            192..=204 => Tile::from_repr(v + 4),
            208..=216 => Tile::from_repr(v + 4),
            220..=248 => Tile::from_repr(v + 4),
            _ => panic!(),
        }
    }

    pub fn get_type(self) -> TileType {
        let offset = self as u8;
        match offset {
            0..=32 => TileType::Character,
            64..=96 => TileType::Dot,
            128..=160 => TileType::Bamboo,
            192..=204 => TileType::Wind,
            208..=216 => TileType::Dragon,
            220..=248 => TileType::Flower,
            _ => unreachable!(),
        }
    }

    pub fn is_suit(self) -> bool {
        self.get_type().is_suit()
    }

    pub fn is_honor(self) -> bool {
        self.get_type().is_honor()
    }

    pub fn is_flower(self) -> bool {
        self.get_type().is_flower()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tile groups by type — source of truth is the enum variant name.
    const CHARACTERS: [Tile; 9] = [
        Tile::Character1, Tile::Character2, Tile::Character3,
        Tile::Character4, Tile::Character5, Tile::Character6,
        Tile::Character7, Tile::Character8, Tile::Character9,
    ];
    const DOTS: [Tile; 9] = [
        Tile::Dot1, Tile::Dot2, Tile::Dot3,
        Tile::Dot4, Tile::Dot5, Tile::Dot6,
        Tile::Dot7, Tile::Dot8, Tile::Dot9,
    ];
    const BAMBOOS: [Tile; 9] = [
        Tile::Bamboo1, Tile::Bamboo2, Tile::Bamboo3,
        Tile::Bamboo4, Tile::Bamboo5, Tile::Bamboo6,
        Tile::Bamboo7, Tile::Bamboo8, Tile::Bamboo9,
    ];
    const WINDS: [Tile; 4] = [
        Tile::East, Tile::South, Tile::West, Tile::North,
    ];
    const DRAGONS: [Tile; 3] = [
        Tile::Red, Tile::Green, Tile::White,
    ];
    const FLOWERS: [Tile; 8] = [
        Tile::Plum, Tile::Orchid, Tile::BambooF,
        Tile::Chrysanthemum, Tile::Spring, Tile::Summer,
        Tile::Autumn, Tile::Winter,
    ];

    // --- TileType method tests ---

    #[test]
    fn tile_type_is_suit() {
        for &t in &CHARACTERS {
            assert!(t.get_type().is_suit(), "{:?}", t);
        }
        for &t in &DOTS {
            assert!(t.get_type().is_suit(), "{:?}", t);
        }
        for &t in &BAMBOOS {
            assert!(t.get_type().is_suit(), "{:?}", t);
        }
        for &t in &WINDS {
            assert!(!t.get_type().is_suit(), "{:?}", t);
        }
        for &t in &DRAGONS {
            assert!(!t.get_type().is_suit(), "{:?}", t);
        }
        for &t in &FLOWERS {
            assert!(!t.get_type().is_suit(), "{:?}", t);
        }
    }

    #[test]
    fn tile_type_is_honor() {
        for &t in &CHARACTERS {
            assert!(!t.get_type().is_honor(), "{:?}", t);
        }
        for &t in &DOTS {
            assert!(!t.get_type().is_honor(), "{:?}", t);
        }
        for &t in &BAMBOOS {
            assert!(!t.get_type().is_honor(), "{:?}", t);
        }
        for &t in &WINDS {
            assert!(t.get_type().is_honor(), "{:?}", t);
        }
        for &t in &DRAGONS {
            assert!(t.get_type().is_honor(), "{:?}", t);
        }
        for &t in &FLOWERS {
            assert!(!t.get_type().is_honor(), "{:?}", t);
        }
    }

    #[test]
    fn tile_type_is_flower() {
        for &t in &CHARACTERS {
            assert!(!t.get_type().is_flower(), "{:?}", t);
        }
        for &t in &DOTS {
            assert!(!t.get_type().is_flower(), "{:?}", t);
        }
        for &t in &BAMBOOS {
            assert!(!t.get_type().is_flower(), "{:?}", t);
        }
        for &t in &WINDS {
            assert!(!t.get_type().is_flower(), "{:?}", t);
        }
        for &t in &DRAGONS {
            assert!(!t.get_type().is_flower(), "{:?}", t);
        }
        for &t in &FLOWERS {
            assert!(t.get_type().is_flower(), "{:?}", t);
        }
    }

    // --- Tile::from_repr — round-trip every variant ---

    #[test]
    fn from_repr_round_trips_all_variants() {
        for tile in Tile::ALL {
            let raw = tile as u8;
            assert_eq!(tile, Tile::from_repr(raw), "from_repr({}) should produce {:?}", raw, tile);
        }
    }

    // --- Tile::get_type — every tile maps to the correct TileType ---

    #[test]
    fn get_type_for_characters() {
        for &t in &CHARACTERS {
            assert_eq!(t.get_type(), TileType::Character, "{:?}", t);
        }
    }

    #[test]
    fn get_type_for_dots() {
        for &t in &DOTS {
            assert_eq!(t.get_type(), TileType::Dot, "{:?}", t);
        }
    }

    #[test]
    fn get_type_for_bamboos() {
        for &t in &BAMBOOS {
            assert_eq!(t.get_type(), TileType::Bamboo, "{:?}", t);
        }
    }

    #[test]
    fn get_type_for_winds() {
        for &t in &WINDS {
            assert_eq!(t.get_type(), TileType::Wind, "{:?}", t);
        }
    }

    #[test]
    fn get_type_for_dragons() {
        for &t in &DRAGONS {
            assert_eq!(t.get_type(), TileType::Dragon, "{:?}", t);
        }
    }

    #[test]
    fn get_type_for_flowers() {
        for &t in &FLOWERS {
            assert_eq!(t.get_type(), TileType::Flower, "{:?}", t);
        }
    }

    // --- Tile::next — advance by one within each group ---

    #[test]
    fn next_character_tiles() {
        // Character1 through Character8 each have a next; Character9 is the last.
        for i in 0..8 {
            assert_eq!(CHARACTERS[i].next(), CHARACTERS[i + 1], "{:?}.next()", CHARACTERS[i]);
        }
    }

    #[test]
    fn next_dot_tiles() {
        for i in 0..8 {
            assert_eq!(DOTS[i].next(), DOTS[i + 1], "{:?}.next()", DOTS[i]);
        }
    }

    #[test]
    fn next_bamboo_tiles() {
        for i in 0..8 {
            assert_eq!(BAMBOOS[i].next(), BAMBOOS[i + 1], "{:?}.next()", BAMBOOS[i]);
        }
    }

    #[test]
    fn next_wind_tiles() {
        // East, South, West advance; North is the last.
        for i in 0..3 {
            assert_eq!(WINDS[i].next(), WINDS[i + 1], "{:?}.next()", WINDS[i]);
        }
    }

    #[test]
    fn next_dragon_tiles() {
        // Red, Green advance; White is the last.
        for i in 0..2 {
            assert_eq!(DRAGONS[i].next(), DRAGONS[i + 1], "{:?}.next()", DRAGONS[i]);
        }
    }

    #[test]
    fn next_flower_tiles() {
        // Plum through Autumn advance; Winter is the last.
        for i in 0..7 {
            assert_eq!(FLOWERS[i].next(), FLOWERS[i + 1], "{:?}.next()", FLOWERS[i]);
        }
    }

    // --- Tile::is_suit / is_honor / is_flower delegation tests ---

    #[test]
    fn tile_is_suit_delegates_to_type() {
        for tile in Tile::ALL {
            assert_eq!(tile.is_suit(), tile.get_type().is_suit(), "{:?}", tile);
        }
    }

    #[test]
    fn tile_is_honor_delegates_to_type() {
        for tile in Tile::ALL {
            assert_eq!(tile.is_honor(), tile.get_type().is_honor(), "{:?}", tile);
        }
    }

    #[test]
    fn tile_is_flower_delegates_to_type() {
        for tile in Tile::ALL {
            assert_eq!(tile.is_flower(), tile.get_type().is_flower(), "{:?}", tile);
        }
    }

    // --- Constant sanity checks ---

    #[test]
    fn constants_are_correct() {
        assert_eq!(Tile::NON_FLOWER_TILE_COUNT, 34);
        assert_eq!(Tile::FLOWER_TILE_COUNT, 8);
        assert_eq!(Tile::COUNT, 42);
        assert_eq!(Tile::ALL.len(), 42);
    }

    #[test]
    fn all_array_matches_tile_groups() {
        // Verify Tile::ALL is exactly the concatenation of the group arrays.
        let mut expected = Vec::new();
        expected.extend_from_slice(&CHARACTERS);
        expected.extend_from_slice(&DOTS);
        expected.extend_from_slice(&BAMBOOS);
        expected.extend_from_slice(&WINDS);
        expected.extend_from_slice(&DRAGONS);
        expected.extend_from_slice(&FLOWERS);
        assert_eq!(Tile::ALL.as_slice(), &expected);
    }

    #[test]
    fn iter_produces_all_tiles() {
        let collected: Vec<Tile> = Tile::iter().collect();
        assert_eq!(collected, Tile::ALL);
    }
}
