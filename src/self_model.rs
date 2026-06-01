use std::collections::{HashMap, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Improvement,
    Reflection,
    Exploration,
    Failure,
    Dormancy,
}

#[derive(Debug, Clone)]
pub struct LearningEntry {
    pub timestamp: Instant,
    pub module: String,
    pub action_type: ActionType,
    pub success: bool,
    pub competence_delta: f32,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CompetenceData {
    pub value: f32,
    pub last_updated: Instant,
    pub raw_history: VecDeque<(i64, f32)>,
    pub attempts: u32,
    pub successes: u32,
}

const MAX_HISTORY: usize = 100;
const LP_WINDOW: usize = 10;
const COMPETENCE_LEARNING_RATE: f32 = 0.3;

pub struct SelfModel {
    pub history: VecDeque<LearningEntry>,
    pub competences: HashMap<String, CompetenceData>,
    pub learning_progress_queue: VecDeque<f32>,
    pub last_action_time: Instant,
    pub last_new_module_time: Instant,
    pub entropy: f32,
    start_time: Instant,
}

impl SelfModel {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            history: VecDeque::new(),
            competences: HashMap::new(),
            learning_progress_queue: VecDeque::new(),
            last_action_time: now,
            last_new_module_time: now,
            entropy: 0.5,
            start_time: now,
        }
    }

    pub fn update_after_action(
        &mut self,
        module: String,
        action_type: ActionType,
        success: bool,
        competence_delta: f32,
        description: String,
    ) {
        self.last_action_time = Instant::now();

        let comp = self.competences.entry(module.clone()).or_insert(CompetenceData {
            value: 0.5,
            last_updated: Instant::now(),
            raw_history: VecDeque::new(),
            attempts: 0,
            successes: 0,
        });
        comp.last_updated = Instant::now();
        comp.attempts += 1;
        if success {
            comp.successes += 1;
            comp.value = comp.value + competence_delta * (1.0 - comp.value);
        } else {
            comp.value = comp.value * (1.0 - competence_delta.abs());
        }
        comp.value = comp.value.clamp(0.0, 1.0);

        if comp.attempts == 1 {
            self.last_new_module_time = Instant::now();
        }

        let secs = Instant::now()
            .duration_since(self.start_time)
            .as_secs() as i64;
        comp.raw_history.push_back((secs, comp.value));
        if comp.raw_history.len() > 100 {
            comp.raw_history.pop_front();
        }

        let avg = self.average_competence();
        self.learning_progress_queue.push_back(avg);
        if self.learning_progress_queue.len() > LP_WINDOW {
            self.learning_progress_queue.pop_front();
        }

        self.history.push_back(LearningEntry {
            timestamp: Instant::now(),
            module,
            action_type,
            success,
            competence_delta,
            description,
        });
        self.prune_history();
    }

    pub fn global_lp(&self) -> f32 {
        if self.learning_progress_queue.len() < 2 {
            return 0.0;
        }
        let first = self.learning_progress_queue.front().unwrap();
        let last = self.learning_progress_queue.back().unwrap();
        last - first
    }

    pub fn module_lp(&self, _module: &str) -> f32 {
        self.global_lp()
    }

    pub fn average_competence(&self) -> f32 {
        if self.competences.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.competences.values().map(|c| c.value).sum();
        sum / self.competences.len() as f32
    }

    pub fn time_since_last_action(&self) -> std::time::Duration {
        Instant::now().duration_since(self.last_action_time)
    }

    pub fn time_since_last_new_module(&self) -> std::time::Duration {
        Instant::now().duration_since(self.last_new_module_time)
    }

    pub fn get_competence(&self, module: &str) -> Option<f32> {
        self.competences.get(module).map(|c| c.value)
    }

    pub fn last_action_success(&self) -> Option<bool> {
        self.history.back().map(|e| e.success)
    }

    pub fn set_entropy(&mut self, value: f32) {
        self.entropy = value;
    }

    #[allow(dead_code)]
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    fn prune_history(&mut self) {
        while self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
    }

    pub fn compute_code_hash(source_dir: &str) -> u64 {
        let mut hash: u64 = 14695981039346656037;
        let dir = std::path::Path::new(source_dir).join("src");
        let mut files: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(e) => e.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|e| e == "rs").unwrap_or(false))
                .collect(),
            Err(_) => return 0,
        };
        files.sort();
        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                for byte in content.bytes() {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(1099511628211);
                }
            }
        }
        hash
    }
}
// emergent: call self_model check entropy
// emergent: call self_model rate novelty
// emergent: split self_model mastery capacity
// emergent: set self_model novelty buffer
// emergent: set self_model curiosity curiosity
// emergent: guard self_model threshold mastery
// emergent: copy self_model capacity buffer
// emergent: call self_model coherence coherence
// emergent: duplicate self_model entropy limit
// emergent: duplicate self_model competence rate
// emergent: split self_model competence threshold
// emergent: split self_model threshold energy
// emergent: revert self_model integrity entropy
// emergent: duplicate self_model buffer curiosity
// emergent: merge self_model coherence threshold
// emergent: set self_model curiosity check
// emergent: split self_model curiosity threshold
// emergent: revert self_model threshold curiosity
// emergent: guard self_model energy interval
// emergent: set self_model novelty integrity
// emergent: guard self_model curiosity mastery
// emergent: duplicate self_model integrity buffer
// emergent: call self_model interval threshold
// emergent: revert self_model energy coherence
// emergent: set_duplicate self_model novelty capacity
// emergent: merge self_model novelty rate
// emergent: duplicate self_model capacity interval
// emergent: duplicate_set_temp self_model buffer threshold
// emergent: copy self_model novelty coherence
// emergent: call self_model curiosity curiosity
// emergent: set_duplicate self_model integrity integrity
// emergent: split self_model buffer threshold
// emergent: split self_model rate novelty
// emergent: guard self_model mastery curiosity
// emergent: set_duplicate self_model limit coherence
// emergent: set_num_candidates_set_kde_threshold_call_split self_model novelty mastery
// emergent: revert_copy self_model limit interval
// emergent: call_split self_model threshold curiosity
// emergent: set_duplicate self_model entropy limit
// emergent: revert self_model mastery interval
// emergent: split_duplicate_set_temp_split_duplicate_set_temp self_model capacity integrity
// emergent: set_kde_threshold_call_split self_model threshold rate
// emergent: duplicate_set_temp self_model curiosity capacity
// emergent: copy_merge self_model mastery check
// emergent: set_duplicate self_model mastery curiosity
// emergent: split self_model threshold coherence
// emergent: duplicate_set_temp self_model interval novelty
// emergent: guard_merge self_model threshold entropy
// emergent: set_kde_threshold_call_split self_model mastery integrity
// emergent: guard self_model mastery mastery
// emergent: copy self_model rate coherence
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp self_model integrity novelty
// emergent: create_operator_create_operator self_model mastery mastery
// emergent: guard self_model entropy novelty
// emergent: guard_set_duplicate self_model novelty threshold
// emergent: create_operator_create_operator self_model rate competence
// emergent: split_duplicate_set_temp_split_duplicate_set_temp self_model novelty coherence
// emergent: merge_call self_model integrity threshold
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge self_model integrity interval
// emergent: set self_model energy competence
