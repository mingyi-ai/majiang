# Project Identity

Mahjong game engine implementing MCR rules. A pure state machine with explicit query/apply API.

**Goals:**
- Correct MCR rule enforcement (draw, self-action, discard, reaction resolution)
- Deterministic, testable engine core
- Clean API for UI/Engine wrapper to drive game flow
- Bit-packed hand representation for solver efficiency

**Non-goals:**
- Multi-player networking (handled by outer Engine layer)
- History accumulation (caller builds from engine outputs)
- AI / bot logic (player trait is the boundary)
- Snapshot resume / replay from arbitrary state

# Current Architecture

```
lib.rs
├── array_vec.rs    // generic ArrayVec<T, N> — replaces [Option<T>; N] + len patterns
├── engine.rs       // game driver (single round) — Engine<'a, P: Player>
├── round/          // pure state machine — State, Output, Input
│   ├── mod.rs      //   query/apply API, new_shuffled, deal
│   ├── state.rs    //   core types: PlayerAction, GameEvent, State internals
│   └── actions.rs  //   Hand mutation methods
├── structs/        // domain primitives
│   ├── mod.rs      //   Wind, Wall
│   ├── tiles.rs    //   Tile, TileType (sparse enum, 4-bit-gap layout)
│   ├── hand.rs     //   Hand, BitTileCounts (rows: [u64; 4], LSB-fill counts)
│   └── sets.rs     //   Meld, Sequence, Triplet, Quad, Pair
└── solver/         // hu detection + decomposition + fan scoring
    ├── mod.rs                  // orchestration: is_hu(), decompose()
    ├── decompose_standard.rs   // standard suit/honor DFS decomposition
    ├── decompose_special.rs    // special hand detection (stubs)
    ├── decomposition.rs        // Decomposition enum (target type)
    └── fan.rs                  // fan types + search kernel (flat)
```

## Layer breakdown

### Public API (re-exported from `mahjong_core`)
- `Engine`, `EngineOutput`, `EngineError` — game driver
- `GameResult` — `Hu { winner }` | `Draw`
- `PlayerDecision` — `Pick(PlayerAction)` | `Exit`
- `Player` trait — `fn decide(&self, options: &[PlayerAction]) -> PlayerDecision`
- `PlayerAction` — `Skip`, `Discard`, `Pong`, `Chow`, `Kong`, `Hu`, `ConcealedKong`, `AddedKong`
- `GameEvent` — `DrawTile { seat, tile }` | `Action(PlayerAction)`
- `Wind` — `East`, `South`, `West`, `North`

### Engine — Game driver
Borrows `[&'a Player; 4]`. Drives the loop, handles auto-advance, validates plays.

```rust
Engine::new(wind, turn, [&players; 4], &mut rng)
engine.run(on_initial_deal, on_game_event) -> Result<EngineOutput, EngineError>
```

Key invariants:
- `None` from `apply()` means internal mechanics (Skip, reaction all-skip). Not reported.
- `GameEvent::Action(PlayerAction::Skip { .. })` is never constructed.

### round/ — Pure state machine
- `State::new_shuffled(wind, turn, rng)` — shuffle wall, empty hands
- `State::query()` → read-only peek (`Output`)
- `State::apply(Input) -> Option<GameEvent>` — mutate, return event or None
- `State::deal()` → alternating East→South→West→North with immediate flower replacement

Phase transitions:
```
RequestDrawTile → Apply(DrawTile) → RequestSelfAction (non-flower) or RequestDrawTile (flower)
RequestSelfAction → Apply(SelfAction) → RequestDrawTile (kong/hu) or RequestDiscard (skip)
RequestDiscard → Apply(Discard) → RequestReaction(tile)
RequestReaction → Apply(Reactions) → RequestDiscard (winner) or RequestDrawTile (all skip/kong)
```

### structs/ — Domain primitives
- `Wind`: East/South/West/North, `next()`, `iter()`
- `Wall`: `[Tile; 144]` with `pointer: usize` + shuffle + yield_tile
- `Tile`: 42 variants (34 non-flower + 8 flower), sparse 4-bit-gap encoding
- `BitTileCounts`: `[u64; 4]` — 4 rows × 64 bits, LSB-fill counting
- `Hand`: `Copy`. concealed (BitTileCounts) + melds `ArrayVec<Meld, 4>` + flower_count
- `Meld`: flat enum over Sequence/Triplet/Quad structs

### solver/ — hu detection + Fan computation

**`solver/mod.rs` — orchestration**
- `is_hu(counts) -> bool` — short-circuit check, tries standard then special
- `decompose(counts) -> Vec<Decomposition>` — full enumeration for fan scoring (not yet wired)

**`solver/decompose_standard.rs` — standard decomposition algorithm**
- `decompose_standard(counts) -> Vec<StandardDecomp>` — recursive DFS per suit, Cartesian product across suits, honor parser
- Internal types: `SuitDecomp = ArrayVec<Meld, 4>` (stack-allocated DFS path), `SuitDecomps = Vec<SuitDecomp>`
- Algorithm: at each nibble, try pong (≥3 copies) or chow (3 consecutive), backtrack
- Honored separately: must be pungs or single pair
- Pair handled in honors or suits

**`solver/decompose_special.rs` — special hand detection (stubs)**
- `detect_seven_pairs` — placeholder
- `detect_thirteen_orphans` — placeholder
- Knitted patterns — placeholder

**`solver/decomposition.rs` — target types**
- `Decomposition::Standard { pair, sets: ArrayVec<Meld, 4> }`
- `Decomposition::SevenPairs`, `ThirteenOrphans`, etc. (for future use)

**`solver/fan.rs` — fan types + search (flat)**
- `FanType` — 6 of 81 MCR fans
- `solve_max_score` — iterative DFS for max-score compatible subset
- **No fan extractors exist yet** — the search kernel is scaffolding

### `array_vec.rs` — generic fixed-capacity vec
```rust
pub(crate) struct ArrayVec<T: Copy, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}
```
Replaces the duplicated `[Option<T>; N] + len` pattern found in:
- `SuitDecomp` (formerly `[Option<Meld>; 4] + len: u8`)
- `Hand::melds` (formerly `[Option<Meld>; 4]`)
- `ConcealedDecomp::sets` (deleted, folded into `Decomposition::Standard`)

# Important Decisions

## Two-type split: `PlayerAction` + `GameEvent`
**Why**: The old flat `Event` mixed two roles. `DrawTile` and internal mechanics aren't player choices.

## `State::apply` returns `Option<GameEvent>`
**Why**: Event feed contains only observable table events. Skips are implicit.

## Alternating deal sequence
**Why**: Mirrors real mahjong dealing procedure.

## Engine borrows Players (`[&'a P; 4]`)
**Why**: Same players reused across rounds without cloning.

## Two callbacks in `run()`: `on_initial_deal` + `on_event`
**Why**: Deal events are private per-player; game events are public.

## Explicit RNG threading
**Why**: Caller controls determinism.

## `ArrayVec` over `[Option<T>; N] + usize`
**Why**: Eliminated 3x copy-pasted `[Option<Meld>; 4]` patterns. `ArrayVec` provides safe push/pop/iter with no heap allocation and no `Option` overhead for unused slots. Uses `MaybeUninit<T>` internally — safe because the API only exposes initialized indices.

## `Decomposition` as the target type
**Why**: The `Decomposition` enum is the public contract — it represents every valid winning pattern (Standard, SevenPairs, ThirteenOrphans, etc.). Fan extractors pattern-match on it. `ConcealedDecomp` was a redundant bridge type that duplicated `Standard`'s shape with `Option` wrappers; it was removed.

## Solver directory structure
```
solver/
├── mod.rs                  → orchestration (is_hu, decompose)
├── decompose_standard.rs   → suit/honor DFS algorithm
├── decompose_special.rs    → special hand detection
├── decomposition.rs        → Decomposition enum
└── fan.rs                  → fan types + search (flat, not nested)
```
**Why**: Separates concerns clearly. Orchestration lives in `mod.rs`, algorithms in their own files, fan scoring flat (no subdir needed for ~300 lines).

# Known Issues

## Solver module

### 1. No fan extractors exist
`fan.rs` has `FanType`, `solve_max_score`, and `FanExclusionSet` — but no code translates a `Decomposition` into `Vec<FanCandidate>`. The search kernel is scaffolding. The entire pipeline from `decompose()` → extract candidates → `solve_max_score()` is not wired.

### 2. Special hands unimplemented
`SevenPairs`, `ThirteenOrphans`, and knitted patterns are defined in `Decomposition` but `decompose_special.rs` returns `None` for all. `is_hu()` ignores these patterns.

### 3. Hu events accepted without minimum fan validation
Engine currently accepts any Hu without checking MCR 8-point minimum.

### 4. `can_hu()` ignores declared melds
Delegates to `crate::solver::is_hu(&self.concealed)` which expects 4 sets from concealed tiles. Hands with declared melds (e.g., 3 declared + 1 concealed) are incorrectly rejected. Fix requires plumbing `n_sets` through.

### 5. `is_hu` short-circuit is partial
Currently tries standard decomposition (full enumeration) first, then special hands (stubs). For the fast "is it hu?" check, a proper short-circuit that stops as soon as any decomposition finds a valid split would be faster.

## Other known issues
- **No multi-round game coordinator.** Single-round only.
- **Meld struct types** (Sequence, Triplet, Quad) alongside Meld enum — could inline.
- **`GameEvent::Action(PlayerAction::Skip)` is inhabited but never constructed.**
- **`add_to_row` is `pub(crate)` but only used from test code in a different module** — dead code flagged but harmless.

# Refactoring Completed

The following structural changes from the MEMORY.md refactor plan have been implemented:

- [x] **`array_vec.rs`** — generic `ArrayVec<T, N>` at crate root
- [x] **`ConcealedDecomp` deleted** — `Decomposition::Standard` uses `ArrayVec<Meld, 4>` directly
- [x] **Solver restructured** — `mod.rs` (orchestration), `decompose_standard.rs`, `decompose_special.rs`, `decomposition.rs`, `fan.rs` (flat)
- [x] **`Hand::melds`** — changed from `[Option<Meld>; 4]` to `ArrayVec<Meld, 4>`
- [x] **`SuitDecomp`** — uses `ArrayVec<Meld, 4>` instead of `[Option<Meld>; 4] + len: u8`
- [x] **`push_meld`** — simplified to `self.melds.push(meld)` (ArrayVec panics on overflow)
- [x] **Fan module flattened** — `solver/fan.rs` instead of `solver/fan/` directory
- [x] **Old files deleted** — `hu_solver.rs`, `fan/mod.rs`, `fan/fan_types.rs`, `fan/search.rs`
- [x] **211 tests passing** — no regressions

Remaining (not yet done):
- [ ] `try_each_pair_in_suit` — hoisted invariant checks (partially done, can be further cleaned)
- [ ] `is_hu` short-circuit optimization — still calls full `decompose_standard`
- [ ] `combine_suits` / `combine_suits_with` deduplication — minor, low priority
- [ ] Fan extractors — requires `Decomposition` → `FanCandidate` conversion
- [ ] Plumb declared melds through solver — needs `n_sets` parameter
- [ ] Wire minimum fan check (8 points)
- [ ] Implement special hand detection

# Open Questions
- Should `StandardDecomp` be folded into `Decomposition::Standard` directly (remove the intermediate struct)?
- Should Meld struct types be inlined into the flat Meld enum?
- `PlayerDecision::Exit` single variant — need `Exit { reason }` for network play?
