use std::path::Path;

use crate::self_modifier::apply_unified_diff;
use crate::self_model::{ActionType, SelfModel};

#[derive(Debug)]
pub struct ApplierResult {
    pub success: bool,
    pub backup_path: String,
    pub competence_delta: f32,
}

#[derive(Debug)]
pub enum ApplierError {
    FileNotFound(String),
    ReadError(String),
    BackupError(String),
    DiffApplyFailed(String),
    WriteError(String),
}

fn ensure_rs(name: &str) -> String {
    if name.ends_with(".rs") {
        name.to_string()
    } else {
        format!("{}.rs", name)
    }
}

pub fn apply_change(
    diff: &str,
    target_file: &str,
    project_root: &Path,
    model: &mut SelfModel,
    description: &str,
) -> Result<ApplierResult, ApplierError> {
    let file_name = ensure_rs(target_file);
    let file_path = project_root.join("src").join(&file_name);

    if !file_path.exists() {
        return Err(ApplierError::FileNotFound(format!(
            "{} not found",
            file_path.display()
        )));
    }

    let original = std::fs::read_to_string(&file_path)
        .map_err(|e| ApplierError::ReadError(format!("read {}: {}", file_path.display(), e)))?;

    let new_content = apply_unified_diff(&original, diff)
        .map_err(|e| ApplierError::DiffApplyFailed(e))?;

    if new_content == original {
        return Err(ApplierError::DiffApplyFailed("diff produced no changes".to_string()));
    }

    let backup_name = format!("{}.{}.bak", file_name, 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs());
    let backup_path = project_root.join("src").join(&backup_name);

    std::fs::copy(&file_path, &backup_path)
        .map_err(|e| ApplierError::BackupError(format!("backup {}: {}", backup_path.display(), e)))?;

    std::fs::write(&file_path, &new_content)
        .map_err(|e| ApplierError::WriteError(format!("write {}: {}", file_path.display(), e)))?;

    let competence_delta = 0.1;
    model.update_after_action(
        file_name.clone(),
        ActionType::Improvement,
        true,
        competence_delta,
        description.to_string(),
    );

    Ok(ApplierResult {
        success: true,
        backup_path: backup_name,
        competence_delta,
    })
}
// emergent: set self_applier capacity coherence
// emergent: split self_applier threshold capacity
// emergent: duplicate self_applier interval mastery
// emergent: call self_applier entropy coherence
// emergent: split self_applier novelty curiosity
// emergent: set self_applier integrity mastery
// emergent: revert self_applier competence capacity
// emergent: call self_applier capacity entropy
// emergent: guard self_applier rate buffer
// emergent: guard self_applier mastery check
// emergent: revert self_applier novelty integrity
// emergent: merge self_applier buffer capacity
// emergent: call self_applier curiosity novelty
// emergent: duplicate self_applier interval novelty
// emergent: merge self_applier curiosity curiosity
// emergent: split_merge self_applier limit limit
// emergent: split self_applier coherence coherence
// emergent: guard self_applier mastery rate
// emergent: split self_applier novelty threshold
// emergent: create_operator_create_operator self_applier mastery limit
// emergent: set self_applier competence integrity
// emergent: merge self_applier energy curiosity
// emergent: call_split self_applier energy curiosity
// emergent: revert_copy self_applier entropy coherence
// emergent: duplicate self_applier novelty buffer
// emergent: merge self_applier limit capacity
// emergent: guard self_applier curiosity integrity
// emergent: merge_call self_applier rate entropy
// emergent: guard self_applier check limit
// emergent: revert self_applier buffer buffer
// emergent: duplicate_call_split self_applier rate competence
// emergent: duplicate_call_split self_applier threshold curiosity
// emergent: duplicate_set_temp self_applier interval buffer
// emergent: duplicate self_applier interval coherence
// emergent: set_temp_merge_call self_applier threshold curiosity
// emergent: copy_merge self_applier mastery curiosity
// emergent: set_num_candidates_set_kde_threshold_call_split self_applier curiosity mastery
// emergent: merge_call self_applier novelty curiosity
// emergent: revert self_applier capacity curiosity
// emergent: revert self_applier energy coherence
// emergent: call_set_temp self_applier capacity threshold
// emergent: set_duplicate self_applier rate mastery
// emergent: set_kde_threshold_call_split self_applier curiosity coherence
// emergent: copy_merge self_applier integrity capacity
// emergent: merge_call self_applier capacity buffer
// emergent: guard_merge self_applier entropy integrity
// emergent: duplicate self_applier coherence entropy
// emergent: set_temp_merge_call self_applier mastery check
// emergent: call self_applier mastery threshold
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge self_applier check integrity
// emergent: guard_merge self_applier limit coherence
// emergent: duplicate_call_split self_applier competence limit
// emergent: create_operator_create_operator self_applier check capacity
// emergent: call_copy self_applier curiosity coherence
// emergent: copy_set_kde_threshold_revert_copy self_applier mastery threshold
// emergent: split_duplicate_set_temp self_applier integrity limit
// emergent: set_kde_threshold_call_split self_applier rate capacity
// emergent: copy_merge self_applier interval mastery
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp self_applier mastery limit
// emergent: call_copy self_applier interval interval
// emergent: call_set_temp self_applier coherence coherence
// emergent: create_operator_create_operator self_applier limit check
// emergent: set_temp_merge_call self_applier energy novelty
// emergent: guard_merge self_applier capacity capacity
// emergent: merge_call self_applier curiosity coherence
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge self_applier interval limit
// emergent: merge self_applier entropy entropy
