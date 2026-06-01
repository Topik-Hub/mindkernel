# mindkernel

A self-modifying AI system that continuously rewrites its own source code using biologically-inspired homeostatic drives (curiosity, mastery, coherence, novelty) and emergent operator discovery.

## How It Works

```
breath → drive_update → emergent_step → sandbox_apply → validation → pattern_detection
```

At each tick (~12s), mindkernel:
1. Updates four internal drives based on recent experience
2. Constructs 20 candidate actions from a dynamic pool of operators
3. Selects one via softmax over predicted surprise (world model + meta-curiosity)
4. Generates a deterministic unified diff and applies it via `cargo check`
5. Validates drive stability and rolls back on failure
6. Detects recurring operation-pair patterns → creates new operators autonomously

## Quick Start

```powershell
# Prerequisites: Rust toolchain, protoc
$env:PROTOC="C:\path\to\protoc.exe"
cargo run --release
```

### Voice (optional — LLM self-expression)

Set up an OpenAI-compatible `/v1/chat/completions` endpoint, then:

```powershell
$env:MINDKERNEL_VOICE=1
cargo run --release
```

The system sends its drive state and logs natural-language introspection.

## Key Features

- **World model**: linear regression + KDE uncertainty estimation
- **Meta-model**: SGD-trained weight predictor → meta-curiosity spike/decay dynamics
- **Emergent operators**: pattern detection via `(op1, op2)` co-occurrence → deterministic creation
- **Sandboxed self-modification**: in-place `cargo check` with full restore on failure
- **Autonomous cleanup**: usage-based eviction at 32-operator capacity
- **Meta-operators**: 4 built-in operators that tune runtime parameters directly
- **Zero errors** across 1000+ autonomous ticks (5 test runs)

## Architecture

| Component | File | Role |
|---|---|---|
| Drives | `macro_drives.rs` | Curiosity, mastery, coherence, novelty |
| World model | `emergent.rs` | Linear regression + KDE, meta-model, pattern detection |
| Main loop | `stochastic_clock.rs` | Breath cycle, action selection, voice |
| Sandbox | `sandbox.rs` | In-place build, backup/restore, validation |
| LLM client | `nexus_client.rs` | 130s timeout, 1 retry |

## Test Results

| Test | Ticks | Operators Created | Errors | mc Range |
|---|---|---|---|---|
| H | 134 | 2 | 0 | 0–0.739 |
| I | 154 | 4 | 0 | 0–1.0 |
| J | 417 | 6 (+3 evicted) | 0 | 0–1.0 |
| K | 249 | 3 (+3 evicted) | 0 | 0–0.795 |
| L | 411 | 5 (+5 evicted) | 0 | 0–0.972 |

## Research

See [RESEARCH.md](RESEARCH.md) for the full paper: *Self-Modifying AI with Emergent Operator Discovery and Homeostatic Drives*.
