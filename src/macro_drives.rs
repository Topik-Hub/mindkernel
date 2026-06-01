use crate::self_model::SelfModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveType {
    Curiosity,
    Mastery,
    Coherence,
    Novelty,
}

#[derive(Debug, Clone)]
pub struct MacroDrives {
    pub curiosity: f32,
    pub mastery: f32,
    pub coherence: f32,
    pub novelty: f32,
    pub meta_curiosity: f32,
}

const LP_WINDOW: usize = 10;
const DECAY_NOVELTY_S: f32 = 3600.0;
const MAX_ENTROPY: f32 = 10.0;
const CURIOSITY_NEUTRAL: f32 = 0.5;

impl MacroDrives {
    pub fn compute(model: &SelfModel, meta_curiosity: f32) -> Self {
        let curiosity = if model.learning_progress_queue.len() < 2 {
            CURIOSITY_NEUTRAL
        } else {
            let first = model.learning_progress_queue.front().unwrap();
            let last = model.learning_progress_queue.back().unwrap();
            let derivative = last - first;
            (derivative / 0.1).clamp(0.0, 1.0).max(0.1)
        };

        let mastery = 1.0 - model.average_competence();

        let entropy_norm = (model.entropy / MAX_ENTROPY).clamp(0.0, 1.0);
        let coherence = 1.0 - entropy_norm;

        let dt = model.time_since_last_new_module().as_secs_f32();
        let novelty = (-dt / DECAY_NOVELTY_S).exp();

        Self {
            curiosity: curiosity.clamp(0.0, 1.0),
            mastery: mastery.clamp(0.0, 1.0),
            coherence: coherence.clamp(0.0, 1.0),
            novelty: novelty.clamp(0.0, 1.0),
            meta_curiosity: meta_curiosity.clamp(0.0, 1.0),
        }
    }

    pub fn max(&self) -> f32 {
        self.curiosity
            .max(self.mastery)
            .max(self.coherence)
            .max(self.novelty)
    }

    pub fn dominant(&self) -> DriveType {
        let max_val = self.max();
        if self.curiosity >= max_val {
            DriveType::Curiosity
        } else if self.mastery >= max_val {
            DriveType::Mastery
        } else if self.coherence >= max_val {
            DriveType::Coherence
        } else {
            DriveType::Novelty
        }
    }

    pub fn values(&self) -> [f32; 4] {
        [self.curiosity, self.mastery, self.coherence, self.novelty]
    }

    pub fn weighted_sum(&self, weights: Option<[f32; 4]>) -> f32 {
        let w = weights.unwrap_or([0.25; 4]);
        self.curiosity * w[0]
            + self.mastery * w[1]
            + self.coherence * w[2]
            + self.novelty * w[3]
    }
}
// emergent: set macro_drives integrity competence
// emergent: set macro_drives integrity capacity
// emergent: copy macro_drives competence novelty
// emergent: split macro_drives buffer buffer
// emergent: duplicate macro_drives novelty competence
// emergent: merge macro_drives competence capacity
// emergent: merge macro_drives check coherence
// emergent: guard macro_drives threshold entropy
// emergent: merge macro_drives rate novelty
// emergent: revert macro_drives novelty capacity
// emergent: call macro_drives energy curiosity
// emergent: call macro_drives coherence integrity
// emergent: revert macro_drives threshold check
// emergent: guard macro_drives competence threshold
// emergent: guard macro_drives rate buffer
// emergent: revert macro_drives interval threshold
// emergent: merge macro_drives capacity interval
// emergent: revert_copy macro_drives entropy novelty
// emergent: copy macro_drives coherence energy
// emergent: call macro_drives curiosity mastery
// emergent: split macro_drives interval check
// emergent: duplicate macro_drives buffer threshold
// emergent: merge_call macro_drives capacity curiosity
// emergent: revert_copy macro_drives threshold limit
// emergent: call macro_drives competence rate
// emergent: duplicate_set_temp macro_drives interval capacity
// emergent: duplicate_call_split macro_drives novelty competence
// emergent: split macro_drives threshold integrity
// emergent: revert macro_drives novelty competence
// emergent: guard_merge macro_drives integrity novelty
// emergent: split macro_drives curiosity energy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp macro_drives threshold energy
// emergent: duplicate_set_temp macro_drives novelty capacity
// emergent: call_set_temp macro_drives energy integrity
// emergent: duplicate macro_drives competence entropy
// emergent: merge_call macro_drives threshold coherence
// emergent: merge macro_drives curiosity entropy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge macro_drives limit threshold
// emergent: call_split macro_drives rate limit
// emergent: set_num_candidates_set_kde_threshold_call_split macro_drives energy energy
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp macro_drives energy rate
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge macro_drives entropy integrity
// emergent: duplicate macro_drives interval mastery
// emergent: revert_copy macro_drives check rate
// emergent: guard macro_drives rate integrity
// emergent: copy_set_kde_threshold macro_drives capacity rate
// emergent: create_operator_create_operator macro_drives competence energy
// emergent: split macro_drives curiosity integrity
// emergent: call_set_temp_split macro_drives novelty buffer
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge macro_drives competence entropy
// emergent: merge_call_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge macro_drives rate competence
// emergent: split macro_drives threshold curiosity
