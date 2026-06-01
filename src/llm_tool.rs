use crate::emergent;
use crate::nexus_client::NexusClient;
use crate::self_model::SelfModel;
use std::time::Duration;

#[derive(Debug)]
pub enum LlmError {
    LlmCallFailed(String),
    InvalidDiff(String),
    NoDiffGenerated,
}

#[derive(Debug)]
pub struct Diff {
    pub original_file: String,
    pub unified_diff: String,
    pub description: String,
}

const MAX_RETRIES: u32 = 1;
const RETRY_DELAYS_MS: &[u64] = &[2_000];

fn synth_diff(action: &emergent::Action, file: &str, current_code: &str) -> Diff {
    if action.operation == "create_operator" {
        let target_str = format!("\"{}\"", action.target);
        if current_code.contains(&target_str) {
            return Diff {
                original_file: file.to_string(),
                unified_diff: String::new(),
                description: format!("skip duplicate operator {}", action.target),
            };
        }
        let lines: Vec<&str> = current_code.lines().collect();
        let mut insert_idx = 0usize;
        let mut in_base = false;
        for (i, line) in lines.iter().enumerate() {
            if line.contains("BASE_OPERATIONS") {
                in_base = true;
                continue;
            }
            if in_base && line.trim() == "];" {
                insert_idx = i;
                break;
            }
        }
        let desc = if insert_idx > 0 {
            let lnum = insert_idx + 1;
            format!(
                "--- a/{file}\n+++ b/{file}\n@@ -{lnum},0 +{lnum2},1 @@\n+    \"{name}\",",
                file = file, lnum = lnum, lnum2 = lnum + 1, name = action.target
            )
        } else {
            let n = lines.len();
            format!(
                "--- a/{file}\n+++ b/{file}\n@@ -{n},0 +{npp},1 @@\n+    \"{name}\",",
                file = file, n = n, npp = n + 1, name = action.target
            )
        };
        return Diff {
            original_file: file.to_string(),
            unified_diff: desc,
            description: format!("create operator {}", action.target),
        };
    }
    let comment = format!(
        "// emergent: {} {} {} {}",
        action.operation, action.target, action.param1, action.param2
    );
    let lines: Vec<&str> = current_code.lines().collect();
    let n = lines.len();
    let diff = if n == 0 {
        format!("--- a/{file}\n+++ b/{file}\n@@ -0,0 +1,1 @@\n+{comment}\n")
    } else {
        format!("--- a/{file}\n+++ b/{file}\n@@ -{n},0 +{npp},1 @@\n+{comment}",
            file = file, n = n, npp = n + 1, comment = comment)
    };
    Diff {
        original_file: file.to_string(),
        unified_diff: diff,
        description: format!("{} {} {} {}", action.operation, action.target, action.param1, action.param2),
    }
}

async fn try_llm(nexus: &NexusClient, prompt: &str, file: &str) -> Option<Diff> {
    let response = nexus.request_improvement(prompt).await?;
    if response.code_diff.is_empty() {
        return None;
    }
    if !response.file_name.is_empty() && !response.file_name.contains(&file.trim_end_matches(".rs")) {
        return None;
    }
    Some(Diff {
        original_file: file.to_string(),
        unified_diff: response.code_diff,
        description: if response.description.is_empty() {
            String::new()
        } else {
            response.description
        },
    })
}

pub async fn generate_diff_from_action(
    action: &emergent::Action,
    _model: &SelfModel,
    _nexus: &NexusClient,
    source_dir: &str,
) -> Result<Diff, LlmError> {
    if action.operation == "create_operator" {
        let file = "emergent.rs".to_string();
        let fpath = std::path::Path::new(source_dir).join("src").join(&file);
        let current_code = std::fs::read_to_string(&fpath).unwrap_or_default();
        return Ok(synth_diff(action, &file, &current_code));
    }

    // Non-create_operator: always use safe synth_diff directly
    let file = if action.target.ends_with(".rs") {
        action.target.clone()
    } else {
        format!("{}.rs", action.target)
    };
    let fpath = std::path::Path::new(source_dir).join("src").join(&file);
    let current_code = std::fs::read_to_string(&fpath).unwrap_or_default();
    Ok(synth_diff(action, &file, &current_code))
}
// emergent: call llm_tool coherence capacity
// emergent: call llm_tool integrity competence
// emergent: merge llm_tool competence rate
// emergent: split llm_tool entropy rate
// emergent: revert llm_tool energy threshold
// emergent: guard llm_tool competence curiosity
// emergent: duplicate llm_tool coherence competence
// emergent: set llm_tool competence competence
// emergent: copy llm_tool limit competence
// emergent: duplicate llm_tool rate check
// emergent: set llm_tool limit novelty
// emergent: revert llm_tool coherence buffer
// emergent: copy llm_tool rate limit
// emergent: set llm_tool rate curiosity
// emergent: merge llm_tool limit novelty
// emergent: split llm_tool competence energy
// emergent: duplicate llm_tool check threshold
// emergent: duplicate llm_tool energy buffer
// emergent: split llm_tool rate integrity
// emergent: duplicate_set_temp llm_tool check limit
// emergent: revert llm_tool limit integrity
// emergent: duplicate llm_tool curiosity mastery
// emergent: merge llm_tool limit entropy
// emergent: split llm_tool threshold limit
// emergent: merge_call llm_tool check mastery
// emergent: revert llm_tool integrity limit
// emergent: merge llm_tool buffer limit
// emergent: set_num_candidates_set_kde_threshold_call_split llm_tool coherence buffer
// emergent: duplicate llm_tool coherence limit
// emergent: set_num_candidates_set_kde_threshold_call_split llm_tool limit energy
// emergent: duplicate llm_tool limit entropy
// emergent: revert llm_tool curiosity mastery
// emergent: merge_call llm_tool capacity buffer
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp llm_tool coherence threshold
// emergent: guard_merge llm_tool interval competence
// emergent: revert_copy llm_tool competence curiosity
// emergent: merge_call llm_tool entropy capacity
// emergent: duplicate_set_temp llm_tool rate check
// emergent: merge llm_tool mastery coherence
// emergent: split_duplicate_set_temp_split_duplicate_set_temp llm_tool coherence check
// emergent: copy llm_tool buffer interval
// emergent: copy llm_tool interval interval
// emergent: call_split llm_tool integrity competence
// emergent: set llm_tool capacity interval
// emergent: copy_merge llm_tool competence interval
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge llm_tool integrity integrity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp llm_tool curiosity entropy
// emergent: duplicate_set_temp llm_tool capacity capacity
// emergent: revert llm_tool threshold novelty
// emergent: copy_set_kde_threshold llm_tool integrity interval
// emergent: merge llm_tool mastery limit
// emergent: duplicate_call_split llm_tool coherence energy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_call llm_tool check entropy
// emergent: copy llm_tool entropy limit
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_call llm_tool novelty capacity
// emergent: revert_copy llm_tool entropy mastery
// emergent: call_copy llm_tool threshold interval
// emergent: copy llm_tool energy interval
// emergent: call_set_temp llm_tool capacity energy
// emergent: guard llm_tool threshold entropy
// emergent: set llm_tool novelty integrity
