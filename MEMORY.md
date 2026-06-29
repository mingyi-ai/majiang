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
├── array_vec.rs    // generic ArrayVec<T, N>
├── engine.rs       // game driver (single round)
├── round/          // pure state machine
│   ├── mod.rs
│   ├── state.rs
│   └── actions.rs
├── structs/        // domain primitives
│   ├── mod.rs      // Wind, Wall
│   ├── tiles.rs    // Tile, TileType (sparse enum, 4-bit-gap layout)
│   ├── hand.rs     // Hand, BitTileCounts (rows: [u64; 4])
│   └── sets.rs     // Meld, Sequence, Triplet, Quad, Pair
└── solver/         // hu detection + decomposition + fan scoring
    ├── mod.rs                  ← NEW orchestration stubs (lazy iterators, StaticFanContext+DynamicFanContext)
    ├── decompose_standard.rs   ← OLD active DFS implementation (has #[cfg(test)])
    ├── decompose_special.rs    ← OLD active special hand detection
    ├── fan_solver.rs           ← OLD active search kernel (commented out in mod.rs)
    ├── mcr_tests.rs            ← OLD active 2448-line integration tests
    └── rules/
        ├── mod.rs              ← MACRO-GENERATED FanType + rule registry (mcr_rules!)
        │                          HandProfile (re-exported from profile.rs)
        │                          FanExclusionSet + FanCandidate (supporting types)
        ├── profile.rs          ← HandProfile (moved from view.rs)
        ├── helpers.rs          ← cand(), meld predicates (commented out, broken paths)
        ├── high.rs             ← 88/64/48pt rule check functions (commented out, broken paths)
        ├── mid_high.rs         ← 32/24/16pt rule check functions (commented out, broken paths)
        ├── mid_low.rs          ← 12/8/6pt rule check functions (commented out, broken paths)
        └── low.rs              ← 4/2/1pt rule check functions (commented out, broken paths)
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

### solver/ — Three-layer transitional architecture

The solver is mid-refactor. Three code layers coexist:

**Layer 1 — NEW stubs (solver/mod.rs):**
- `solve_fan` — pipeline: decompose → flat_map score → keep_highest_score
- `decompose_hand` → returns `Result<impl Iterator<Item = DecomposeResult>>` (lazy)
- `score_decomposition` → stub, returns `Vec<FanResult>`
- `keep_highest_score` → consumes iterator, O(n) single-pass max tracking
- `is_hu` → short-circuits on first decomposition via `iter.next().is_some()`
- Context split: `StaticFanContext` (per-deal constants) + `DynamicFanContext` (per-hand events)
- `Decomposition` enum defined inline (Standard, SevenPairs, ThirteenOrphans)

**Layer 2 — OLD active code (decompose_standard.rs, decompose_special.rs):**
- Full DFS decomposition for standard hands (suit-by-suit, Cartesian product)
- SevenPairs and ThirteenOrphans detection (complete, tested)
- These are functional but NOT wired to the new mod.rs stubs yet

**Layer 3 — OLD rule code with broken paths (rules/*.rs, fan_solver.rs, mcr_tests.rs):**
- 81 rule check functions exist across high/mid_high/mid_low/low (all commented out)
- `view.rs` contents moved to `rules/profile.rs` with fixed imports
- Reference types that no longer exist: `FanContext`, `super::super::types::*`
- `mcr_tests.rs` has 2448 lines of integration tests referencing old types
- `fan_solver.rs` has the max-score subset search kernel (commented out in mod.rs)

### `array_vec.rs` — generic fixed-capacity vec

```rust
pub(crate) struct ArrayVec<T: Copy, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}
```

Used by: `SuitDecomp`, `Hand::melds`, `Decomposition::Standard::sets`.

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

## Lazy iterator pipeline for score search
`decompose_hand` returns `impl Iterator<Item = DecomposeResult>`. The pipeline chains with `flat_map` (score each decomposition) and a consuming `keep_highest_score`. No intermediate `Vec<DecomposeResult>` or `Vec<Vec<FanResult>>` is materialized. See `solver/mod.rs`.

**Why**: Consistent with Rust's standard lazy iteration model. No external crate needed. Reduces allocation pressure for hands with many decompositions.

## `keep_highest_score` as a single-pass fold
Consumes `impl Iterator<Item = FanResult>`, tracks `Option<(Vec<FanResult>, u8)>`, folds with `score.cmp(&best_score)`, returns all ties. O(n) without re-computing `total_score()` on the best-so-far.

## `Decomposition` enum inline in mod.rs (not separate file)
**Why**: The enum is the target type for decomposition output and the input to scoring. Keep it close to the orchestration that creates and consumes it.

## Context split: StaticFanContext vs DynamicFanContext
**Why**: `StaticFanContext` (seat wind, prevalent wind, hand properties) is invariant across all decompositions of a hand. `DynamicFanContext` (kong replacement, rob kong) is event-specific. Separating them makes the rule interface explicit about what changes per decomposition vs per hand.

## `HandProfile` lives in `rules/profile.rs` (moved from view.rs)
**Why**: `HandProfile` converts `Decomposition` into a flat accessor for rule checking. It has exactly one consumer: the 81 rule functions. Moving to `rules/profile.rs` co-locates it with its consumers and gives a descriptive name. Import path: `use crate::solver::rules::HandProfile;`. The old `solver/view.rs` is kept as a dead file for now (its contents migrated to profile.rs with imports fixed).

## Tests co-located with rules (desired, not yet done)
Each rule file should have `#[cfg(test)] mod tests { ... }` with its rule's test cases. Shared helpers (hand builders, context builder, assertion fns) go in `rules/test_helpers.rs`. The 2448-line `mcr_tests.rs` is the consolidation target — it will be deleted once migration is complete.

# Solver Architecture — Macro-Generated Rule Registry

## `mcr_rules!` macro (IMPLEMENTED)

The old commented-out design required three touchpoints per rule. This has been replaced by a single `mcr_rules!` macro invocation in `rules/mod.rs`. Syntax:

```rust
mcr_rules! {
    BigFourWinds(88) / "Big Four Winds"
        excludes [BigThreeWinds, AllPungs, PrevalentWind, SeatWind, PungOfTerminalsOrHonors]
        => empty_rule,  // change to high::big_four_winds when module is activated
    // ... all 81 ...
}
```

This macro expands to:
- `FanType` enum (81 variants, `#[repr(u16)]`, compiler-assigned discriminants 0..80), `pub`
- `impl FanType { fn points(), fn name(), fn excludes_mask(), fn bit_index() }`
- `pub(crate) struct RuleEntry { fan_type: FanType, check: RuleFn }`
- `pub(crate) const ALL_RULES: &[RuleEntry]` — point-descending registration order
- `pub(crate) fn check_all(profile, static_ctx, dynamic_ctx) -> Vec<FanCandidate>`

**What was eliminated:**
- 81 hand-written `const X_EXCLUDES` arrays in old rule files
- 81 hand-written `fn points()` match arms in the old FanType impl
- 81 hand-written `fn name()` match arms in the old FanType impl
- Old `struct RuleEntry` and `const ALL_RULES` array (commented-out code deleted)
- The old `check_all` dispatch function
- Manual sync between excludes in rule file vs registry
- The `FanContext` aggregate is NOT used — rule functions take `(&StaticFanContext, &DynamicFanContext)` separately

**Why a declarative macro instead of proc-macro:**
- No extra dependency (no proc-macro crate)
- `macro_rules!` is sufficient — generating structured code from a flat list
- Rust assigns 0..N for fieldless enums, so `#[repr(u16)]` gives us bit indices for free
- Exclusion masks precomputed as `u128` bit arrays via `FanExclusionSet::set_bit()` in generated match arms

**Activation workflow:**
Currently all 81 rules are stubbed with `empty_rule` (returns `vec![]`). To activate a rule:
1. Uncomment `mod high;` (etc.) in the submodules section
2. Fix import paths in the activated module
3. Change the check function path in the `mcr_rules!` invocation from `empty_rule` to `high::big_four_winds`

## FanCandidate exclusivity model

`FanCandidate` carries `excludes_mask: FanExclusionSet`. The `check_all` function sets this mask from the precomputed `FanType::excludes_mask()`. The search kernel (`fan_solver.rs`) accumulates an `excluded_mask` as it selects candidates in score-descending order.

## No FanContext aggregate (deliberate)

Rule functions take `(&StaticFanContext, &DynamicFanContext)` separately. No unified wrapper. These contexts come from different sources and have different responsibilities — `StaticFanContext` is invariant per hand, `DynamicFanContext` is event-specific.

## RuleFn type signature

```rust
pub(crate) type RuleFn = fn(&HandProfile, &StaticFanContext, &DynamicFanContext) -> Vec<FanCandidate>;
```

# Known Issues

## Solver module

### 1. Rule submodules not yet activated
All 81 rule check functions are stubbed with `empty_rule`. The old rule files (high.rs, mid_high.rs, mid_low.rs, low.rs, helpers.rs) have broken import paths (`super::super::FanContext`, `super::super::types::*`) and are commented out from `rules/mod.rs`. Activating them requires:
- Fixing imports to use `super::{FanType, FanCandidate, FanExclusionSet}` and `super::profile::*`
- Changing `FanContext` references to `(&StaticFanContext, &DynamicFanContext)`

### 2. No fan extractors wired
The decomposition engine (decompose_standard.rs) produces `ConcealedDecompStd`, but the new mod.rs defines `Decomposition` and `DecomposeResult` independently. No code translates one to the other, and no code calls the 81 rule check functions.

### 3. Special hands unimplemented
`SevenPairs` and `ThirteenOrphans` detection exists (decompose_special.rs) but the new mod.rs decompose_hand returns `std::iter::empty()`.

### 4. Hu events accepted without minimum fan validation
Engine currently accepts any Hu without checking MCR 8-point minimum.

### 5. `can_hu()` ignores declared melds
Delegates to `crate::solver::is_hu(&self.concealed)` which expects 4 sets from concealed tiles. Hands with declared melds are incorrectly rejected.

### 6. `mcr_tests.rs` is a monolith
2448 lines of integration tests referencing old types. Must be split into per-rule `#[cfg(test)]` modules and deleted after migration.

## Other known issues
- **No multi-round game coordinator.** Single-round only.
- **Meld struct types (Sequence, Triplet, Quad) alongside Meld enum** — could inline.
- **`GameEvent::Action(PlayerAction::Skip)` is inhabited but never constructed.**
- **`add_to_row` is `pub(crate)` but only used from test code in a different module** — dead code flagged but harmless.

# Refactoring Completed

- [x] **`array_vec.rs`** — generic `ArrayVec<T, N>` at crate root
- [x] **`ConcealedDecomp` deleted** — `Decomposition::Standard` uses `ArrayVec<Meld, 4>` directly
- [x] **`Hand::melds`** — changed from `[Option<Meld>; 4]` to `ArrayVec<Meld, 4>`
- [x] **`SuitDecomp`** — uses `ArrayVec<Meld, 4>` instead of `[Option<Meld>; 4] + len: u8`
- [x] **Old files deleted** — `hu_solver.rs`, `fan/mod.rs`, `fan/fan_types.rs`, `fan/search.rs`
- [x] **Lazy iterator pipeline** — `decompose_hand` returns `impl Iterator`, `keep_highest_score` consumes `impl Iterator`, `solve_fan` uses `flat_map`
- [x] **Context split** — `StaticFanContext` + `DynamicFanContext` replacing unified `FanContext`
- [x] **is_hu early exit** — uses `iter.next().is_some()` instead of collecting to Vec
- [x] **Macro-generated rule registry** — `mcr_rules!` replaces FanType enum + ALL_RULES table + exclusion const arrays (all 81 rules)
- [x] **HandProfile moved to rules/profile.rs** — fixed imports (`crate::solver::Decomposition`), co-located with consumers
- [x] **No FanContext aggregate** — rule functions take separated `(&StaticFanContext, &DynamicFanContext)`
- [x] **RuleFn type alias** — defined as `fn(&HandProfile, &StaticFanContext, &DynamicFanContext) -> Vec<FanCandidate>`
- [x] **empty_rule stub** — placeholder for unimplemented rules, returns `vec![]`
- [x] **All 81 exclusion lists extracted and embedded in macro invocation** — verified from source files
- [x] **All 5 rule submodules activated** — helpers, high, mid_high, mid_low, low
- [x] **All rule import paths fixed** — `super::super::view::*` → `super::profile::*`, `super::super::FanContext` → `super::super::{StaticFanContext, DynamicFanContext}`, `super::super::types::*` → `super::*`
- [x] **All 81 rule check functions wired** — `mcr_rules!` invocation updated from `empty_rule` to real paths (`high::big_four_winds`, etc.), except `GreaterHonorsAndKnittedTiles` (not yet implemented)
- [x] **All rule function signatures updated** — `(&FanContext)` → `(&StaticFanContext, &DynamicFanContext)`, context field accesses updated (`ctx.win_method` → `static_ctx.win_method`, `ctx.is_kong_replacement` → `dynamic_ctx.is_kong_replacement`, etc.)
- [x] **`wait_type` as separate parameter to `RuleFn`/`check_all`** — per-decomposition, not absorbed into `StaticFanContext`. 4 rules (edge_wait, closed_wait, single_wait, melded_hand) consume it; 77 ignore with `_wait_type`.

# Remaining Work (prioritized)

1. **Reconcile old rule code paths** — Fix `rules/*.rs` imports to match current module structure. Re-introduce `FanContext` aggregate. Compile view.rs under rules/profile.rs.

2. **Activate rule submodules** — Fix imports in high.rs, mid_high.rs, mid_low.rs, low.rs, helpers.rs. Change `FanContext` → `(&StaticFanContext, &DynamicFanContext)`. Wire check functions into the `mcr_rules!` invocation.

3. **Wire decomposition → scoring** — Connect decompose_standard.rs/decompose_special.rs output to the new `DecomposeResult` type, then call `check_all()` + `solve_max_score()`.

4. **Split mcr_tests.rs** — Move tests to per-rule files, create `rules/test_helpers.rs` for shared infrastructure.

5. **Plumb declared melds** — Pass `n_declared` through the solver pipeline.

6. **Minimum fan validation** — Wire 8-point minimum check in engine Hu acceptance.

7. **Implement special hand detection** — Knitted patterns.

8. **Add symmetry test** — `#[cfg(test)]` that verifies all exclusion relationships are bidirectional.

9. **Delete old view.rs** — Once profile.rs is confirmed working, remove the orphaned `solver/view.rs`.

# Open Questions
- Should Meld struct types be inlined into the flat Meld enum?
- `PlayerDecision::Exit` single variant — need `Exit { reason }` for network play?
- After macro generation, should FanType be `pub` or `pub(crate)`? Currently `pub` (exposed to mahjong_core consumers).
