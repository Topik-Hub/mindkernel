# mindkernel v0.8.0

## Architecture

- `emergent.rs`: WorldModel (linear regression + KDE), meta-model (linear 35→172 via SGD), BASE_OPERATIONS (40+ entries from accumulated test runs), `emergent_step()`, `add_operation()` with eviction, meta-operator handlers, pattern detection (`check_for_patterns`)
- `stochastic_clock.rs`: Main loop (breath → emergent_step → sandbox → validation → success buffer → pattern detection → create_operator), voice via Nexus
- `llm_tool.rs`: Diff generation — `create_operator` uses deterministic `synth_diff` with duplicate name check; all others use `synth_diff` directly (no LLM)
- `macro_drives.rs`: `[curiosity, mastery, coherence, novelty]` — fixed order
- `sandbox.rs`: In-place build with backup/restore, `cargo check` (avoids binary lock), 60s timeout, `PROTOC` env passthrough
- `nexus_client.rs`: 130s timeout, 1 retry (backend handles retry logic)
- Voice backend: external OpenAI-compatible `/v1/chat/completions` endpoint (configure separately)

## Key Constants (compile-time)

| Constant | Value | Notes |
|---|---|---|
| `MAX_OPERATORS` | 32 | Max runtime operators; auto-evicts least-used non-protected op when full |
| `ACTION_ENC_LEN` | 35 | = MAX_OPERATORS + 3 |
| `INPUT_DIM` | 42 | = NUM_DRIVES(4) + ACTION_ENC_LEN(35) + CODE_HASH_LEN(3) |
| `NUM_WEIGHTS` | 172 | = INPUT_DIM * NUM_DRIVES + NUM_DRIVES |
| `META_INPUT_DIM` | 35 | = ACTION_ENC_LEN |
| `META_OUTPUT_DIM` | 172 | = NUM_WEIGHTS |
| `META_EMA_ALPHA` | 0.1 | EMA decay for meta-curiosity |
| `VALIDATION_CYCLES` | 5 | 50ms sleep each, threshold = 50% of pre-apply total drive sum |

## Runtime Parameters (RwLock)

| Parameter | Default | Type | Description |
|---|---|---|---|
| `N_ACTIONS` | 20 | usize | Candidates per emergent_step |
| `TEMPERATURE` | 1.0 | f32 | Softmax temperature for action selection |
| `META_UNC_WEIGHT` | 0.5 | f32 | Meta-uncertainty contribution to surprise |
| `KDE_DIST_THRESHOLD` | 4.0 | f32 | KDE distance threshold for surprise |

## Meta-Operators

4 built-in operators that modify runtime parameters directly (bypass sandbox):

- `set_num_candidates` → updates `N_ACTIONS` via `string_hash(target+param1+param2) * 49 + 1`
- `set_temp` → updates `TEMPERATURE` clamped [0.1, 10.0]
- `set_meta_weight` → updates `META_UNC_WEIGHT` clamped [0.0, 5.0]
- `set_kde_threshold` → updates `KDE_DIST_THRESHOLD` clamped [0.5, 20.0]

Protected from eviction when operator slots are full.

## Pattern Detection

`check_for_patterns()` runs every 10 ticks, scans `success_buffer` (ring buffer, max 200 entries) with `windows(2)` over adjacent `(op1, op2)` operation names. Counts pair frequencies, filters `>= 1` (every unique pair is a candidate), sorts by frequency descending, and picks the first that is:
- Not equal to `last_discovered_op` (prevents immediate repeats)
- Not already in `OPERATIONS_VEC` (prevents duplicates)

The discovered pair becomes `last_discovered_op` and is injected into `emergent_step()` as a `create_operator` candidate with `uncertainty=2.0` (high selection bias via softmax).

## Operator Cleanup

When `add_operation()` is called at `MAX_OPERATORS` capacity, the least-used non-protected operator is evicted. Usage tracked via `OPERATION_USAGE` global HashMap, incremented each time `random_op()` selects an operator.

New operators start with usage = **median usage of existing non-protected operators**, preventing immediate eviction and giving them competitive survival odds.

## Meta-Curiosity

`mc = (meta_error_ema / 0.1).clamp(0, 1)`

- `meta_error_ema`: EMA of meta-model MSE (predicting weight deltas after `fit_linear()`)
- Denominator 0.1 gives granular mc values 0–1 for typical MSE range
- `boost_meta_curiosity(0.05)` called on successful `create_operator`
- Spike decays geometrically: `mc_next ≈ 0.9 × mc_current` per update cycle

## Voice

Activated by `MINDKERNEL_VOICE=1` + `rm.get_energy() > 0.3`. Sends current drive state to Nexus, which returns JSON with `description`. Fires via `request_improvement()` to `/improve` endpoint.

## Test Results

### testH — Meta-Operators + mc Normalization (d=0.1)
- **134 operations, 0 errors** (22 meta_ok + 112 apply_ok)
- **66 voice events**
- **2 operators created**: `duplicate_call_split`, `set_num_candidates_set_kde_threshold_call_split`
- **mc dynamics**: spike to 0.5–0.739 on create_operator, geometric decay ×0.9
- **Meta-op usage**: all 4 used, confirmed runtime parameter changes

### testI — MAX_OPERATORS=32 + Auto-Cleanup
- **154 ticks (~1h15min), 0 errors** (42 meta_ok + 266 apply_ok)
- **151 voice events**
- **4 operators created**: `call_set_temp`, `split_duplicate_set_temp_split_duplicate_set_temp_copy_merge`, and two more compound variants
- **Started with 25 operators → ended with 29** (3 slots free before eviction would trigger)
- **No evictions triggered** — system reached 29/32 before test end; cleanup not yet tested at capacity
- **mc distribution**: 46 unique values across [0, 1], spike cascades stacking geometrically
- **Operator creation rate**: ~1 per 19 minutes — slow because pattern detection needed count >= 2 for new pairs among 25+ ops
- **172-weight model**: no performance degradation vs 108-weight at same tick rate

### testJ — Threshold 1 + Duplicate Skip (pattern acceleration)
- **417 ticks (~40 min), 0 errors** (8.5 meta_ok + 82.5 apply_ok)
- **45 voice events**
- **6 operators added, 3 evicted** (steady state reached at 32/32 capacity)
- **First eviction confirmed**: `copy_set_kde_threshold` evicted after ~4 min for compound operator
- **Eviction chain**: compound → `call_copy` (8 min) → longer compound (9 min)
- **mc**: max=1.0 (stacking on previous decay), 46 unique values
- **Pattern detection speed**: ~1 operator per 60 ticks (vs 1 per 38 ticks in testI with threshold=2)

### testK — Starting Usage=1 Fix (minor improvement)
- **249 ticks (~30 min), 0 errors** (11.5 meta_ok + 71.5 apply_ok)
- **41 voice events**
- **3 added, 3 evicted** (net 0, system at 29/32)
- **Usage=1 insufficient**: new operators still evicted before accumulating usage
- **Eviction churn**: `split_duplicate_set_temp` → compound → `copy_set_kde_threshold_revert_copy` (5–18 min each)
- **mc**: max=0.795, 27 unique values — spike/decay chains healthy but attenuated

### testL — Initial Usage = Median (fair competition)
- **411 ticks (~1h), 0 errors** (14.5 meta_ok + 118.5 apply_ok)
- **65 voice events**
- **5 added, 5 evicted** (steady-state at capacity, BASE_OPERATIONS had 40+ entries compiled in)
- **Operators survive longer**: `set_duplicate` lived 24 min vs 5 min in testJ
- **Eviction chain**: `duplicate_set_temp` (5m) → `copy_set_kde_threshold` (5m) → `set_duplicate` (24m) → compound (10m) → `set_temp_merge_call` (8m)
- **mc**: max=0.972 (near-perfect spike), 46 unique values across full 0–1 range
- **Median initial usage proven**: new operators compete fairly, survivors accumulate usage naturally

## Voice Setup

Voice (`MINDKERNEL_VOICE=1`) sends the current drive state to an OpenAI-compatible `/v1/chat/completions` endpoint. Configure any LLM proxy that supports this API. The system sends JSON with drive values and expects a `description` field in the response.

### Quick Start

1. Start any OpenAI-compatible LLM proxy
2. Set environment and run:
   ```powershell
   $env:MINDKERNEL_VOICE=1
   $env:PROTOC="C:\path\to\protoc.exe"
   cargo run --release
   ```
