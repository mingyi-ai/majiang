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
    ├── mod.rs                  → orchestration: solve_fan, is_hu, decompose_hand, score_decomposition
    ├── decompose_standard.rs   → standard suit/honor DFS decomposition
    ├── decompose_special.rs    → special hand detection (seven pairs, thirteen orphans)
    ├── fan_solver.rs           → search kernel: solve_max_score (DFS + exclusion check)
    └── rules/
        ├── mod.rs              → mcr_rules! macro: generates FanType, impl, ALL_RULES, check_all
        │                          FanExclusionSet, FanCandidate, RuleFn, empty_rule stub
        ├── profile.rs          → HandProfile: pre-computed decomposition view for rule checking
        ├── helpers.rs          → cand(), meld predicates (is_pung_or_kong, is_chow, etc.)
        ├── test_helpers.rs     → concealed_hand, declared_hand, solve_default, assertion helpers
        ├── high.rs             → 88/64/48pt rule check functions + 23 tests
        ├── mid_high.rs         → 32/24/16pt rule check functions + 33 tests
        ├── mid_low.rs          → 12/8/6pt rule check functions + 32 tests
        └── low.rs              → 4/2/1pt rule check functions + 38 tests
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

### Solver public API
- `solve_fan(hand, &StaticFanContext, &DynamicFanContext) -> Result<Vec<FanResult>, SolverError>`
- `FanResult { fans: Vec<FanInstance> }`, `FanInstance { fan_type: FanType, score, used_set_mask, uses_pair }`
- `StaticFanContext` (seat wind, prevalent wind, win method, winning tile, flower count, concealment flags, last tile flags)
- `DynamicFanContext` (kong replacement flag, rob kong flag)

### Engine — Game driver
Borrows `[&'a Player; 4]`. Drives the loop, handles auto-advance, validates plays.

### round/ — Pure state machine
Standard MCR phase transitions: DrawTile → SelfAction → Discard → Reaction → ...

### structs/ — Domain primitives
- `Wind`, `Wall`, `Tile` (42 variants, sparse 4-bit-gap encoding), `BitTileCounts` (`[u64; 4]`)
- `Hand`: `Copy` — concealed (`BitTileCounts`) + melds (`ArrayVec<Meld, 4>`) + flower_count
- `Meld`: flat enum over Sequence/Triplet/Quad

### solver/ — Pipeline architecture

```
solve_fan(hand, static_ctx, dynamic_ctx)
  │
  ├─ decompose_hand(hand)
  │     → decompose_standard (concealed tiles → ConcealedDecompStd)
  │     → detect_seven_pairs, detect_thirteen_orphans
  │     → combine with declared melds → Vec<DecomposeResult>
  │
  ├─ flat_map lazy iteration
  │     score_decomposition(decomp, static_ctx, dynamic_ctx)
  │       ├─ HandProfile::from_decomposition(decomp.decompositions)
  │       ├─ rules::check_all(profile, static_ctx, dynamic_ctx, wait_type)
  │       │     → iterates ALL_RULES (81 entries), calls each check fn
  │       │     → embeds precomputed excludes_mask on each candidate
  │       ├─ fan_solver::solve_max_score(candidates)
  │       │     → iterative DFS: exclusion check + sig_key dedup + score monotonicity
  │       └─ converts FanSolveResult → FanResult
  │
  └─ keep_highest_score(results) → Vec<FanResult> (ties kept)
```

# Important Decisions

## Macro-generated rule registry (`mcr_rules!`)
**Why**: Eliminated 81 hand-written const arrays + 81 points() match arms + 81 name() match arms + RuleEntry struct + ALL_RULES table + check_all dispatch. Single source of truth: one macro invocation with 81 entries.

**Exclusion model**: Explicit lists in the macro invocation, extracted from the MCR rulebook. No auto-derivation from structural categories — MCR exclusions are semantic, not structural (many pairs that logically could coexist are excluded by rulebook flat, and vice versa). A `#[cfg(test)]` symmetry check verifies bidirectionality.

**Why not a category-based system**: The MCR rulebook does not define STRUCTURAL/ENVIRONMENTAL/GLOBAL categories. Attempting to derive exclusions from set-mask subset relationships would produce incomplete results — the rulebook's explicit "does not combine with" lists are the authoritative source.

## No unified `FanContext`
**Why**: `StaticFanContext` (per-hand constants: seat wind, concealment, win method) and `DynamicFanContext` (per-event flags: kong replacement, rob kong) come from different sources and have different lifetimes. A unified wrapper would obscure this.

## `WaitType` as a separate parameter to rule functions
**Why**: Wait type is per-decomposition, not per-hand. It's known only after `decompose_hand` yields a result. Not absorbed into either context.

## `keep_highest_score` as a single-pass fold
Consumes `impl Iterator<Item = FanResult>`, folds with `score.cmp(&best_score)`, returns all ties. O(n).

## Lazy iterator pipeline
`decompose_hand` returns `impl Iterator<Item = DecomposeResult>`. Pipeline chains with `flat_map` and a consuming `keep_highest_score`.

## HandProfile in rules/profile.rs (not solver/view.rs)
**Why**: `HandProfile` converts `Decomposition` into a flat accessor for rule checking. It has exactly one consumer: the 81 rule functions. Co-located.

## The Five MCR Scoring Principles (fan(x,y) notation)

We model each fan as operating on a set of melds indexed 0..n. Notation:

| Term | Meaning | Example fan | `used_set_mask` |
|---|---|---|---|
| `fan(A)` | Single-set — evaluates one meld against context | `SeatWind(0)` | `0b0001` |
| `fan(A, B)` | Structural — combines two melds | `PureDoubleChow(0,1)` | `0b0011` |
| `fan(A, B, C)` | Structural — combines three melds | `PureTripleChow(0,1,2)` | `0b0111` |
| `fan(∅)` | Hand-property — no melds consumed | `AllChows` | `0` |

Fans with `popcount(used_set_mask) ≥ 2` are **structural** and subject to Account-Once
bridging. Popcount < 2 or mask = 0 are exempt.

---

### Principle 1: Non-Repeat (Implication Exclusion)

**Rule**: If fan P structurally implies fan Q, Q cannot be scored alongside P.  
**Enforced by**: `excludes_mask` in `mcr_rules!`.

| Scenario | Result | Why |
|---|---|---|
| `QuadrupleChow(0,1,2,3) + PureDoubleChow(0,1)` | ✗ | QuadrupleChow's {0,1,2,3} ⊇ PureDoubleChow's {0,1} |
| `PureTripleChow(0,1,2) + PureDoubleChow(0,1)` | ✗ | Structural subset |
| `BigFourWinds(0,1,2,3) + LittleFourWinds(0,1,2)` | ✗ | Structural subset |
| `BigThreeDragons(0,1,2,3) + DragonPung(0)` | ✗ | BigThreeDragons inherently contains three dragon pungs |
| `AllTerminalsAndHonors(∅) + AllTerminals(∅)` | ✗ | Hand-property subset: AllT&H implies AllTerminals |

Some exclusions are pure rulebook declarations (e.g., `BigFourWinds` excludes `AllPungs`)
with no structural basis — these must be enumerated in the macro. Most structural
subset exclusions are auto-derivable but listed explicitly for defense-in-depth.

---

### Principle 2: Non-Separation (Unbreakable Sets)

**Rule**: A meld cannot be split into smaller units to claim additional fans.  
**Enforced by**: The hand parser — each segmentation is an independent branch.

| Scenario | Result | Why |
|---|---|---|
| Pung of East → `SeatWind` + `PungOfTerminals` | ✓ | Pung stays intact for both calculations |
| 4 identical tiles → 2 pairs + 1 pung | ✗ | Would require splitting tiles across meld boundaries. Parser produces one valid segmentation per branch |

---

### Principle 3: Non-Identical (No Double-Counting)

**Rule**: The same fan instance cannot appear twice.  
**Enforced by**: `sig_key` dedup (`fan_type | uses_pair | used_set_mask`).

| Scenario | Result |
|---|---|
| Two `cand(AllPungs, 0, true)` from same check | ✗ Second rejected (identical sig_key) |
| Same `AllPungs` candidate from different passes | ✗ Same sig_key regardless of source |

---

### Principle 4: Free Choice (Highest Score)

**Rule**: Choose the highest-scoring configuration. Ties keep all.  
**Enforced by**: `keep_highest_score` + DFS search kernel.

---

### Principle 5: Account-Once (Bridge Counter)

**Rule**: Each meld can bridge from already-used sets to remaining new sets at most once.  
**Enforced by**: `bridge_count: [u8; 4]` in `solve_max_score`.

Only structural fans (popcount ≥ 2) trigger this check. Hand-property (mask = 0) and
single-set (popcount = 1) fans are exempt — they don't combine sets.

| Scenario | Step 1 | Step 2 | Step 3 | Result |
|---|---|---|---|---|
| `fan(0,1) + fan(0,2) + fan(0,3)` | Anchor: used={0,1} | Bridge: set0→2. b[0]=1 | Bridge: set0→3. b[0]≥1 → ✗ | ✗ |
| `fan(0,1) + fan(2,3)` | Anchor: used={0,1} | Disjoint: used={0,1,2,3} | — | ✓ |
| `fan(0,1) + fan(0,2) + fan(0)` | Anchor: used={0,1} | Bridge: set0→2. b[0]=1 | Single-set fan(0): exempt | ✓ |
| `fan(0,1) + fan(2,3) + fan(1,2)` | Anchor: used={0,1} | Disjoint: used={0,1,2,3} | Bridge: set1→2, set2→1. Both b=0→1 | ✓ |
| `fan(0) + fan(1) + fan(0,2)` | Single-set | Single-set | Bridge: {0}→2. b[0]=0→1 | ✓ |

---

### How the constraints interact

| Mechanism | Principle | Auto-derivable? |
|---|---|---|
| `excludes_mask` | Non-Repeat + rulebook extras | Partial (subset relationships) |
| `bridge_count` | Account-Once | Always (structural only) |
| `sig_key` dedup | Non-Identical | Always |
| `keep_highest_score` | Free Choice | Always |
| Multiple segmentations | Non-Separation | Always (parser) |

The `bridge_count` mechanism ensures Account-Once is correct regardless of exclusion list
completeness. A full audit of the 81 exclusion lists against the MCR rulebook is the top
priority for correctness.

## `is_hu` swallows `decompose_hand` errors
`is_hu` converts `Err(InvalidHand)` to `Ok(false)` because callers (round/actions) call `is_hu` on hands in intermediate states. `solve_fan` preserves the original error behavior.

# Known Issues

## Solver module

### 1. Exclusion lists have NOT been verified against the MCR rulebook
The current exclusion lists were extracted from the old `const X_EXCLUDES` arrays in the rule files. These are known to be incomplete. Official MCR "does not combine with" declarations from the rulebook should be cross-referenced and the macro invocation updated.

### 2. Exclusion symmetry is NOT verified
A `#[cfg(test)]` test that verifies all exclusion relationships are bidirectional needs to be written.

### 3. Rule check functions are stubs / not yet implemented
After uncommenting the modules and fixing imports, the check functions contain their original matching logic. However, many may be incomplete or incorrect — they haven't been tested against known hand examples.

### 4. Wait-type detection is a stub
`stub_wait_type()` returns `WaitType::Multiple`. The actual algorithm for determining wait type from decomposition is not implemented. This affects 4 rules (edge_wait, closed_wait, single_wait, melded_hand).

### 5. Test coverage is insufficient
Current test suite: 328 passing, 9 failing. The 9 failures are expected (rule stubs). However, the 328 passing tests are:
- 211 pre-existing structural tests (decomposition, engine, state machine)
- 126 migrated integration tests (split from mcr_tests.rs)

The migrated tests use hand examples from the MCR rulebook but **assert against incomplete rule implementations**. They test that the pipeline doesn't crash, not that correct scores are computed. True test coverage requires reference data from a mature MCR engine.

### 6. Hu validation missing MCR 8-point minimum
Engine accepts any Hu without checking minimum 8-point requirement.

### 7. `can_hu()` ignores declared melds
Currently delegates to `crate::solver::is_hu(&self.concealed)` which only sees concealed tiles. Hands with declared melds (e.g., 3 declared + 1 concealed set) are incorrectly rejected.

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
- [x] **HandProfile moved to rules/profile.rs** — fixed imports, co-located with consumers
- [x] **No FanContext aggregate** — rule functions take `(&StaticFanContext, &DynamicFanContext, WaitType)`
- [x] **RuleFn type alias** — `fn(&HandProfile, &StaticFanContext, &DynamicFanContext, WaitType) -> Vec<FanCandidate>`
- [x] **All 81 exclusion lists extracted and embedded in macro invocation**
- [x] **All 5 rule submodules activated and import paths fixed**
- [x] **All 81 rule check functions wired** (GreaterHonorsAndKnittedTiles → empty_rule stub)
- [x] **Full pipeline wired** — decompose_hand → score_decomposition → check_all → solve_max_score
- [x] **solve_fan made pub** with necessary types pub (StaticFanContext, DynamicFanContext, WinMethod, WaitType, FanInstance fields)
- [x] **mcr_tests.rs deleted** — 126 tests migrated to per-rule-file `#[cfg(test)]` modules
- [x] **test_helpers.rs created** — shared test infrastructure

# Remaining Work (prioritized)

1. **Audit exclusion lists against MCR rulebook** — Cross-reference all 81 "does not combine with" declarations. Add missing exclusions to the `mcr_rules!` invocation.

2. **Add exclusion symmetry test** — `#[cfg(test)]` in rules/mod.rs that verifies every exclusion is bidirectional.

3. **Build comprehensive test suite using reference engine** — Use an existing mature MCR engine as ground truth. For each rule, construct hand examples + expected scores + expected combined fans. Test the full pipeline (solve_fan) against these expectations. This will reveal gaps in both rule implementations and exclusion lists.

4. **Fix declared meld plumbing** — `can_hu()` should account for declared melds. Currently only checks concealed tiles.

5. **Implement wait-type detection** — Replace `stub_wait_type()` with proper algorithm based on the decomposition structure and winning tile.

6. **Wire 8-point minimum check** — Engine should reject Hu actions where `solve_fan` returns no result meeting minimum point threshold, or where the highest score is below 8.

7. **Implement special hand detection** — Knitted patterns (GreaterHonorsAndKnittedTiles, LesserHonorsAndKnittedTiles, KnittedStraight).

# Open Questions
- Should Meld struct types be inlined into the flat Meld enum?
- `PlayerDecision::Exit` single variant — need `Exit { reason }` for network play?
- Reference engine: which existing MCR implementation to use for test data?
