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
    ├── mod.rs
    ├── hu_solver.rs     // HuSolver — suit partition DFS, honor parser
    ├── decomposition.rs // Decomposition (target type), ConcealedDecomp (redundant bridge)
    └── fan/             // Scoring/fan computation
        ├── mod.rs
        ├── fan_types.rs // FanType enum (6 of 81 MCR fans), FanExclusionSet
        └── search.rs    // DFS max-score fan subset search
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
- `Tile`: 42 variants (34 non-flower + 8 flower), sparse 4-bit-gap encoding for bitwise row ops
- `BitTileCounts`: `[u64; 4]` — 4 rows × 64 bits, LSB-fill counting (`pub(crate)`)
- `Hand`: `Copy`. concealed (BitTileCounts) + melds `[Option<Meld>; 4]` + flower_count
- `Meld`: flat enum over Sequence/Triplet/Quad structs

### solver/ — Hu detection + Fan computation

**HuSolver** — standalone struct (methods, not instance state), `pub(crate)`.

**`is_hu(counts) -> bool`**: Calls `find_all_decompositions` and checks non-empty — wasteful, computes all decompositions just to check emptiness.

**`find_all_decompositions(counts) -> Vec<ConcealedDecomp>`**: Enumerates all valid (pair, sets) decompositions of concealed tiles only.

**Suit decomposition**: Recursive DFS per suit row (3 suits). At each nibble: try pong (≥3 copies) or chow (3 consecutive tiles present). Results cached per suit, combined via Cartesian product.

**Honor decomposition**: Each honor tile must form a pung (3 copies) or the single pair. Singles, concealed kongs (4 copies), multiple pairs rejected.

**Pair handling**: Honors pair → decompose suits without pair. Suit pair → iterate pair positions in that suit, re-decompose that suit after pair removal.

**Fan module** (`solver/fan/`):
- `FanType` — 6 of 81 MCR fans with `points()`, `name()`, exclusion rules
- `FanExclusionSet` — u128 bitset for mutual exclusions
- `solve_max_score(candidates) -> FanSolveResult` — iterative DFS for max-score subset

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

# Known Issues

## Solver module — critical analysis

### 1. `ConcealedDecomp` is the AI slop (not `Decomposition`)
The real analysis is reversed from what a casual read suggests:

**`Decomposition`** (the pub enum with 7 variants) is the **target type** — it represents what the solver should ultimately produce: a specific pattern like `Standard { pair, sets }`, `SevenPairs`, `ThirteenOrphans`, etc. This is the right abstraction for fan extractors to pattern-match on. It's correct to keep and expand.

**`ConcealedDecomp`** (the pub(crate) struct) is the **problem**:
- **Incomplete**: Only represents the concealed-tiles portion of a standard hand. Can't express special hands, can't combine with declared melds.
- **Duplicates purpose**: Mirrors `Decomposition::Standard` but with `Option<Meld>` + `len` instead of direct `[Meld; 4]`. Callers must convert it — or ignore it entirely.
- **Incapable for the future**: When `n_sets` parameter enters (hands with declared melds), ConcealedDecomp forces an awkward "emit partial result, caller appends declared melds" dance. Better to compute the full `Decomposition::Standard` directly inside the solver.

**Evidence**: `ConcealedDecomp` is constructed/consumed only inside `hu_solver.rs` and its tests. `Decomposition` is exported from `solver/mod.rs` — the public boundary. The bridge type (`ConcealedDecomp`) should be eliminated, and `HuSolver` should produce `Vec<Decomposition>` directly.

### 2. `[Option<Meld>; 4]` is copy-pasted in 3 places
```
hu_solver.rs:     melds: [Option<Meld>; 4],  // SuitDecomp
decomposition.rs: sets: [Option<Meld>; 4],  // ConcealedDecomp
hand.rs:          melds: [Option<Meld>; 4],  // Hand
```

All three are "fixed-size array with a count of how many slots are active" — a.k.a. `ArrayVec<Meld, 4>`. This pattern needs a generic reusable type. The `Wall` in `structs/mod.rs` uses the same pattern (`[Tile; WALL_SIZE]` + `pointer: usize`) at a larger scale.

### 3. `HuSolver` function signatures and flow are messy
- `find_all_decompositions` takes `&BitTileCounts` only — no `n_sets`, no declared melds context, so it always assumes 4 concealed sets. This breaks for hands with declared melds.
- `try_each_pair_in_suit` has an awkward `skip` flag with a nested suit loop that rechecks invariants that don't depend on pair position.
- `is_hu` calls `find_all_decompositions` (full enumeration) then checks `.is_empty()` — wasteful. A short-circuit "is decomposable?" check would be faster.
- `emit_solutions` hand-rolls the `Option<Meld>` → final array copy. Would be cleaner with `ArrayVec`.
- `combine_suits` and `combine_suits_with` are nearly identical — one calls the other with a specific parameter.

### 4. Fan module starts too deep (nested `fan/` dir)
The `solver/fan/` directory with `mod.rs`, `fan_types.rs`, `search.rs` is over-nested for what exists (~200 lines total across 3 files). Should be flat: `solver/fan.rs` (or two files if needed).

### 5. No fan extractors exist — pipeline disconnected
- `HuSolver` → produces `Vec<ConcealedDecomp>` (or eventually `Vec<Decomposition>`)
- Fan module → expects `Vec<FanCandidate>`
- No code translates between them

### 6. Special hands unimplemented
`SevenPairs`, `ThirteenOrphans`, `KnittedStraight`, etc. are defined in `Decomposition` but no code constructs them. `HuSolver` returns false for valid special hands.

## Other known issues
- **Hu events accepted without minimum fan validation.** Engine accepts any Hu without 8-point minimum.
- **`can_hu()` ignores declared melds.** Delegates to `HuSolver::is_hu(&self.concealed)` which expects 4 sets from concealed tiles.
- **No multi-round game coordinator.** Single-round only.
- **Meld struct types** (Sequence, Triplet, Quad) alongside Meld enum — could inline.
- **`GameEvent::Action(PlayerAction::Skip)` is inhabited but never constructed.**

# Refactor Plan — Solver Module

## Step 1: Add `ArrayVec<N, T>` to crate root
Create `mahjong_core/src/array_vec.rs` (or tile under `common.rs`) with a generic:

```rust
pub(crate) struct ArrayVec<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    len: usize,
}
```

With push/pop/len/iter/index/slice operations. No heap allocation, no `Option` overhead.

Replace:
- `SuitDecomp` → internal use of `ArrayVec<Meld, 4>` instead of `[Option<Meld>; 4] + len`
- `Hand::melds` → `ArrayVec<Meld, 4>` instead of `[Option<Meld>; 4]`
- `ConcealedDecomp::sets` → `ArrayVec<Meld, 4>` (see Step 2 — this goes away)
- `Wall` → evaluate: Wall is consumed from the front (not push/pop from back), so ArrayVec's API doesn't fit. Keep Wall's current structure; extract only if a bidirectional ring buffer is warranted.

## Step 2: Delete `ConcealedDecomp`, produce `Decomposition` directly
- Remove `ConcealedDecomp` from `decomposition.rs`
- Have the solver produce `Vec<Decomposition>` where `Standard` variant is built with `ArrayVec<Meld, 4>`
- The `Decomposition::Standard` needs its `sets` field changed from `[Meld; 4]` to `ArrayVec<Meld, 4>` (or keep `[Meld; 4]` but handle n_sets externally)

Decision point: should `Decomposition::Standard` always hold 4 melds (combining declared + concealed), or hold only the concealed portion?
- **Option A**: Full hand always. Declared melds fed in, solver fills remaining from concealed. `sets: [Meld; 4]` (no ArrayVec needed).
- **Option B**: Variable-length. `sets: ArrayVec<Meld, 4>` — n_sets depends on how many declared melds exist.

Option A is simpler if we always plumb declared melds. Option B is more flexible but makes `Decomposition` variable-length.

Recommendation: **Option A** — declare `sets: [Meld; 4]`, accept `declared_melds: &[Meld]` as parameter, solver fills the remaining slots from concealed tiles.

## Step 3: Restructure solver directory
```
solver/
├── mod.rs                 → public API (orchestration)
├── decompose_standard.rs  → standard suit/honor decomposition algorithms
├── decompose_special.rs   → special hand detection (SevenPairs, etc.)
├── decomposition.rs       → Decomposition enum (types only)
└── fan.rs                 → fan types + search (flat, no subdir)
```

### `solver/mod.rs` — orchestration
```rust
pub fn decompose(counts: &BitTileCounts, declared: &[Meld]) -> Vec<Decomposition>
pub fn is_hu(counts: &BitTileCounts, declared: &[Meld]) -> bool
```

Short-circuit `is_hu` — return true as soon as ANY decomposition found, don't enumerate all.

### `solver/decompose_standard.rs`
Move from `hu_solver.rs`:
- `decompose_suit` / `dfs_decompose` — suit DFS, unchanged algorithm, cleaner signatures
- `parse_honors` — unchanged
- `decompose_standard(counts, n_concealed_sets) -> Vec<[Meld; 4]>` — top-level entry, returns completed set arrays ready to merge with declared melds

### `solver/decompose_special.rs`
- `detect_seven_pairs(counts) -> Option<Decomposition>`
- `detect_thirteen_orphans(counts) -> Option<Decomposition>`
- Additional specials as needed

### `solver/decomposition.rs`
Keep only the `Decomposition` enum. Remove `ConcealedDecomp`.
Consider using `ArrayVec<Meld, 4>` for `Standard::sets` if Option A is rejected.

### `solver/fan.rs` (flat)
Merge `fan_types.rs` + `search.rs` into one file. Keep `solve_max_score` but add prefacing comment: "No fan extractors exist yet — this is scaffolding."

## Step 4: Fix function signatures and flow

### Fix `try_each_pair_in_suit`
```rust
// Before: skip flag with inner loop mixed with pair-position loop
fn try_each_pair_in_suit(suit, counts, honor_pungs, suit_cache, out) {
    for shift in pair_positions_in_row(counts.rows[suit]) {
        // ... checks other suits' decomposability in each iteration ...
        let mut skip = false;
        for i in 0..3 {
            if i == suit { continue; }
            if counts.rows[i] != 0 && suit_cache[i].is_empty() { skip = true; break; }
        }
    }
}

// After: pre-check, then only iterate pair positions
fn try_each_pair_in_suit(suit, counts, honor_pungs, suit_cache, out) {
    // Other suits' decomposability doesn't depend on pair position
    for i in 0..3 {
        if i != suit && counts.rows[i] != 0 && suit_cache[i].is_empty() {
            return; // No decomposition possible regardless of pair position
        }
    }
    for shift in pair_positions_in_row(counts.rows[suit]) {
        // ... just decompose this suit with pair removed ...
    }
}
```

### Fix `is_hu` short-circuit
```rust
// Before: always enumerates all decompositions
pub fn is_hu(counts: &BitTileCounts) -> bool {
    !Self::find_all_decompositions(counts).is_empty()
}

// After: early-exit path
pub fn is_hu(counts: &BitTileCounts, declared: &[Meld]) -> bool {
    if let Some(honors) = parse_honors(counts.rows[3]) {
        if honors.pair.is_some() && all_suits_decomposable(&counts.rows) {
            return true; // Fast path: honors pair + all suits decomposable
        }
    }
    // fall through to full enumeration
    !Self::find_all_decompositions(counts, declared).is_empty()
}
```

### Clean up `combine_suits` / `combine_suits_with` duplication
```rust
fn combine_suits(cache: &[SuitDecomps; 3]) -> SuitDecomps {
    combine_triple(&cache[0], &cache[1], &cache[2])
}

fn combine_suits_with(cache: &[SuitDecomps; 3], idx: usize, repl: &SuitDecomps) -> SuitDecomps {
    let s = |i| if i == idx { repl } else { &cache[i] };
    combine_triple(s(0), s(1), s(2))
}
```

## Step 5: Add fan extractors (wire the pipeline)
```rust
// solver/fan.rs
pub fn extract_candidates(decomp: &Decomposition, ctx: &RoundContext) -> Vec<FanCandidate> {
    match decomp {
        Decomposition::Standard { pair, sets } => {
            let mut candidates = vec![];
            if is_all_pungs(sets) { candidates.push(all_pungs_candidate()); }
            if is_half_flush(pair, sets) { candidates.push(half_flush_candidate()); }
            // etc.
            candidates
        }
        Decomposition::SevenPairs { .. } => { /* special hand fans */ }
        // etc.
    }
}
```

## Step 6: Plumb declared melds through
- `State::reaction_options_by_seat` and `self_action_options` call `hand.can_hu()` / `hand.can_hu_on()`
- These call `HuSolver::is_hu(&self.concealed)` — ignores `self.melds`
- Fix: `HuSolver::is_hu(&self.concealed, &self.melds)` — or `HuSolver::is_hu(&self)` taking full `Hand`

## Step 7: Wire minimum fan check (8 points)
- After `is_hu()` passes, run fan scoring
- Reject Hu if total score < 8 points (MCR minimum)
- This requires Steps 5-6 to be complete first

# Open Questions
- Should `Decomposition::Standard.sets` be `[Meld; 4]` (always full, declared melds plumbed in) or `ArrayVec<Meld, 4>` (variable length)?
- How to handle the `Wall` — keep `pointer: usize` or refactor to a generic consume-front type? ArrayVec doesn't match the semantics (consumption from front, not accumulation at back).
- Should `SeatState::discards` also use ArrayVec? Currently sparse `[Option<Tile>; 32]` with `push_discard` finding first None.
- `PlayerDecision::Exit` single variant — need `Exit { reason }` for network play?
