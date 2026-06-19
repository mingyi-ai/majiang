from __future__ import annotations

from dataclasses import dataclass

from fan_rules import FanCandidate, FanType, extract_fan_candidates


def _fan_bit(fan: FanType) -> int:
    # FanType is a small IntEnum; use its integer value as the bit index.
    return 1 << int(fan)


SigKey = int


@dataclass(slots=True, frozen=True)
class FanInstance:
    id: int
    fan: FanType
    used_set_ids: frozenset[int]
    uses_pair: bool

    score: int
    sig_key: SigKey
    excludes_mask: int


@dataclass(slots=True, frozen=True)
class FanSearchSpace:
    """Precomputed, compact view of all candidate fan instances."""

    instances: tuple[FanInstance, ...]  # by id; ids are 0..n-1
    order: tuple[int, ...]  # instance ids sorted by (-score, id)


@dataclass(slots=True, frozen=True)
class FanSolveResult:
    total_score: int
    fans: tuple[FanInstance, ...]


@dataclass(slots=True, frozen=True)
class Frame:
    resume_pos: int
    max_allowed_score: int
    excluded_mask: int
    total_score: int
    path_len: int


def _build_search_space(candidates: list[FanCandidate]) -> FanSearchSpace:
    # Map arbitrary set ids to compact bit positions for fast signatures.
    all_set_ids: set[int] = set()
    for c in candidates:
        all_set_ids.update(c.used_set_ids)

    set_id_to_bit = {sid: i for i, sid in enumerate(sorted(all_set_ids))}
    set_bits = len(set_id_to_bit)

    def sig_key(fan: FanType, uses_pair: bool, used_set_ids: frozenset[int]) -> SigKey:
        # Compact signature:
        # - used sets encoded as bitmask
        # - pack (fan, uses_pair, mask) into a single int
        mask = 0
        for sid in used_set_ids:
            mask |= 1 << set_id_to_bit[sid]

        # Layout: [fan][uses_pair][mask]
        # This is only required to be stable within this search space.
        return (int(fan) << (set_bits + 1)) | (int(uses_pair) << set_bits) | mask

    instances: list[FanInstance] = []

    for i, c in enumerate(candidates):
        fan = c.fan
        score = fan.points

        excludes_mask = 0
        for ex in fan.excludes():
            excludes_mask |= _fan_bit(ex)

        instances.append(
            FanInstance(
                id=i,
                fan=fan,
                used_set_ids=c.used_set_ids,
                uses_pair=c.uses_pair,
                score=score,
                sig_key=sig_key(fan, c.uses_pair, c.used_set_ids),
                excludes_mask=excludes_mask,
            )
        )

    order = tuple(sorted(range(len(instances)), key=lambda j: (-instances[j].score, j)))

    return FanSearchSpace(instances=tuple(instances), order=order)


def _is_eligible(
    inst: FanInstance,
    *,
    excluded_mask: int,
    max_allowed_score: int,
    used_sig_keys: set[SigKey],
) -> bool:
    if inst.score > max_allowed_score:
        return False
    if excluded_mask & _fan_bit(inst.fan):
        return False
    if inst.sig_key in used_sig_keys:
        return False
    return True


def _unwind_path(
    instances: tuple[FanInstance, ...],
    path: list[int],
    used_sig_keys: set[SigKey],
    *,
    to_len: int,
) -> None:
    while len(path) > to_len:
        removed_id = path.pop()
        used_sig_keys.remove(instances[removed_id].sig_key)


def _search_best(space: FanSearchSpace) -> tuple[int, list[int]]:
    instances = space.instances
    order = space.order

    # Mutable search state
    path: list[int] = []
    used_sig_keys: set[SigKey] = set()

    excluded_mask = 0
    max_allowed_score = instances[order[0]].score
    total_score = 0

    best_score = -1
    best_path: list[int] = []

    stack: list[Frame] = []
    pos = 0

    while True:
        # Try to pick the next eligible instance.
        for idx in range(pos, len(order)):
            inst_id = order[idx]
            inst = instances[inst_id]

            if not _is_eligible(
                inst,
                excluded_mask=excluded_mask,
                max_allowed_score=max_allowed_score,
                used_sig_keys=used_sig_keys,
            ):
                continue

            # Choose this instance; push a frame for "try next" at this depth.
            stack.append(
                Frame(
                    resume_pos=idx + 1,
                    max_allowed_score=max_allowed_score,
                    excluded_mask=excluded_mask,
                    total_score=total_score,
                    path_len=len(path),
                )
            )

            path.append(inst_id)
            used_sig_keys.add(inst.sig_key)

            excluded_mask |= inst.excludes_mask
            max_allowed_score = inst.score
            total_score += inst.score

            # Deterministic combination enumeration (avoid permutations).
            pos = idx + 1
            break
        else:
            # Leaf
            if total_score > best_score:
                best_score = total_score
                best_path = path.copy()

            if not stack:
                break

            # Backtrack to previous depth.
            frame = stack.pop()
            pos = frame.resume_pos
            max_allowed_score = frame.max_allowed_score
            excluded_mask = frame.excluded_mask
            total_score = frame.total_score
            _unwind_path(
                instances,
                path,
                used_sig_keys,
                to_len=frame.path_len,
            )

    return best_score, best_path


def solve_highest_fan(decomposition) -> FanSolveResult:
    candidates = extract_fan_candidates(decomposition)

    if not candidates:
        return FanSolveResult(total_score=0, fans=())

    space = _build_search_space(candidates)

    best_score, best_path = _search_best(space)

    best_fans = tuple(space.instances[i] for i in best_path)
    return FanSolveResult(total_score=best_score, fans=best_fans)


if __name__ == "__main__":
    # Test case:
    # 3 East, 3 South, 3 West, 3 North, 2 Character9
    # Expected scoring fans:
    # - Big Four Winds (88)
    # - All Terminals and Honors (32)
    # - Half Flush (6)

    from hand_solver import SetInstance, StandardDecomposition
    from structs import Character, Pair, Pong, Wind

    decomposition = StandardDecomposition(
        sets=(
            SetInstance(Pong(Wind.East, True), id=0),
            SetInstance(Pong(Wind.South, True), id=1),
            SetInstance(Pong(Wind.West, True), id=2),
            SetInstance(Pong(Wind.North, True), id=3),
        ),
        pair=Pair(Character.Character9),
    )

    res = solve_highest_fan(decomposition)

    fan_types = {f.fan for f in res.fans}
    assert res.total_score == 88 + 32 + 6
    assert FanType.BIG_FOUR_WINDS in fan_types
    assert FanType.ALL_TERMINALS_AND_HONORS in fan_types
    assert FanType.HALF_FLUSH in fan_types
    assert FanType.ALL_PUNGS not in fan_types
    assert FanType.LITTLE_FOUR_WINDS not in fan_types

    print("PASS", res.total_score, [f.fan.display_name() for f in res.fans])
