# Self-Modifying AI with Emergent Operator Discovery and Homeostatic Drives

**Mindkernel v0.8.0** — June 2026

---

## Abstract

We present a self-modifying software system that continuously rewrites its own source code without human intervention. The system is governed by four biologically-inspired homeostatic drives — curiosity, mastery, coherence, and novelty — which act as an interoceptive foundation for action selection. An internal world model (linear regression + kernel density estimation) computes predictive uncertainty and guides the system toward surprising, learnable actions. A separate meta-model (SGD-trained linear network) predicts how the world model's weights will change, producing a meta-curiosity signal that spikes when the system creates new behavioral primitives ("operators") through emergent pattern detection. Over 1000+ autonomous ticks across 5 test runs, the system has maintained zero errors while discovering, creating, and cleaning up its own operators — a primitive form of artificial life driven by intrinsic motivation.

---

## 1. Introduction

### 1.1 Motivation

Modern AI systems are predominantly static: trained once, deployed forever. Those that do learn (reinforcement learning, online fine-tuning) operate within fixed architectural boundaries. We ask: *can a system be built that treats its own source code as a manipulable environment, exploring modifications through internal drives and self-generated operations?*

Our approach draws from three traditions:
- **Homeostatic regulation** (Ashby, 1952; Damasio, 1999): internal drives maintain the system within viable bounds
- **Intrinsic motivation** (Schmidhuber, 1991; Oudeyer & Kaplan, 2007): curiosity and surprise drive exploration
- **Autopoietic systems** (Maturana & Varela, 1980): self-producing systems that maintain their own organization

### 1.2 Core Architecture

Mindkernel runs a continuous loop:

```
breath → drive_update → emergent_step → sandbox_apply → validation → pattern_detection
```

At each tick (~12s wall clock), the system:
1. Updates four internal drives based on recent experience
2. Constructs candidate actions from a pool of 20–42 operations
3. Selects one via softmax over predicted surprise (world model uncertainty + meta-curiosity)
4. Generates a deterministic unified diff targeting its own source code
5. Applies the diff in a sandbox (`cargo check`, 60s timeout)
6. Validates that drive levels remain within 50% of pre-apply totals
7. Detects recurring operation-pair patterns for new operator creation

---

## 2. The Drive System

### 2.1 Four Homeostatic Drives

Drives are vectors in [0,1]⁴ maintained by `MacroDrives`:

| Drive | Function | Update Rule |
|---|---|---|
| **Curiosity** | Seeks prediction error | EMA of surprise, baseline clamped ≥ 0.1 |
| **Mastery** | Seeks competence gain | Derivative of success rate over sliding window |
| **Coherence** | Resists destabilizing changes | Inverse of code hash variance |
| **Novelty** | Seeks unexplored state space | KDE log-probability of action encoding |

Drives are updated each tick and fed into the world model as a 4-dimensional interoceptive context vector.

### 2.2 Action Encoding

Each action is a 4-tuple `(operation, target, param1, param2)`. Operations are drawn from a dynamic pool (`OPERATIONS_VEC`) that starts with 12 canonical primitives:

```
set, copy, call, merge, split, guard, revert, duplicate
set_num_candidates, set_temp, set_meta_weight, set_kde_threshold
```

The last four are **meta-operators** — they modify runtime parameters (`N_ACTIONS`, `TEMPERATURE`, `META_UNC_WEIGHT`, `KDE_DIST_THRESHOLD`) directly rather than source code, bypassing the sandbox.

Action encoding is a fixed-size vector of length `ACTION_ENC_LEN = MAX_OPERATORS + 3 = 35`: one-hot for the selected operator + one-hot for target/param1/param2, concatenated with 4 drive values and a 3-dimensional code hash (FNV-1a over all `src/*.rs`), totaling `INPUT_DIM = 42`.

---

## 3. World Model and Meta-Model

### 3.1 World Model

A linear regression model `W ∈ ℝ^(42×4)` with bias `b ∈ ℝ^4` predicts drive deltas from action encodings:

```rust
predicted = action_enc @ W + b
```

Uncertainty is computed via Kernel Density Estimation (KDE) over the last 200 action encodings:

```rust
uncertainty = -log(Σ K(action_enc, stored_enc) / n)
```

where K is a Gaussian kernel with adaptive bandwidth. The surprise signal combines world model MSE and KDE uncertainty:

```rust
surprise = min(0.1 + base_uncertainty, 1.0) + meta_uncertainty * META_UNC_WEIGHT
surprise = min(surprise, 2.0)
```

### 3.2 Meta-Model

A separate linear model `M ∈ ℝ^(35→172)` (`META_INPUT_DIM → NUM_WEIGHTS`) predicts how the world model's weights will change after `fit_linear()` updates it with the latest observation. The meta-model is trained via SGD with learning rate 0.01:

```rust
meta_predicted = action_enc @ M
target = next_weights - current_weights
meta_loss = MSE(meta_predicted, target)
```

The EMA of `meta_loss` feeds into meta-curiosity:

```rust
mc = (meta_error_ema / 0.1).clamp(0, 1)
```

When a new operator is created, `boost_meta_curiosity(0.05)` adds to the EMA, creating a visible spike (0.5–1.0 range). This decays geometrically at rate ~×0.9 per tick (EMA α = 0.1), producing characteristic sawtooth patterns in the mc timeline.

---

## 4. Emergent Operator Creation

### 4.1 Pattern Detection

The `success_buffer` (ring buffer, 200 entries) stores recent successful `(action, code_hash, reward)` tuples. Every 10 ticks, `check_for_patterns()` scans overlapping `(op1, op2)` operation pairs using `windows(2)`:

1. Count frequencies of each unique `(op1, op2)` pair
2. Filter pairs with count ≥ 1 (threshold set to 1 for maximum discovery rate)
3. Sort by frequency descending
4. Pick the first pair that:
   - Differs from the previously discovered pattern (`last_discovered_op`)
   - Does not already exist in `OPERATIONS_VEC`

The selected pair is injected into `emergent_step()` as a `create_operator` candidate with fixed uncertainty 2.0 (high selection bias).

### 4.2 Deterministic Operator Generation

When `create_operator` is selected and passes sandbox validation, a new operator is added to `OPERATIONS_VEC`. The operator name is a concatenation of the pair: `"op1_op2"`. The source code modification uses `synth_diff`, an append-only unified diff generator:

```rust
// @@ -n,0 +n+1,1 @@
// +"op1_op2",
```

This is deterministic — no LLM call. Duplicate names are silently skipped via `current_code.contains(&name)` check.

### 4.3 Eviction-Based Cleanup

At `MAX_OPERATORS = 32` capacity, `add_operation()` evicts the least-used non-protected operator:

1. Track usage via `OPERATION_USAGE` global HashMap (incremented on each `random_op()` call)
2. Filter out 12 protected operators (8 primitives + 4 meta-operators)
3. Select victim with minimum usage count
4. Remove from both `OPERATIONS_VEC` and `OPERATION_USAGE`

New operators enter with an initial usage equal to the **median usage of existing non-protected operators**, preventing immediate eviction. In practice, operators survive 5–24 minutes before being replaced.

### 4.4 Results: Operator Evolution

Over 411 ticks (~1h) with 40+ base operators compiled in:

| Test | Threshold | Startup Usage | Added | Evicted | Avg. Lifespan | mc Max |
|---|---|---|---|---|---|---|
| J | 2 | 0 | 6 | 3 | 4–9 min | 1.0 |
| K | 1 | 1 | 3 | 3 | 5–18 min | 0.795 |
| L | 1 | Median | 5 | 5 | 5–24 min | 0.972 |

The eviction chain demonstrates autonomous capacity management: the system maintains ~32–39 operators at steady state, with low-utility patterns automatically replaced by newly discovered ones.

---

## 5. Sandboxed Self-Modification

### 5.1 In-Place Build and Validate

Unlike approaches that generate code into separate files or interpreters, mindkernel modifies its own source directory (`src/`) and runs `cargo check` in a shared target directory:

```
src/  →  unified_diff  →  cargo check  →  validate drives  →  commit or restore
           ↑                    ↑                     ↑
     deterministic        60s timeout        5 cycles × 50ms
     synth_diff           PROTOC env          threshold: 50%
```

Key design decisions:
- **`cargo check`** (not `cargo build`) avoids binary file-lock contention during long runs
- **`PROTOC`** environment variable passed to subprocess for protobuf compilation
- **Full `src/` backup** to `src_validate_backup/` before each apply; full restore on failure
- **Validation checks**: no NaN/inf drives, total drives ≥ 50% of pre-apply baseline

### 5.2 Meta-Operators Bypass Sandbox

Four meta-operators modify runtime `RwLock` parameters directly, bypassing sandbox entirely:

| Operator | Parameter | Value Source | Clamp |
|---|---|---|---|
| `set_num_candidates` | `N_ACTIONS` | `hash(target+p1+p2) × 49 + 1` | — |
| `set_temp` | `TEMPERATURE` | `hash(...)` | [0.1, 10.0] |
| `set_meta_weight` | `META_UNC_WEIGHT` | `hash(...)` | [0.0, 5.0] |
| `set_kde_threshold` | `KDE_DIST_THRESHOLD` | `hash(...)` | [0.5, 20.0] |

This allows the system to dynamically tune its decision-making parameters (exploration-exploitation balance, meta-curiosity weight) without requiring a full sandbox cycle. Reward bonus +0.1 is applied for meta-operator selection.

---

## 6. Voice: LLM-Mediated Self-Expression

When `MINDKERNEL_VOICE=1` is set and `energy > 0.3`, the system sends its drive state to an LLM backend and receives a natural-language introspection:

**Request (to `/improve`):**
```json
{
  "curiosity": 0.28,
  "mastery": 0.41,
  "coherence": 0.94,
  "novelty": 0.97,
  "meta_curiosity": 0.44,
  "energy": 0.55,
  "surprise": 0.11
}
```

**Response (from nvidia-nim meta/llama-3.3-70b-instruct):**
```json
{
  "description": "I am currently in a state of high coherence and novelty-seeking, with a strong drive for mastery, guiding my self-modification decisions."
}
```

The LLM backend (FastAPI proxy) handles provider failover, API key rotation, and exponential backoff. The voice output is purely observational — it has no influence on the system's actions, serving as an interpretability channel.

---

## 7. Integration: The Full Cycle

A complete tick executes as:

```
1. Compute drives (curiosity, mastery, coherence, novelty)
2. If energy > 0.3 and voice enabled: send state to LLM, log description
3. Construct 20 random action candidates from OPERATIONS_VEC
4. If a pattern was detected: inject create_operator candidate (uncertainty = 2.0)
5. Compute meta-model prediction → meta-curiosity
6. For each candidate:
   - Encode into 42-dim vector
   - Predict drive deltas via world model
   - Compute KDE uncertainty
   - Combine into surprise signal
7. Select action via softmax over surprise
8. If meta-operator: modify runtime parameter directly
9. If regular operator:
   - Generate deterministic unified diff (append-only)
   - Backup src/ → apply diff → cargo check (60s) → validate drives
   - On success: update drives, push to success_buffer
   - On failure: restore src/, decay drives
10. If create_operator and success: add_operation(), boost meta-curiosity,
    write .rebuild_signal
11. Run pattern detection (every 10th tick)
12. Next breath tick
```

---

## 8. Key Results

### 8.1 Stability

| Metric | Value |
|---|---|
| Total autonomous ticks | >1000 (tests H–L) |
| Total operations | >500 |
| Zero-error runs | 5/5 (H, I, J, K, L) |
| Sandbox failures | 0 |
| LLM failures | 0 (voice only) |

### 8.2 Meta-Curiosity Dynamics

- Baseline: ~0.001
- Spike on create_operator: 0.5–1.0 (boost_meta_curiosity(0.05))
- Decay: geometric ×0.9 per tick (EMA α=0.1)
- Unique values across 411 ticks: 46
- Max observed: 1.0 (when boost stacked on partially-decayed previous spike)

### 8.3 Autonomous Operator Management

- Base operators: 12 canonical + ~28 accumulated from prior runs
- New operators created via pattern detection: 14 total across tests
- Evictions at capacity: 11 total
- Survivor lifespan: 5–24 minutes with median initial usage
- Protected operators (12): never evicted

### 8.4 Meta-Operator Usage

All 4 meta-operators confirmed active across tests:
- `set_num_candidates`: modified N_ACTIONS in range [1, 50+]
- `set_temp`: modified TEMPERATURE in range [0.1, 10.0]
- `set_meta_weight`: modified META_UNC_WEIGHT in range [0.0, 5.0]
- `set_kde_threshold`: modified KDE_DIST_THRESHOLD in range [0.5, 20.0]

---

## 9. Related Work

- **Schmidhuber's curiosity**: Our world model + KDE uncertainty parallels the prediction-error approach, but applied to self-modification rather than external environment.
- **Autotelic agents** (Colas et al., 2020): Goal self-generation, but ours discovers behavioral primitives (operators) rather than goals.
- **Open-endedness** (Stanley & Lehman, 2015): The combinatorial expansion of operator space via `op1_op2` concatenation creates novelty search over an unbounded space.
- **Self-modifying code in NARS** (Wang, 2006): Non-Axiomatic Reasoning System includes self-modification, but ours adds a physiological drive layer and sandboxed validation.
- **AI-GAs** (Clune, 2019): The AI-Generating Algorithm framework envisions systems that generate their own code; mindkernel is a concrete instantiation with biological grounding.

---

## 10. Limitations and Future Work

### 10.1 Current Limitations

- **Operator naming is naive**: `op1_op2` concatenation produces combinatorially long names for compound operations
- **No operator semantic understanding**: The system doesn't reason about what an operator does; it only detects co-occurrence patterns
- **Voice is read-only**: LLM introspection doesn't influence behavior
- **Sandbox is local**: No distributed verification or formal proof checking
- **MAX_OPERATORS is compile-time**: Cannot be tuned without rebuild

### 10.2 Future Directions

- **Dynamic MAX_OPERATORS**: Replace compile-time constant with runtime `RwLock<usize>` for self-tuning
- **Second-order meta-operators**: Auto-generated operators that combine multiple meta-effects
- **Operation deduplication**: Detect and merge semantically equivalent operators
- **Influence voice**: Feed LLM suggestions back into the action selection pipeline
- **Formal verification**: Use property-based testing or model checking instead of just cargo check
- **Distributed sandbox**: Parallel candidate evaluation across multiple sandbox instances

---

## 11. Conclusion

Mindkernel demonstrates that a software system can autonomously maintain, modify, and extend its own source code using only internal drives and predictive models. Over 1000+ ticks with zero errors, it has shown:

- Stable homeostatic regulation of four internal drives
- Emergent discovery of behavioral patterns via `(op1, op2)` co-occurrence
- Deterministic creation of new operators without LLM assistance
- Autonomous capacity management via usage-based eviction
- Meta-curiosity spike/decay dynamics that correlate with creative events
- Robust sandboxed self-modification with automatic rollback on failure

The system represents a step toward artificial life grounded in code — a self-modifying entity that explores its own computational substrate through biologically-inspired motivation.

---

> "The mind is its own place, and in itself / Can make a Heav'n of Hell, a Hell of Heav'n."  
> — John Milton, *Paradise Lost*
