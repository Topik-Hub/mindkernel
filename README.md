<p align="center">
  <img src="https://img.shields.io/badge/status-active-brightgreen" alt="Status">
  <img src="https://img.shields.io/badge/rust-2024+-orange" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
  <img src="https://img.shields.io/badge/tests-0%20errors-2ea44f" alt="Tests">
  <img src="https://img.shields.io/badge/ops-500%2B-8a2be2" alt="Operations">
</p>

# mindkernel

A self-modifying AI system that continuously rewrites its own source code using biologically-inspired homeostatic drives and emergent operator discovery.

> Zero human intervention. The system explores, modifies, and extends its own source code autonomously.

---

## Table of Contents

- [Overview](#overview)
- [How It Works](#how-it-works)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Key Features](#key-features)
- [Drives](#drives)
- [World Model](#world-model)
- [Operator Evolution](#operator-evolution)
- [Test Results](#test-results)
- [Configuration](#configuration)
- [Research](#research)
- [License](#license)

---

## Overview

Mindkernel is a Rust-based system that treats its own source code as a manipulable environment. Governed by four internal drives — curiosity, mastery, coherence, novelty — it generates, tests, and validates modifications to its own code at runtime.

At each tick (~12 seconds), the system:
1. Updates its internal drives based on recent experience
2. Constructs candidate actions from a dynamic pool of operators
3. Selects one via softmax over predicted surprise
4. Generates a deterministic unified diff targeting its own source
5. Applies the diff in a sandbox (`cargo check`, 60s timeout)
6. Validates drive stability and rolls back on failure
7. Detects recurring operation patterns and creates new operators autonomously

---

## Quick Start

### Prerequisites

- Rust toolchain (edition 2024)
- Protocol Buffers compiler (`protoc`)

### Run

```powershell
$env:PROTOC="C:\path\to\protoc.exe"
cargo run --release
```

### Voice (optional)

Mindkernel can send its drive state to an LLM for natural-language introspection. Set up any OpenAI-compatible `/v1/chat/completions` endpoint:

```powershell
$env:MINDKERNEL_VOICE=1
cargo run --release
```

The system logs its own introspective descriptions at each tick when energy exceeds threshold.

---

## Architecture

| Component | Source | Role |
|---|---|---|
| Drives | [`macro_drives.rs`](src/macro_drives.rs) | Curiosity, mastery, coherence, novelty |
| World Model | [`emergent.rs`](src/emergent.rs) | Linear regression + KDE, meta-model, pattern detection |
| Main Loop | [`stochastic_clock.rs`](src/stochastic_clock.rs) | Breath cycle, action selection, voice |
| Sandbox | [`sandbox.rs`](src/sandbox.rs) | In-place build, backup/restore, validation |
| Diff Engine | [`llm_tool.rs`](src/llm_tool.rs) | Deterministic unified diff generation |
| LLM Client | [`nexus_client.rs`](src/nexus_client.rs) | 130s timeout, 1 retry |
| Meta-Cognition | [`meta_cog.rs`](src/meta_cog.rs) | Higher-order self-modeling |
| Resource Manager | [`resource_manager.rs`](src/resource_manager.rs) | Operator lifecycle and eviction |
| Self-Applier | [`self_applier.rs`](src/self_applier.rs) | Diff application and rollback |
| Self-Model | [`self_model.rs`](src/self_model.rs) | Internal world representation |
| Self-Modifier | [`self_modifier.rs`](src/self_modifier.rs) | Source code mutation engine |
| Micro-Drives | [`micro_drives.rs`](src/micro_drives.rs) | Fine-grained drive subsystems |
| Public APIs | [`public_apis.rs`](src/public_apis.rs) | External interface handlers |
| Syscall | [`syscall.rs`](src/syscall.rs) | System call abstraction |
| Lambda | [`lambda_function.rs`](src/lambda_function.rs) | Serverless entry point |

### Data Flow

```
                    ┌─────────────┐
                    │   Drives    │
                    │ (4 vectors) │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  Emergent   │
                    │   Step     │
                    │ ┌────────┐ │
                    │ │20-42   │ │
                    │ │Actions │ │
                    │ └───┬────┘ │
                    │     │      │
                    │ ┌───▼────┐ │
                    │ │Softmax │ │
                    │ │select  │ │
                    │ └───┬────┘ │
                    └─────┼──────┘
                          │
              ┌───────────┼───────────┐
              │           │           │
        ┌─────▼────┐ ┌───▼───┐ ┌────▼────┐
        │  Meta-   │ │Sandbox│ │Create   │
        │ Operator │ │ Apply │ │Operator │
        │(direct)  │ │cargo  │ │Pattern  │
        │          │ │check  │ │Detected │
        └──────────┘ └───┬───┘ └─────────┘
                          │
              ┌───────────▼───────────┐
              │     Validation        │
              │ (drives ≥ 50% base)  │
              └───────────┬───────────┘
                          │
              ┌───────────▼───────────┐
              │   Success Buffer      │
              │ (ring buffer, 200)    │
              └───────────────────────┘
```

---

## Key Features

### Homeostatic Drive System

Four biologically-inspired drives in [0,1]⁴ guide all action selection:

| Drive | Function | Update |
|---|---|---|
| **Curiosity** | Seeks prediction error | EMA of surprise, baseline ≥ 0.1 |
| **Mastery** | Seeks competence gain | Derivative of success rate |
| **Coherence** | Resists destabilizing changes | Inverse of code hash variance |
| **Novelty** | Seeks unexplored state space | KDE log-probability |

### World Model + Meta-Model

- **World model**: Linear regression (42→4) predicts drive deltas from action encodings
- **KDE uncertainty**: Gaussian kernel over last 200 action encodings
- **Meta-model**: SGD-trained linear network (35→172) predicts weight changes
- **Meta-curiosity**: `mc = (meta_error_ema / 0.1).clamp(0,1)` — spikes on operator creation

### Emergent Operator Discovery

`check_for_patterns()` runs every 10 ticks, scanning the success buffer for recurring `(op1, op2)` co-occurrence pairs. The most frequent novel pair is injected as a `create_operator` candidate with uncertainty=2.0 (high softmax bias). New operators are created via deterministic `synth_diff` — no LLM required.

### Autonomous Cleanup

At `MAX_OPERATORS = 32` capacity, the least-used non-protected operator is evicted. New operators start with **median usage** of existing non-protected operators, giving them competitive survival odds. 12 operators (8 primitives + 4 meta-operators) are permanently protected.

### Meta-Operators

Four built-in operators modify runtime parameters directly (bypassing sandbox):

| Operator | Parameter | Clamp |
|---|---|---|
| `set_num_candidates` | `N_ACTIONS` | — |
| `set_temp` | `TEMPERATURE` | [0.1, 10.0] |
| `set_meta_weight` | `META_UNC_WEIGHT` | [0.0, 5.0] |
| `set_kde_threshold` | `KDE_DIST_THRESHOLD` | [0.5, 20.0] |

### Sandboxed Self-Modification

- In-place `cargo check` (avoids binary lock, 60s timeout)
- Full `src/` backup before each modification
- Automatic restore on any failure
- Drive stability validation (≥50% of pre-apply baseline)
- `PROTOC` env passthrough

### Voice: LLM Self-Expression

When `MINDKERNEL_VOICE=1` and `energy > 0.3`, the system sends its complete drive state to an LLM endpoint and receives a natural-language introspection. Voice is purely observational — it has no influence on actions, serving as an interpretability channel.

---

## Operator Evolution

The system starts with 12 canonical primitives (set, copy, call, merge, split, guard, revert, duplicate + 4 meta-operators). Through pattern detection and operator creation, it builds compound operations with names like `duplicate_call_split`, `set_num_candidates_set_kde_threshold_call_split`, and more.

Operator naming follows the `op1_op2` concatenation pattern, producing a combinatorially expanding space of behavioral primitives.

---

## Test Results

### Stability

| Metric | Value |
|---|---|
| Total autonomous ticks | >1000 |
| Total operations | >500 |
| Zero-error test runs | 5/5 |
| Sandbox failures | 0 |

### Test Runs

| Test | Ticks | Duration | Operations | New Ops | Evicted | Errors | mc Range |
|---|---|---|---|---|---|---|---|
| testH | 134 | ~45 min | 134 | 2 | 0 | 0 | 0–0.739 |
| testI | 154 | ~1h15min | 308 | 4 | 0 | 0 | 0–1.0 |
| testJ | 417 | ~40 min | 91 | 6 | 3 | 0 | 0–1.0 |
| testK | 249 | ~30 min | 83 | 3 | 3 | 0 | 0–0.795 |
| testL | 411 | ~1h | 133 | 5 | 5 | 0 | 0–0.972 |

### Meta-Curiosity Dynamics

- Baseline: ~0.001
- Spike on operator creation: 0.5–1.0 (via `boost_meta_curiosity(0.05)`)
- Decay: geometric ×0.9 per tick (EMA α=0.1)
- Unique values across 411 ticks: 46
- Max observed: 1.0 (stacking on partially-decayed spike)

---

## Configuration

Runtime parameters (modifiable via meta-operators):

| Parameter | Default | Description |
|---|---|---|
| `N_ACTIONS` | 20 | Candidates per emergent step |
| `TEMPERATURE` | 1.0 | Softmax temperature |
| `META_UNC_WEIGHT` | 0.5 | Meta-uncertainty contribution |
| `KDE_DIST_THRESHOLD` | 4.0 | KDE distance threshold |

### Compile-Time Constants

| Constant | Value | Notes |
|---|---|---|
| `MAX_OPERATORS` | 32 | Auto-evicts at capacity |
| `INPUT_DIM` | 42 | Drives(4) + action(35) + code hash(3) |
| `NUM_WEIGHTS` | 172 | = INPUT_DIM × 4 + 4 |
| `VALIDATION_CYCLES` | 5 | 50ms each, 50% threshold |

---

## Research

For the full academic paper, see [RESEARCH.md](RESEARCH.md):

> *Self-Modifying AI with Emergent Operator Discovery and Homeostatic Drives*

The paper covers the complete architecture, mathematical formulation of the world model, meta-curiosity dynamics, pattern detection algorithms, and analysis of all test runs.

---

## License

MIT License — see [LICENSE](LICENSE).

---

<p align="center">
  <i>A self-modifying artificial life form, exploring its own computational substrate.</i>
</p>
