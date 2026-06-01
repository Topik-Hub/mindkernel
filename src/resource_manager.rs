use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct ResourceManager {
    start_time: Instant,
    energy_pct: Arc<AtomicI64>,
    user_active: Arc<AtomicBool>,
    time_budget_sec: f64,
    blocked_ideas: Arc<tokio::sync::RwLock<Vec<String>>>,
}

impl ResourceManager {
    pub fn new(time_budget_sec: f64) -> Self {
        Self {
            start_time: Instant::now(),
            energy_pct: Arc::new(AtomicI64::new(10000)),
            user_active: Arc::new(AtomicBool::new(false)),
            time_budget_sec,
            blocked_ideas: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub fn set_energy(&self, pct: f64) {
        self.energy_pct.store((pct * 10000.0) as i64, Ordering::Relaxed);
    }

    pub fn get_energy(&self) -> f64 {
        self.energy_pct.load(Ordering::Relaxed) as f64 / 10000.0
    }

    pub fn set_user_active(&self, active: bool) {
        self.user_active.store(active, Ordering::Relaxed);
    }

    pub fn time_budget_sec(&self) -> f64 {
        self.time_budget_sec
    }

    pub fn uptime_sec(&self) -> i64 {
        self.start_time.elapsed().as_secs() as i64
    }

    pub fn compute_suggested_interval(&self) -> i32 {
        let energy = self.get_energy();
        if energy > 0.8 {
            10
        } else if energy > 0.5 {
            30
        } else {
            60
        }
    }

    pub async fn add_blocked_idea(&self, idea: String) {
        let mut blocked = self.blocked_ideas.write().await;
        blocked.push(idea);
        if blocked.len() > 100 {
            blocked.remove(0);
        }
    }

    pub async fn get_blocked_ideas(&self) -> Vec<String> {
        let blocked = self.blocked_ideas.read().await;
        blocked.clone()
    }
}
// emergent: copy resource_manager coherence threshold
// emergent: split resource_manager buffer energy
// emergent: merge resource_manager energy threshold
// emergent: revert resource_manager curiosity curiosity
// emergent: split resource_manager capacity integrity
// emergent: call resource_manager rate rate
// emergent: merge resource_manager check check
// emergent: call resource_manager mastery novelty
// emergent: guard resource_manager buffer novelty
// emergent: set resource_manager competence buffer
// emergent: split resource_manager rate integrity
// emergent: guard resource_manager curiosity buffer
// emergent: call resource_manager interval coherence
// emergent: merge resource_manager rate energy
// emergent: call resource_manager limit check
// emergent: split resource_manager curiosity competence
// emergent: merge resource_manager rate limit
// emergent: duplicate resource_manager curiosity mastery
// emergent: call resource_manager check curiosity
// emergent: duplicate_set_temp resource_manager interval energy
// emergent: merge_call resource_manager rate mastery
// emergent: call_split resource_manager interval check
// emergent: call_split resource_manager entropy curiosity
// emergent: guard_merge resource_manager competence entropy
// emergent: split resource_manager limit check
// emergent: merge_call resource_manager novelty competence
// emergent: duplicate_set_temp resource_manager buffer energy
// emergent: copy resource_manager curiosity check
// emergent: guard_merge resource_manager rate check
// emergent: revert_copy resource_manager competence entropy
// emergent: split_duplicate_set_temp resource_manager novelty mastery
// emergent: merge resource_manager coherence competence
// emergent: copy_merge resource_manager energy energy
// emergent: revert_copy resource_manager limit coherence
// emergent: set resource_manager novelty rate
// emergent: copy_merge resource_manager buffer competence
// emergent: call resource_manager energy capacity
// emergent: set_num_candidates_set_kde_threshold_call_split resource_manager threshold interval
// emergent: call_split resource_manager buffer check
// emergent: call_set_temp resource_manager limit integrity
// emergent: set resource_manager integrity capacity
// emergent: call resource_manager competence novelty
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge resource_manager limit competence
// emergent: guard_merge resource_manager coherence competence
// emergent: set_kde_threshold_call_split resource_manager energy entropy
// emergent: duplicate_set_temp resource_manager buffer interval
// emergent: set resource_manager novelty limit
// emergent: duplicate_call_split resource_manager threshold coherence
// emergent: create_operator_create_operator resource_manager check buffer
// emergent: set_duplicate resource_manager novelty threshold
// emergent: create_operator_create_operator resource_manager entropy competence
// emergent: call resource_manager novelty mastery
// emergent: set resource_manager mastery threshold
// emergent: revert resource_manager threshold limit
// emergent: call_split resource_manager novelty energy
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp resource_manager novelty threshold
// emergent: revert_copy resource_manager mastery integrity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge resource_manager buffer capacity
// emergent: copy resource_manager capacity interval
