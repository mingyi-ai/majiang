#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TileType {
    Character,
    Dot,
    Bamboo,
    Wind,
    Dragon,
    Flower,
}

#[derive(Debug)]
pub enum TileError {
    ConversionError(&'static str),
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
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

    pub const MCR_SET_COUNT: usize = 144;

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

    #[inline]
    pub fn iter_all() -> impl Iterator<Item = Tile> {
        Self::ALL.iter().copied()
    }

    #[inline]
    pub fn id(self) -> u8 {
        self as u8
    }

    #[inline]
    pub fn from_id(value: u8) -> Result<Self, TileError> {
        let row = (value >> 6) as usize;
        let shift = (value & 0x3F) as usize;

        let tile = Self::from_indices(row, shift)?;
        Ok(tile)
    }

    #[inline]
    pub fn from_indices(row: usize, shift: usize) -> Result<Self, TileError> {
        if row > 3 {
            return Err(TileError::ConversionError("Row index out of range"));
        }
        if !shift.is_multiple_of(4) {
            return Err(TileError::ConversionError("Shift not multiple of 4"));
        }
        if (row < 3 && shift >= 36) || (row == 3 && shift >= 60) {
            return Err(TileError::ConversionError(
                "Shift out of range for row",
            ));
        }
        let id = (row << 6) as u8 | shift as u8;
        Ok(unsafe { std::mem::transmute::<u8, Self>(id) })
    }

    #[inline]
    pub fn from_dense_index(value: usize) -> Option<Self> {
        Self::ALL.get(value).copied()
    }

    #[inline]
    pub fn get_type(self) -> TileType {
        let id = self.id();
        match id {
            0..=32 => TileType::Character,
            64..=96 => TileType::Dot,
            128..=160 => TileType::Bamboo,
            192..=204 => TileType::Wind,
            208..=216 => TileType::Dragon,
            220..=227 => TileType::Flower,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn is_flower(self) -> bool {
        (self as u8) >= 220
    }

    /// Returns the dense index (0..41) for array indexing.
    pub fn dense_index(self) -> usize {
        match self {
            Tile::Character1 => 0,
            Tile::Character2 => 1,
            Tile::Character3 => 2,
            Tile::Character4 => 3,
            Tile::Character5 => 4,
            Tile::Character6 => 5,
            Tile::Character7 => 6,
            Tile::Character8 => 7,
            Tile::Character9 => 8,
            Tile::Dot1 => 9,
            Tile::Dot2 => 10,
            Tile::Dot3 => 11,
            Tile::Dot4 => 12,
            Tile::Dot5 => 13,
            Tile::Dot6 => 14,
            Tile::Dot7 => 15,
            Tile::Dot8 => 16,
            Tile::Dot9 => 17,
            Tile::Bamboo1 => 18,
            Tile::Bamboo2 => 19,
            Tile::Bamboo3 => 20,
            Tile::Bamboo4 => 21,
            Tile::Bamboo5 => 22,
            Tile::Bamboo6 => 23,
            Tile::Bamboo7 => 24,
            Tile::Bamboo8 => 25,
            Tile::Bamboo9 => 26,
            Tile::East => 27,
            Tile::South => 28,
            Tile::West => 29,
            Tile::North => 30,
            Tile::Red => 31,
            Tile::Green => 32,
            Tile::White => 33,
            Tile::Plum => 34,
            Tile::Orchid => 35,
            Tile::BambooF => 36,
            Tile::Chrysanthemum => 37,
            Tile::Spring => 38,
            Tile::Summer => 39,
            Tile::Autumn => 40,
            Tile::Winter => 41,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            // Dots
            Tile::Dot1 => "1 Dot",
            Tile::Dot2 => "2 Dot",
            Tile::Dot3 => "3 Dot",
            Tile::Dot4 => "4 Dot",
            Tile::Dot5 => "5 Dot",
            Tile::Dot6 => "6 Dot",
            Tile::Dot7 => "7 Dot",
            Tile::Dot8 => "8 Dot",
            Tile::Dot9 => "9 Dot",

            // Bamboo
            Tile::Bamboo1 => "1 Bamboo",
            Tile::Bamboo2 => "2 Bamboo",
            Tile::Bamboo3 => "3 Bamboo",
            Tile::Bamboo4 => "4 Bamboo",
            Tile::Bamboo5 => "5 Bamboo",
            Tile::Bamboo6 => "6 Bamboo",
            Tile::Bamboo7 => "7 Bamboo",
            Tile::Bamboo8 => "8 Bamboo",
            Tile::Bamboo9 => "9 Bamboo",

            // Characters
            Tile::Character1 => "1 Character",
            Tile::Character2 => "2 Character",
            Tile::Character3 => "3 Character",
            Tile::Character4 => "4 Character",
            Tile::Character5 => "5 Character",
            Tile::Character6 => "6 Character",
            Tile::Character7 => "7 Character",
            Tile::Character8 => "8 Character",
            Tile::Character9 => "9 Character",

            // Winds
            Tile::East => "East",
            Tile::South => "South",
            Tile::West => "West",
            Tile::North => "North",

            // Dragons
            Tile::Red => "Red Dragon",
            Tile::Green => "Green Dragon",
            Tile::White => "White Dragon",

            // Flowers
            Tile::Plum => "Plum",
            Tile::Orchid => "Orchid",
            Tile::Chrysanthemum => "Chrysanthemum",
            Tile::BambooF => "Bamboo",
            Tile::Spring => "Spring",
            Tile::Summer => "Summer",
            Tile::Autumn => "Autumn",
            Tile::Winter => "Winter",
        }
    }
}
