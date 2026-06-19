from dataclasses import dataclass
from structs import Hand, Pair, Set


@dataclass(slots=True, frozen=True)
class SetInstance:
    payload: Set
    id: int


@dataclass(slots=True, frozen=True)
class StandardDecomposition:
    """A standard decomposition of a winning hand, consisting of 4 sets and a pair."""

    sets: tuple[SetInstance, SetInstance, SetInstance, SetInstance]
    pair: Pair


def decompose(_hand: Hand) -> list[StandardDecomposition]:
    pass
    return []
