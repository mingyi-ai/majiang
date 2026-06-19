from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum

from hand_solver import StandardDecomposition
from structs import (
    Bamboo,
    Character,
    Chow,
    Dot,
    Dragon,
    Kong,
    Pong,
    Tile,
    Wind,
)


class FanType(IntEnum):
    # MCR subset we need right now
    BIG_FOUR_WINDS = 1
    LITTLE_FOUR_WINDS = 2
    ALL_TERMINALS_AND_HONORS = 3
    HALF_FLUSH = 4
    ALL_PUNGS = 5

    # existing prototype fan (kept because it is useful)
    ALL_HONORS = 6

    @property
    def points(self) -> int:
        return {
            FanType.BIG_FOUR_WINDS: 88,
            FanType.LITTLE_FOUR_WINDS: 64,
            FanType.ALL_TERMINALS_AND_HONORS: 32,
            FanType.HALF_FLUSH: 6,
            FanType.ALL_PUNGS: 6,
            FanType.ALL_HONORS: 64,
        }[self]

    def excludes(self) -> frozenset["FanType"]:
        """Fans that may not be scored together with this fan.

        This encodes the Non-Repeat principle (implied fans) and a few obvious
        mutual exclusions. It is deliberately minimal; we can extend it as we
        add more fans.
        """

        if self == FanType.BIG_FOUR_WINDS:
            # Big Four Winds implies All Pungs; also mutually exclusive with
            # Little Four Winds.
            return frozenset({FanType.ALL_PUNGS, FanType.LITTLE_FOUR_WINDS})

        if self == FanType.LITTLE_FOUR_WINDS:
            return frozenset({FanType.BIG_FOUR_WINDS})

        if self == FanType.ALL_HONORS:
            # All Honors is a strict special case of All Terminals and Honors.
            return frozenset({FanType.ALL_TERMINALS_AND_HONORS})

        return frozenset()

    def display_name(self) -> str:
        return {
            FanType.BIG_FOUR_WINDS: "Big Four Winds",
            FanType.LITTLE_FOUR_WINDS: "Little Four Winds",
            FanType.ALL_TERMINALS_AND_HONORS: "All Terminals and Honors",
            FanType.HALF_FLUSH: "Half Flush",
            FanType.ALL_PUNGS: "All Pungs",
            FanType.ALL_HONORS: "All Honors",
        }[self]


@dataclass(slots=True, frozen=True)
class FanCandidate:
    fan: FanType
    used_set_ids: frozenset[int]
    uses_pair: bool = False


def _flatten_tiles(decomposition: StandardDecomposition) -> list[Tile]:
    return [
        tile for s in decomposition.sets for tile in s.payload.to_tiles()
    ] + decomposition.pair.to_tiles()


def _is_honor(tile: Tile) -> bool:
    return tile in Wind or tile in Dragon


def _is_suited(tile: Tile) -> bool:
    return isinstance(tile, (Character, Bamboo, Dot))


def _is_terminal(tile: Tile) -> bool:
    if isinstance(tile, Character):
        return tile in (Character.Character1, Character.Character9)
    if isinstance(tile, Bamboo):
        return tile in (Bamboo.Bamboo1, Bamboo.Bamboo9)
    if isinstance(tile, Dot):
        return tile in (Dot.Dot1, Dot.Dot9)
    return False


def _all_set_ids(decomposition: StandardDecomposition) -> frozenset[int]:
    return frozenset(s.id for s in decomposition.sets)


def _extract_big_four_winds(
    decomposition: StandardDecomposition,
) -> FanCandidate | None:
    wind_sets = [
        s
        for s in decomposition.sets
        if isinstance(s.payload, (Pong, Kong)) and s.payload.payload in Wind
    ]

    if len(wind_sets) != 4:
        return None

    winds = {s.payload.payload for s in wind_sets}
    if winds != {Wind.East, Wind.South, Wind.West, Wind.North}:
        return None

    return FanCandidate(
        fan=FanType.BIG_FOUR_WINDS,
        used_set_ids=frozenset(s.id for s in wind_sets),
        uses_pair=False,
    )


def _extract_little_four_winds(
    decomposition: StandardDecomposition,
) -> FanCandidate | None:
    wind_sets = [
        s
        for s in decomposition.sets
        if isinstance(s.payload, (Pong, Kong)) and s.payload.payload in Wind
    ]

    if len(wind_sets) != 3:
        return None

    pair_tile = decomposition.pair.payload
    if pair_tile not in Wind:
        return None

    winds = {s.payload.payload for s in wind_sets}
    if len(winds) != 3:
        return None

    if pair_tile in winds:
        return None

    return FanCandidate(
        fan=FanType.LITTLE_FOUR_WINDS,
        used_set_ids=frozenset(s.id for s in wind_sets),
        uses_pair=True,
    )


def _extract_all_pungs(decomposition: StandardDecomposition) -> FanCandidate | None:
    if any(isinstance(s.payload, Chow) for s in decomposition.sets):
        return None

    if not all(isinstance(s.payload, (Pong, Kong)) for s in decomposition.sets):
        return None

    return FanCandidate(
        fan=FanType.ALL_PUNGS,
        used_set_ids=_all_set_ids(decomposition),
        uses_pair=True,
    )


def _extract_all_honors(decomposition: StandardDecomposition) -> FanCandidate | None:
    tiles = _flatten_tiles(decomposition)
    if all(_is_honor(t) for t in tiles):
        return FanCandidate(
            fan=FanType.ALL_HONORS,
            used_set_ids=_all_set_ids(decomposition),
            uses_pair=True,
        )
    return None


def _extract_all_terminals_and_honors(
    decomposition: StandardDecomposition,
) -> FanCandidate | None:
    tiles = _flatten_tiles(decomposition)
    if all(_is_honor(t) or _is_terminal(t) for t in tiles):
        return FanCandidate(
            fan=FanType.ALL_TERMINALS_AND_HONORS,
            used_set_ids=_all_set_ids(decomposition),
            uses_pair=True,
        )
    return None


def _extract_half_flush(decomposition: StandardDecomposition) -> FanCandidate | None:
    tiles = _flatten_tiles(decomposition)

    suited = [t for t in tiles if _is_suited(t)]
    if not suited:
        return None

    suit_type = type(suited[0])
    if any(type(t) is not suit_type for t in suited):
        return None

    if not any(_is_honor(t) for t in tiles):
        return None

    return FanCandidate(
        fan=FanType.HALF_FLUSH,
        used_set_ids=_all_set_ids(decomposition),
        uses_pair=True,
    )


def extract_fan_candidates(decomposition: StandardDecomposition) -> list[FanCandidate]:
    """Extract all fan *instances* satisfied by this fixed decomposition."""

    out: list[FanCandidate] = []

    for cand in (
        _extract_big_four_winds(decomposition),
        _extract_little_four_winds(decomposition),
        _extract_all_honors(decomposition),
        _extract_all_terminals_and_honors(decomposition),
        _extract_half_flush(decomposition),
        _extract_all_pungs(decomposition),
    ):
        if cand is not None:
            out.append(cand)

    # De-dupe by (fan, used_set_ids, uses_pair)
    uniq: dict[tuple[FanType, frozenset[int], bool], FanCandidate] = {}
    for c in out:
        uniq[(c.fan, c.used_set_ids, c.uses_pair)] = c

    return list(uniq.values())
