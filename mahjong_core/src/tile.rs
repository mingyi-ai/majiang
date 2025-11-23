use rand::seq::SliceRandom;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TileType {
    Dot,
    Bamboo,
    Character,
    Wind,
    Dragon,
    Flower,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Tile {
    // Dots
    Dot1 = 0,
    Dot2,
    Dot3,
    Dot4,
    Dot5,
    Dot6,
    Dot7,
    Dot8,
    Dot9,

    // Bamboo
    Bamboo1 = 9,
    Bamboo2,
    Bamboo3,
    Bamboo4,
    Bamboo5,
    Bamboo6,
    Bamboo7,
    Bamboo8,
    Bamboo9,

    // Characters
    Character1 = 18,
    Character2,
    Character3,
    Character4,
    Character5,
    Character6,
    Character7,
    Character8,
    Character9,

    // Winds
    East = 27,
    South,
    West,
    North,

    // Dragons
    Red = 31,
    Green,
    White,

    // Flowers (MCR uses 8)
    Plum = 34,
    Orchid,
    Chrysanthemum,
    BambooF,
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Tile {
    pub const COUNT: usize = 42;

    pub const MCR_SET_COUNT: usize = 144;

    pub const ALL: [Tile; Tile::COUNT] = [
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
        Tile::Chrysanthemum,
        Tile::BambooF,
        Tile::Spring,
        Tile::Summer,
        Tile::Autumn,
        Tile::Winter,
    ];

    pub fn iter_all() -> impl Iterator<Item = Tile> {
        Self::ALL.iter().copied()
    }

    #[inline]
    pub fn id(self) -> u8 {
        self as u8
    }

    pub fn from_usize(value: usize) -> Option<Self> {
        Self::ALL.get(value).copied()
    }

    pub fn get_type(self) -> TileType {
        let id = self.id();
        match id {
            0..=8 => TileType::Dot,
            9..=17 => TileType::Bamboo,
            18..=26 => TileType::Character,
            27..=30 => TileType::Wind,
            31..=33 => TileType::Dragon,
            34..=41 => TileType::Flower,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn is_flower(self) -> bool {
        (self as u8) >= 34
    }

    #[inline]
    pub fn is_suited(self) -> bool {
        matches!(
            self.get_type(),
            TileType::Dot | TileType::Bamboo | TileType::Character
        )
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

#[derive(Debug, Clone)]
pub struct Wall(Vec<Tile>);

impl Wall {
    pub fn new_mcr() -> Self {
        let mut tiles: Vec<Tile> = Vec::with_capacity(Tile::MCR_SET_COUNT);
        for tile in Tile::iter_all() {
            let count: u8 = if tile.is_flower() { 1 } else { 4 };
            for _ in 0..count {
                tiles.push(tile);
            }
        }
        Wall(tiles)
    }

    pub fn shuffle(&mut self) {
        self.0.shuffle(&mut rand::rng());
    }

    pub fn draw(&mut self) -> Option<Tile> {
        self.0.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_mcr_wall() {
        let wall = Wall::new_mcr();
        assert_eq!(wall.0.len(), Tile::MCR_SET_COUNT);
    }

    #[test]
    fn test_draw_tile() {
        let mut wall = Wall::new_mcr();
        let tile = wall.draw();
        assert_eq!(tile, Some(Tile::Winter));
        assert_eq!(wall.0.len(), Tile::MCR_SET_COUNT - 1);
    }
}
