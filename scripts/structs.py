from dataclasses import dataclass
from enum import IntEnum, auto


class Character(IntEnum):
    Character1 = 0
    Character2 = auto()
    Character3 = auto()
    Character4 = auto()
    Character5 = auto()
    Character6 = auto()
    Character7 = auto()
    Character8 = auto()
    Character9 = auto()


class Bamboo(IntEnum):
    Bamboo1 = 9
    Bamboo2 = auto()
    Bamboo3 = auto()
    Bamboo4 = auto()
    Bamboo5 = auto()
    Bamboo6 = auto()
    Bamboo7 = auto()
    Bamboo8 = auto()
    Bamboo9 = auto()


class Dot(IntEnum):
    Dot1 = 18
    Dot2 = auto()
    Dot3 = auto()
    Dot4 = auto()
    Dot5 = auto()
    Dot6 = auto()
    Dot7 = auto()
    Dot8 = auto()
    Dot9 = auto()


class Wind(IntEnum):
    East = 27
    South = auto()
    West = auto()
    North = auto()


class Dragon(IntEnum):
    Red = 31
    Green = auto()
    White = auto()


type SuitedTile = Character | Bamboo | Dot

type Honor = Wind | Dragon

type NonFlowerTile = SuitedTile | Honor


class Flower(IntEnum):
    Plum = 34
    Orchid = auto()
    BambooFlower = auto()
    Chrysanthemum = auto()
    Spring = auto()
    Summer = auto()
    Autumn = auto()
    Winter = auto()


type Tile = NonFlowerTile | Flower

type Hand = list[Tile]


@dataclass(slots=True, frozen=True)
class Chow:
    payload: (
        tuple[Character, Character, Character]
        | tuple[Bamboo, Bamboo, Bamboo]
        | tuple[Dot, Dot, Dot]
    )
    is_concealed: bool

    def __post_init__(self):
        first_tile = self.payload[0]
        if not all(isinstance(tile, type(first_tile)) for tile in self.payload):
            raise ValueError("All tiles in a Chow must be of the same suit")

        if not all(
            tile.value == first_tile.value + i for i, tile in enumerate(self.payload)
        ):
            raise ValueError("Tiles in a Chow must be consecutive")

    def to_tiles(self) -> list[Tile]:
        return list(self.payload)


@dataclass(slots=True, frozen=True)
class Pong:
    payload: Tile
    is_concealed: bool

    def to_tiles(self) -> list[Tile]:
        return [self.payload] * 3


@dataclass(slots=True, frozen=True)
class Kong:
    payload: Tile
    is_concealed: bool

    def to_tiles(self) -> list[Tile]:
        return [self.payload] * 4


type Set = Chow | Pong | Kong


@dataclass(slots=True, frozen=True)
class Pair:
    payload: Tile

    def to_tiles(self) -> list[Tile]:
        return [self.payload] * 2
