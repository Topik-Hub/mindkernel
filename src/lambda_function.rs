use std::time::Duration;

use crate::macro_drives::MacroDrives;

#[derive(Debug, Clone)]
pub struct LambdaParams {
    pub alpha: f32,
    pub beta_sec: f32,
    pub gamma: f32,
    pub epsilon: f32,
}

impl Default for LambdaParams {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta_sec: 1.0,
            gamma: 1.0,
            epsilon: 0.1,
        }
    }
}

pub fn compute_lambda(
    drives: &MacroDrives,
    energy: f32,
    time_since_last_action: Duration,
    params: &LambdaParams,
) -> f32 {
    let t = time_since_last_action.as_secs_f32();
    let sigmoid_input = (t - params.beta_sec) / params.gamma;
    let sigmoid = 1.0 / (1.0 + (-sigmoid_input).exp());
    params.alpha * drives.curiosity * energy * sigmoid + params.epsilon
}
// emergent: merge lambda_function threshold novelty
// emergent: merge lambda_function entropy limit
// emergent: split lambda_function threshold competence
// emergent: set lambda_function curiosity threshold
// emergent: set lambda_function integrity buffer
// emergent: merge lambda_function limit check
// emergent: set lambda_function threshold entropy
// emergent: guard lambda_function coherence check
// emergent: guard lambda_function entropy novelty
// emergent: revert lambda_function integrity energy
// emergent: guard lambda_function novelty competence
// emergent: duplicate lambda_function buffer limit
// emergent: merge lambda_function energy capacity
// emergent: revert lambda_function novelty competence
// emergent: revert lambda_function curiosity integrity
// emergent: merge lambda_function limit curiosity
// emergent: split_merge lambda_function entropy interval
// emergent: merge lambda_function check capacity
// emergent: set_split lambda_function buffer integrity
// emergent: call lambda_function competence threshold
// emergent: split lambda_function competence interval
// emergent: merge lambda_function buffer entropy
// emergent: set lambda_function competence energy
// emergent: call lambda_function limit interval
// emergent: set lambda_function buffer entropy
// emergent: duplicate_set_temp lambda_function mastery buffer
// emergent: duplicate lambda_function novelty novelty
// emergent: merge lambda_function integrity mastery
// emergent: set lambda_function integrity novelty
// emergent: call lambda_function mastery coherence
// emergent: call_split lambda_function rate threshold
// emergent: duplicate lambda_function curiosity rate
// emergent: duplicate_call_split lambda_function novelty curiosity
// emergent: set_kde_threshold_call_split lambda_function entropy energy
// emergent: duplicate lambda_function coherence novelty
// emergent: set lambda_function rate capacity
// emergent: call_split lambda_function coherence threshold
// emergent: copy_merge lambda_function interval interval
// emergent: split_duplicate_set_temp lambda_function novelty curiosity
// emergent: copy lambda_function energy coherence
// emergent: set_duplicate lambda_function capacity limit
// emergent: duplicate_set_temp lambda_function threshold rate
// emergent: guard lambda_function capacity check
// emergent: set_duplicate lambda_function limit coherence
// emergent: revert_copy lambda_function threshold competence
// emergent: copy_merge lambda_function competence mastery
// emergent: duplicate_call_split lambda_function threshold coherence
// emergent: copy_merge lambda_function mastery energy
// emergent: merge lambda_function coherence competence
// emergent: call_set_temp lambda_function entropy coherence
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge lambda_function curiosity interval
// emergent: guard_merge lambda_function threshold curiosity
// emergent: set_duplicate lambda_function buffer interval
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge lambda_function energy integrity
// emergent: call_split lambda_function competence energy
// emergent: copy_merge lambda_function mastery energy
// emergent: call_split lambda_function curiosity buffer
// emergent: copy lambda_function integrity capacity
// emergent: revert lambda_function novelty threshold
// emergent: copy_set_kde_threshold lambda_function buffer interval
// emergent: revert_copy lambda_function check novelty
// emergent: create_operator_create_operator lambda_function integrity threshold
// emergent: copy_set_kde_threshold_revert_copy lambda_function energy energy
// emergent: guard_set_duplicate lambda_function interval rate
// emergent: duplicate lambda_function interval entropy
// emergent: set lambda_function integrity coherence
// emergent: revert_copy_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge lambda_function capacity rate
// emergent: duplicate_set_temp lambda_function competence energy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge lambda_function interval limit
// emergent: set_kde_threshold_call_split lambda_function mastery rate
// emergent: guard lambda_function check mastery
// emergent: copy lambda_function entropy check
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge lambda_function check rate
// emergent: copy lambda_function rate novelty
// emergent: set_num_candidates_set_kde_threshold_call_split lambda_function entropy check
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge lambda_function threshold competence
// emergent: set lambda_function mastery threshold
// emergent: call_set_temp lambda_function interval rate
// emergent: call_split lambda_function mastery entropy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge lambda_function capacity energy
// emergent: merge lambda_function capacity threshold
