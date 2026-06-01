use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex, RwLock};

use crate::self_model::SelfModel;

const BASE_OPERATIONS: &[&str] = &[
    "set", "copy", "call", "merge", "split",
    "guard", "revert", "duplicate",
    "revert_copy",
    "call_split",
    "set_num_candidates",
    "set_temp",
    "set_meta_weight",
    "set_kde_threshold",
    "set_duplicate",
    "duplicate_set_temp",
    "merge_call",
    "set_kde_threshold_call_split",
    "guard_merge",
    "duplicate_call_split",
    "set_num_candidates_set_kde_threshold_call_split",
    "split_duplicate_set_temp",
    "set_temp_merge_call",
    "copy_merge",
    "split_duplicate_set_temp_split_duplicate_set_temp",
    "call_set_temp",
    "split_duplicate_set_temp_split_duplicate_set_temp_copy_merge",
    "set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp",
    "set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge",
    "split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge",
    "guard_set_duplicate",
    "copy_set_kde_threshold",
    "set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp",
    "call_copy",
    "split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge",
    "create_operator_create_operator",
    "revert_copy_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge",
    "copy_set_kde_threshold_revert_copy",
    "split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_call",
    "call_set_temp_split",
    "merge_call_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge",
    "copy_merge_copy",
    "set_temp_merge_call_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp",
    "set_kde_threshold_copy_merge",
];

static OPERATIONS_VEC: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| {
    Mutex::new(BASE_OPERATIONS.iter().map(|&s| s.to_string()).collect())
});

const MAX_OPERATORS: usize = 32;

pub static N_ACTIONS: LazyLock<RwLock<usize>> = LazyLock::new(|| RwLock::new(20));
pub static TEMPERATURE: LazyLock<RwLock<f32>> = LazyLock::new(|| RwLock::new(1.0));
pub static META_UNC_WEIGHT: LazyLock<RwLock<f32>> = LazyLock::new(|| RwLock::new(0.5));
pub static KDE_DIST_THRESHOLD: LazyLock<RwLock<f32>> = LazyLock::new(|| RwLock::new(4.0));
const ACTION_ENC_LEN: usize = MAX_OPERATORS + 3;

static OPERATION_USAGE: LazyLock<Mutex<std::collections::HashMap<String, usize>>> = LazyLock::new(|| {
    Mutex::new(std::collections::HashMap::new())
});

fn protected_op(name: &str) -> bool {
    matches!(name, "set" | "copy" | "call" | "merge" | "split" | "guard" | "revert" | "duplicate"
        | "set_num_candidates" | "set_temp" | "set_meta_weight" | "set_kde_threshold")
}

pub fn add_operation(name: &str) {
    if let Ok(mut ops) = OPERATIONS_VEC.lock() {
        if !ops.iter().any(|o| o == name) {
            if ops.len() >= MAX_OPERATORS {
                if let Ok(mut usage) = OPERATION_USAGE.lock() {
                    let victim = ops.iter()
                        .filter(|o| !protected_op(o))
                        .min_by_key(|o| usage.get(*o).copied().unwrap_or(0))
                        .cloned();
                    if let Some(v) = victim {
                        usage.remove(&v);
                        ops.retain(|o| o != &v);
                        tracing::info!("[operations] evicted '{}' to make room for '{}'", v, name);
                    } else {
                        tracing::info!("[operations] all ops protected, cannot add '{}'", name);
                        return;
                    }
                }
            }
            ops.push(name.to_string());
            if let Ok(mut usage) = OPERATION_USAGE.lock() {
                let existing: Vec<usize> = ops.iter()
                    .filter(|o| !protected_op(o))
                    .filter_map(|o| usage.get(o.as_str()).copied())
                    .collect();
                let initial = if existing.is_empty() {
                    1
                } else {
                    let mut sorted = existing;
                    sorted.sort();
                    sorted[sorted.len() / 2]
                };
                usage.entry(name.to_string()).or_insert(initial);
            }
            tracing::info!("[operations] added '{}' (total {})", name, ops.len());
        }
    }
}

fn record_usage(name: &str) {
    if let Ok(mut usage) = OPERATION_USAGE.lock() {
        *usage.entry(name.to_string()).or_insert(0) += 1;
    }
}

const NUM_DRIVES: usize = 4;
const CODE_HASH_LEN: usize = 3;
const INPUT_DIM: usize = NUM_DRIVES + ACTION_ENC_LEN + CODE_HASH_LEN;
const NUM_WEIGHTS: usize = INPUT_DIM * NUM_DRIVES + NUM_DRIVES;
const META_INPUT_DIM: usize = ACTION_ENC_LEN;
const META_OUTPUT_DIM: usize = NUM_WEIGHTS;
const META_LR: f32 = 0.005;
const META_SGD_STEPS: usize = 10;
const META_MAX_HISTORY: usize = 100;
const META_EMA_ALPHA: f32 = 0.1;

#[derive(Debug, Clone)]
pub struct Action {
    pub operation: String,
    pub target: String,
    pub param1: String,
    pub param2: String,
}

impl Action {
    pub fn encode(&self) -> [f32; ACTION_ENC_LEN] {
        let mut enc = [0.0; ACTION_ENC_LEN];
        if let Ok(ops) = OPERATIONS_VEC.lock() {
            if let Some(idx) = ops.iter().position(|o| o == &self.operation) {
                if idx < MAX_OPERATORS {
                    enc[idx] = 1.0;
                }
            }
        }
        enc[MAX_OPERATORS] = string_hash(&self.target);
        enc[MAX_OPERATORS + 1] = string_hash(&self.param1);
        enc[MAX_OPERATORS + 2] = string_hash(&self.param2);
        enc
    }
}

fn string_hash(s: &str) -> f32 {
    let mut hash: u32 = 2166136261;
    for b in s.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash as f32 / u32::MAX as f32
}

struct Transition {
    drives_t: [f32; NUM_DRIVES],
    action_enc: [f32; ACTION_ENC_LEN],
    drives_t1: [f32; NUM_DRIVES],
    code_hash: u64,
}

struct MetaTransition {
    action_enc: [f32; ACTION_ENC_LEN],
    weight_delta: [f32; NUM_WEIGHTS],
}

pub struct SuccessEntry {
    pub action: Action,
    pub code_hash: u64,
    pub reward: f32,
}

fn code_hash_to_vec(h: u64) -> [f32; CODE_HASH_LEN] {
    [
        (h & 0x1FFFFF) as f32 / 0x1FFFFF as f32,
        ((h >> 21) & 0x1FFFFF) as f32 / 0x1FFFFF as f32,
        ((h >> 42) as f32) / ((1u64 << 22) as f32),
    ]
}

pub struct WorldModel {
    weights: Vec<[f32; INPUT_DIM]>,
    bias: [f32; NUM_DRIVES],
    transitions: HashMap<u64, VecDeque<Transition>>,
    max_per_bucket: usize,
    train_counter: u64,
    meta_weights: Vec<[f32; META_INPUT_DIM]>,
    meta_bias: Vec<f32>,
    meta_history: VecDeque<MetaTransition>,
    meta_counter: usize,
    meta_error_ema: f32,
    last_meta_mse: f32,
    delta_norms: VecDeque<f32>,
    success_buffer: VecDeque<SuccessEntry>,
    pattern_counter: usize,
    pub last_discovered_op: Option<(String, String, String)>,
}

impl WorldModel {
    pub fn new() -> Self {
        Self {
            weights: vec![[0.0; INPUT_DIM]; NUM_DRIVES],
            bias: [0.5; NUM_DRIVES],
            transitions: HashMap::new(),
            max_per_bucket: 200,
            train_counter: 0,
            meta_weights: vec![[0.0; META_INPUT_DIM]; META_OUTPUT_DIM],
            meta_bias: vec![0.0; META_OUTPUT_DIM],
            meta_history: VecDeque::new(),
            meta_counter: 0,
            meta_error_ema: 0.0,
            last_meta_mse: 0.0,
            delta_norms: VecDeque::new(),
            success_buffer: VecDeque::new(),
            pattern_counter: 0,
            last_discovered_op: None,
        }
    }

    pub fn predict(&self, drives: &[f32; NUM_DRIVES], action: &Action, code_hash: u64) -> (Vec<f32>, f32) {
        let action_enc = action.encode();
        let code_vec = code_hash_to_vec(code_hash);
        let input = Self::make_input(drives, &action_enc, &code_vec);

        let mut prediction = [0.0; NUM_DRIVES];
        for i in 0..NUM_DRIVES {
            prediction[i] = self.bias[i];
            for j in 0..INPUT_DIM {
                prediction[i] += self.weights[i][j] * input[j];
            }
            prediction[i] = prediction[i].clamp(0.0, 1.0);
        }

        let (base_unc, n_similar) = self.estimate_uncertainty(&action_enc);
        let (meta_unc, _meta_n) = self.meta_kde_uncertainty(&action_enc);
        let bonus = self.survival_bonus(code_hash, &action_enc);
        let mw = *META_UNC_WEIGHT.read().unwrap_or_else(|e| e.into_inner());
        let uncertainty = if n_similar < 3 {
            1.0 + bonus
        } else {
            ((0.1 + base_unc).min(1.0) + bonus).min(2.0)
        } + (mw * meta_unc).min(0.5);

        (prediction.to_vec(), uncertainty)
    }

    fn estimate_uncertainty(&self, action_enc: &[f32; ACTION_ENC_LEN]) -> (f32, usize) {
        let thr = *KDE_DIST_THRESHOLD.read().unwrap_or_else(|e| e.into_inner());
        let mut outcomes: Vec<[f32; NUM_DRIVES]> = Vec::new();
        for t in self.transitions.values().flat_map(|v| v.iter()) {
            let dist: f32 = t.action_enc.iter()
                .zip(action_enc.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            if dist < thr {
                outcomes.push(t.drives_t1);
            }
        }
        let n = outcomes.len();
        if n < 2 {
            return (1.0, n);
        }
        let nf = n as f32;
        let mut mean = [0.0; NUM_DRIVES];
        for o in &outcomes {
            for i in 0..NUM_DRIVES {
                mean[i] += o[i];
            }
        }
        for i in 0..NUM_DRIVES {
            mean[i] /= nf;
        }
        let var: f32 = outcomes.iter()
            .flat_map(|o| o.iter().zip(mean.iter()).map(|(a, m)| (a - m).powi(2)))
            .sum::<f32>() / (nf * NUM_DRIVES as f32);
        (var, n)
    }

    fn survival_bonus(&self, code_hash: u64, action_enc: &[f32; ACTION_ENC_LEN]) -> f32 {
        let thr = *KDE_DIST_THRESHOLD.read().unwrap_or_else(|e| e.into_inner());
        let bucket = match self.transitions.get(&code_hash) {
            Some(b) => b,
            None => return 0.0,
        };
        let (matching, successful) = bucket.iter().fold((0, 0), |(m, s), t| {
            let dist: f32 = t.action_enc.iter()
                .zip(action_enc.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            if dist < thr {
                let survived = t.drives_t1.iter().sum::<f32>() > t.drives_t.iter().sum::<f32>();
                (m + 1, s + survived as i32)
            } else {
                (m, s)
            }
        });
        if matching < 2 { 0.0 } else { 0.3 * successful as f32 / matching as f32 }
    }

    fn flatten_weights(&self) -> [f32; NUM_WEIGHTS] {
        let mut flat = [0.0; NUM_WEIGHTS];
        let mut idx = 0;
        for i in 0..NUM_DRIVES {
            for j in 0..INPUT_DIM {
                flat[idx] = self.weights[i][j];
                idx += 1;
            }
        }
        for i in 0..NUM_DRIVES {
            flat[idx] = self.bias[i];
            idx += 1;
        }
        flat
    }

    fn predict_meta_delta(&self, action_enc: &[f32; ACTION_ENC_LEN]) -> [f32; NUM_WEIGHTS] {
        let mut delta = [0.0; NUM_WEIGHTS];
        for i in 0..META_OUTPUT_DIM {
            delta[i] = self.meta_bias[i]
                + action_enc.iter()
                    .zip(self.meta_weights[i].iter())
                    .map(|(x, w)| x * w)
                    .sum::<f32>();
        }
        delta
    }

    fn meta_kde_uncertainty(&self, action_enc: &[f32; ACTION_ENC_LEN]) -> (f32, usize) {
        let thr = *KDE_DIST_THRESHOLD.read().unwrap_or_else(|e| e.into_inner());
        let mut deltas: Vec<[f32; NUM_WEIGHTS]> = Vec::new();
        for mt in &self.meta_history {
            let dist: f32 = mt.action_enc.iter()
                .zip(action_enc.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            if dist < thr {
                deltas.push(mt.weight_delta);
            }
        }
        let n = deltas.len();
        if n < 2 {
            return (1.0, n);
        }
        let nf = n as f32;
        let mut mean = [0.0; NUM_WEIGHTS];
        for d in &deltas {
            for i in 0..NUM_WEIGHTS {
                mean[i] += d[i];
            }
        }
        for i in 0..NUM_WEIGHTS {
            mean[i] /= nf;
        }
        let var: f32 = deltas.iter()
            .flat_map(|d| d.iter().zip(mean.iter()).map(|(a, m)| (a - m).powi(2)))
            .sum::<f32>() / (nf * NUM_WEIGHTS as f32);
        (var, n)
    }

    pub fn meta_curiosity(&self) -> f32 {
        (self.meta_error_ema / 0.1).clamp(0.0, 1.0)
    }

    pub fn meta_mse(&self) -> f32 {
        self.meta_error_ema
    }

    pub fn boost_meta_curiosity(&mut self, amount: f32) {
        self.meta_error_ema = (self.meta_error_ema + amount).min(1.0);
    }

    pub fn last_meta_error(&self) -> f32 {
        self.last_meta_mse
    }

    pub fn recent_weight_change(&self) -> f32 {
        let n = self.delta_norms.len();
        if n == 0 { return 0.0; }
        self.delta_norms.iter().sum::<f32>() / n as f32
    }

    pub fn push_success(&mut self, action: Action, code_hash: u64, reward: f32) {
        self.success_buffer.push_back(SuccessEntry { action, code_hash, reward });
        if self.success_buffer.len() > 200 {
            self.success_buffer.pop_front();
        }
    }

    pub fn check_for_patterns(&mut self) {
        self.pattern_counter += 1;
        if self.pattern_counter < 10 { return; }
        self.pattern_counter = 0;
        let entries: Vec<&SuccessEntry> = self.success_buffer.iter().collect();
        if entries.len() < 5 { return; }
        let mut seq_counts: HashMap<(&str, &str), usize> = HashMap::new();
        for window in entries.windows(2) {
            let a1 = &window[0].action;
            let a2 = &window[1].action;
            let key = (a1.operation.as_str(), a2.operation.as_str());
            *seq_counts.entry(key).or_insert(0) += 1;
        }
        let mut candidates: Vec<((String, String, String), usize)> = seq_counts.iter()
            .filter(|&(_, c)| *c >= 1)
            .map(|((op1, op2), &c)| {
                let name = format!("{}_{}", op1, op2);
                ((name, op1.to_string(), op2.to_string()), c)
            })
            .collect();
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        let ops_vec = OPERATIONS_VEC.lock().unwrap();
        for ((name, op1, op2), _) in candidates {
            if self.last_discovered_op.as_ref().map(|(n,_,_)| n.as_str()) != Some(&name)
                && !ops_vec.iter().any(|o| o == &name)
            {
                drop(ops_vec);
                self.last_discovered_op = Some((name, op1, op2));
                return;
            }
        }
        drop(ops_vec);
    }

    fn train_meta_once(&mut self, action_enc: &[f32; ACTION_ENC_LEN], target_delta: &[f32; NUM_WEIGHTS]) {
        self.meta_counter += 1;
        for _ in 0..META_SGD_STEPS {
            for i in 0..META_OUTPUT_DIM {
                let pred = self.meta_bias[i]
                    + action_enc.iter()
                        .zip(self.meta_weights[i].iter())
                        .map(|(x, w)| x * w)
                        .sum::<f32>();
                let error = pred - target_delta[i];
                for j in 0..META_INPUT_DIM {
                    self.meta_weights[i][j] -= META_LR * error * action_enc[j];
                }
                self.meta_bias[i] -= META_LR * error;
            }
        }
    }

    pub fn update(&mut self, drives_t: &[f32; NUM_DRIVES], action: &Action, drives_t1: &[f32; NUM_DRIVES], code_hash: u64) {
        let bucket = self.transitions.entry(code_hash).or_default();
        let action_enc = action.encode();
        bucket.push_back(Transition {
            drives_t: *drives_t,
            action_enc,
            drives_t1: *drives_t1,
            code_hash,
        });
        if bucket.len() > self.max_per_bucket {
            bucket.pop_front();
        }
        self.train_counter += 1;
        if self.train_counter % 3 == 0 {
            let flat_before = self.flatten_weights();
            let pred_delta = self.predict_meta_delta(&action_enc);
            self.fit_linear();
            let flat_after = self.flatten_weights();
            let mut delta = [0.0; NUM_WEIGHTS];
            let mut sq_err = 0.0;
            for i in 0..NUM_WEIGHTS {
                delta[i] = flat_after[i] - flat_before[i];
                let diff = pred_delta[i] - delta[i];
                sq_err += diff * diff;
            }
            let mse = sq_err / NUM_WEIGHTS as f32;
            self.meta_error_ema += META_EMA_ALPHA * (mse - self.meta_error_ema);
            self.last_meta_mse = mse;
            let delta_rms = (delta.iter().map(|d| d * d).sum::<f32>() / NUM_WEIGHTS as f32).sqrt();
            self.delta_norms.push_back(delta_rms);
            if self.delta_norms.len() > 10 {
                self.delta_norms.pop_front();
            }
            self.train_meta_once(&action_enc, &delta);
            self.meta_history.push_back(MetaTransition { action_enc, weight_delta: delta });
            if self.meta_history.len() > META_MAX_HISTORY {
                self.meta_history.pop_front();
            }
        }
    }

    fn fit_linear(&mut self) {
        let xs: Vec<[f32; INPUT_DIM]> = self.transitions.values()
            .flat_map(|v| v.iter())
            .map(|t| {
                let code_vec = code_hash_to_vec(t.code_hash);
                Self::make_input(&t.drives_t, &t.action_enc, &code_vec)
            })
            .collect();
        let ys: Vec<[f32; NUM_DRIVES]> = self.transitions.values()
            .flat_map(|v| v.iter())
            .map(|t| t.drives_t1)
            .collect();
        let n = xs.len();
        if n < 3 {
            return;
        }
        let nf = n as f32;
        let lr = 0.01;
        for _ in 0..30 {
            let mut grad_w = vec![[0.0; INPUT_DIM]; NUM_DRIVES];
            let mut grad_b = [0.0; NUM_DRIVES];
            for k in 0..n {
                let input = &xs[k];
                let target = &ys[k];
                for i in 0..NUM_DRIVES {
                    let pred = self.bias[i]
                        + input.iter()
                            .zip(self.weights[i].iter())
                            .map(|(x, w)| x * w)
                            .sum::<f32>();
                    let error = pred - target[i];
                    grad_b[i] += error;
                    for j in 0..INPUT_DIM {
                        grad_w[i][j] += error * input[j];
                    }
                }
            }
            for i in 0..NUM_DRIVES {
                self.bias[i] -= lr * grad_b[i] / nf;
                for j in 0..INPUT_DIM {
                    self.weights[i][j] -= lr * grad_w[i][j] / nf;
                }
            }
        }
    }

    fn make_input(a: &[f32; NUM_DRIVES], b: &[f32; ACTION_ENC_LEN], c: &[f32; CODE_HASH_LEN]) -> [f32; INPUT_DIM] {
        let mut out = [0.0; INPUT_DIM];
        out[..NUM_DRIVES].copy_from_slice(a);
        out[NUM_DRIVES..NUM_DRIVES + ACTION_ENC_LEN].copy_from_slice(b);
        out[NUM_DRIVES + ACTION_ENC_LEN..].copy_from_slice(c);
        out
    }
}

pub fn source_modules() -> Vec<String> {
    let mut modules = Vec::new();
    if let Ok(entries) = std::fs::read_dir("src") {
        for entry in entries.flatten() {
            if entry.path().extension().map(|e| e == "rs").unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    modules.push(name.trim_end_matches(".rs").to_string());
                }
            }
        }
    }
    if modules.is_empty() {
        modules.push("main".to_string());
    }
    modules
}

fn random_op(rng: &mut u64) -> String {
    if let Ok(ops) = OPERATIONS_VEC.lock() {
        let len = ops.len();
        if len > 0 {
            let idx = (rng_advance(rng) % len as u64) as usize;
            let op = ops[idx].clone();
            record_usage(&op);
            return op;
        }
    }
    "set".to_string()
}

fn random_module(rng: &mut u64) -> String {
    let modules = source_modules();
    let idx = (rng_advance(rng) % modules.len() as u64) as usize;
    modules[idx].clone()
}

fn random_param(rng: &mut u64) -> String {
    let pool = [
        "energy", "integrity", "novelty", "coherence", "competence",
        "entropy", "curiosity", "mastery", "threshold", "buffer",
        "rate", "limit", "interval", "capacity", "check",
    ];
    let idx = (rng_advance(rng) % pool.len() as u64) as usize;
    pool[idx].to_string()
}

fn rng_advance(seed: &mut u64) -> u64 {
    *seed ^= *seed >> 12;
    *seed ^= *seed << 25;
    *seed ^= *seed >> 27;
    *seed
}

fn next_f32(seed: &mut u64) -> f32 {
    rng_advance(seed);
    (*seed as f32) * 5.42101e-20
}

pub struct EmergentOutcome {
    pub action: Action,
    pub predicted_drives: Vec<f32>,
    pub uncertainty: f32,
}

pub fn emergent_step(
    drives: &[f32; NUM_DRIVES],
    _model: &SelfModel,
    world_model: &WorldModel,
    rng: &mut u64,
    code_hash: u64,
) -> EmergentOutcome {
    let n_actions = *N_ACTIONS.read().unwrap_or_else(|e| e.into_inner());
    let temperature = *TEMPERATURE.read().unwrap_or_else(|e| e.into_inner());

    let mut actions: Vec<Action> = Vec::with_capacity(n_actions);
    let mut predictions: Vec<Vec<f32>> = Vec::with_capacity(n_actions);
    let mut uncertainties: Vec<f32> = Vec::with_capacity(n_actions);

    for _ in 0..n_actions {
        let action = Action {
            operation: random_op(rng),
            target: random_module(rng),
            param1: random_param(rng),
            param2: random_param(rng),
        };
        if action.operation == "delete" || action.target == "main" {
            continue;
        }
        let (pred, unc) = world_model.predict(drives, &action, code_hash);
        actions.push(action);
        predictions.push(pred);
        uncertainties.push(unc);
    }

    if let Some((ref name, ref op1, ref op2)) = world_model.last_discovered_op {
        let meta_action = Action {
            operation: "create_operator".to_string(),
            target: name.clone(),
            param1: op1.clone(),
            param2: op2.clone(),
        };
        let (pred, _unc) = world_model.predict(drives, &meta_action, code_hash);
        actions.push(meta_action);
        predictions.push(pred);
        uncertainties.push(2.0);
    }

    let max_unc = uncertainties.iter().cloned().fold(0.0f32, f32::max);
    let exp_scores: Vec<f32> = uncertainties
        .iter()
        .map(|u| ((u - max_unc) / temperature).exp())
        .collect();
    let sum_exp: f32 = exp_scores.iter().sum();

    let r = next_f32(rng);
    let mut cum = 0.0;
    let mut chosen = 0;
    for (i, p) in exp_scores.iter().enumerate() {
        cum += p / sum_exp;
        if r <= cum {
            chosen = i;
            break;
        }
    }

    EmergentOutcome {
        action: actions.swap_remove(chosen),
        predicted_drives: predictions.swap_remove(chosen),
        uncertainty: uncertainties.swap_remove(chosen),
    }
}

pub fn is_meta_operator(op: &str) -> bool {
    matches!(op, "set_num_candidates" | "set_temp" | "set_meta_weight" | "set_kde_threshold")
}

pub fn handle_meta_operator(action: &Action) -> bool {
    if !is_meta_operator(&action.operation) {
        return false;
    }
    let seed = string_hash(&format!("{}{}{}", action.target, action.param1, action.param2));
    match action.operation.as_str() {
        "set_num_candidates" => {
            let val = (seed * 49.0 + 1.0) as usize;
            if let Ok(mut n) = N_ACTIONS.write() {
                *n = val;
            }
            tracing::info!("[meta] set_num_candidates = {}", val);
            true
        }
        "set_temp" => {
            let val = (seed * 4.9 + 0.1).clamp(0.1, 10.0);
            if let Ok(mut t) = TEMPERATURE.write() {
                *t = val;
            }
            tracing::info!("[meta] set_temp = {:.3}", val);
            true
        }
        "set_meta_weight" => {
            let val = (seed * 5.0).clamp(0.0, 5.0);
            if let Ok(mut w) = META_UNC_WEIGHT.write() {
                *w = val;
            }
            tracing::info!("[meta] set_meta_weight = {:.3}", val);
            true
        }
        "set_kde_threshold" => {
            let val = (seed * 19.5 + 0.5).clamp(0.5, 20.0);
            if let Ok(mut t) = KDE_DIST_THRESHOLD.write() {
                *t = val;
            }
            tracing::info!("[meta] set_kde_threshold = {:.3}", val);
            true
        }
        _ => false,
    }
}
// emergent: merge emergent coherence coherence
// emergent: merge emergent limit coherence
// emergent: call emergent limit interval
// emergent: call emergent entropy curiosity
// emergent: merge emergent novelty capacity
// emergent: split emergent rate integrity
// emergent: set emergent buffer coherence
// emergent: merge emergent competence curiosity
// emergent: revert emergent buffer buffer
// emergent: duplicate emergent curiosity entropy
// emergent: revert emergent novelty rate
// emergent: guard emergent check entropy
// emergent: call emergent novelty novelty
// emergent: split emergent integrity novelty
// emergent: create_operator_create_operator emergent energy energy
// emergent: split_merge emergent check integrity
// emergent: split_merge emergent buffer check
// emergent: merge emergent curiosity integrity
// emergent: split emergent limit threshold
// emergent: revert emergent buffer interval
// emergent: duplicate_set_temp emergent buffer competence
// emergent: call emergent novelty mastery
// emergent: call_split emergent energy limit
// emergent: duplicate emergent limit buffer
// emergent: call emergent capacity buffer
// emergent: merge emergent check mastery
// emergent: revert_copy emergent limit threshold
// emergent: guard emergent competence energy
// emergent: guard emergent competence threshold
// emergent: split emergent check competence
// emergent: set_kde_threshold_call_split emergent mastery check
// emergent: call_split emergent interval curiosity
// emergent: revert_copy emergent integrity novelty
// emergent: guard_merge emergent rate competence
// emergent: call_split emergent check coherence
// emergent: merge_call emergent limit rate
// emergent: split_duplicate_set_temp emergent check check
// emergent: guard emergent buffer check
// emergent: revert emergent mastery novelty
// emergent: split_duplicate_set_temp_split_duplicate_set_temp emergent curiosity entropy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge emergent entropy buffer
// emergent: revert_copy emergent energy threshold
// emergent: split_duplicate_set_temp_split_duplicate_set_temp emergent rate entropy
// emergent: call emergent capacity rate
// emergent: split_duplicate_set_temp emergent mastery buffer
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp emergent competence capacity
// emergent: guard emergent capacity mastery
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge emergent integrity integrity
// emergent: set_num_candidates_set_kde_threshold_call_split emergent entropy check
// emergent: call emergent competence energy
// emergent: duplicate_set_temp emergent rate integrity
// emergent: call_split emergent interval check
// emergent: call emergent limit interval
// emergent: call_split emergent mastery novelty
// emergent: set_temp_merge_call emergent novelty curiosity
// emergent: revert emergent capacity buffer
// emergent: copy_merge emergent buffer rate
// emergent: merge emergent energy novelty
// emergent: call emergent curiosity novelty
