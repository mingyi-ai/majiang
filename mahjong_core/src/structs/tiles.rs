#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    pub const COUNT: usize = 42;

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

    pub fn get_type(self) -> TileType {
        let offset = self as u8;
        match offset {
            0..=32 => TileType::Character,
            64..=96 => TileType::Dot,
            128..=160 => TileType::Bamboo,
            192..=204 => TileType::Wind,
            208..=216 => TileType::Dragon,
            220..=227 => TileType::Flower,
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

    /// Convert a raw u8 representation back to a `Tile`.
    /// Returns `None` if the value does not correspond to a valid tile.
    pub fn from_repr(value: u8) -> Option<Tile> {
        // ALL tiles are at multiples of 4; max value is 248 ÷ 4 = 62
        let idx = (value / 4) as usize;
        if idx >= 64 {
            return None;
        }
        static INIT: std::sync::OnceLock<[Option<Tile>; 64]> = std::sync::OnceLock::new();
        let lookup = INIT.get_or_init(|| {
            let mut arr = [None; 64];
            for tile in Tile::ALL {
                let idx = (tile as u8 / 4) as usize;
                arr[idx] = Some(tile);
            }
            arr
        });
        lookup[idx]
    }
}
