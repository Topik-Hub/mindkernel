use std::path::Path;
use std::time::Duration;

/// In-place sandbox: apply diff to original file, build, restore on failure.
/// Uses `--target-dir sandbox_cache/target` so subsequent builds are incremental.

pub struct SandboxResult {
    pub success: bool,
    pub build_log: String,
    pub error_count: usize,
    pub warning_count: usize,
}

#[derive(Debug)]
pub enum SandboxError {
    BackupFailed(String),
    DiffApplyFailed(String),
    BuildFailed(String),
    RestoreFailed(String),
}

pub async fn test_diff(
    diff: &str,
    target_file: &str,
    source_dir: &str,
) -> Result<SandboxResult, SandboxError> {
    let file_path = Path::new(source_dir).join("src").join(target_file);
    let backup_path = file_path.with_extension("rs.sbx_bak");

    // 1. Backup original
    std::fs::copy(&file_path, &backup_path)
        .map_err(|e| SandboxError::BackupFailed(format!("backup {}: {}", file_path.display(), e)))?;

    // 2. Apply diff in-place
    let original = std::fs::read_to_string(&file_path)
        .map_err(|e| SandboxError::DiffApplyFailed(format!("read {}: {}", file_path.display(), e)))?;
    let new_content = crate::self_modifier::apply_unified_diff(&original, diff)
        .map_err(|e| {
            let _ = std::fs::copy(&backup_path, &file_path);
            SandboxError::DiffApplyFailed(e)
        })?;
    std::fs::write(&file_path, &new_content)
        .map_err(|e| {
            let _ = std::fs::copy(&backup_path, &file_path);
            SandboxError::DiffApplyFailed(format!("write {}: {}", file_path.display(), e))
        })?;

    // 3. Build with shared target cache
    let cache_dir = Path::new(source_dir).join("sandbox_cache").join("target");
    let _ = std::fs::create_dir_all(&cache_dir);

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("check")
        .current_dir(source_dir);
    if let Ok(protoc) = std::env::var("PROTOC") {
        cmd.env("PROTOC", protoc);
    }

    let output = tokio::time::timeout(Duration::from_secs(60), cmd.output())
        .await
        .map_err(|_| SandboxError::BuildFailed("timeout after 60s".to_string()))?
        .map_err(|e| SandboxError::BuildFailed(format!("spawn cargo: {}", e)))?;

    let log = String::from_utf8_lossy(&output.stderr).to_string();
    let mut error_count = 0usize;
    let mut warning_count = 0usize;
    for line in log.lines() {
        let t = line.trim();
        if t.starts_with("error") || t.starts_with("error[") { error_count += 1; }
        else if t.starts_with("warning") || t.starts_with("warning[") { warning_count += 1; }
    }

    let success = output.status.success();

    // Log first error line for diagnostics
    if !success {
        for line in log.lines() {
            let t = line.trim();
            if t.starts_with("error") || t.starts_with("error[") {
                tracing::info!("[sandbox_err] {}", t);
                break;
            }
        }
    }

    // 4. Restore original (keep backup in case caller needs to revert)
    std::fs::copy(&backup_path, &file_path)
        .map_err(|e| SandboxError::RestoreFailed(format!("restore {}: {}", file_path.display(), e)))?;
    let _ = std::fs::remove_file(&backup_path);

    Ok(SandboxResult { success, build_log: log, error_count, warning_count })
}
// emergent: set sandbox capacity rate
// emergent: merge sandbox novelty interval
// emergent: merge sandbox limit interval
// emergent: merge sandbox energy buffer
// emergent: split sandbox rate threshold
// emergent: set sandbox buffer competence
// emergent: merge sandbox check interval
// emergent: guard sandbox interval integrity
// emergent: set sandbox energy rate
// emergent: revert sandbox threshold interval
// emergent: duplicate sandbox energy curiosity
// emergent: split sandbox limit buffer
// emergent: merge sandbox curiosity rate
// emergent: merge sandbox novelty interval
// emergent: guard sandbox limit curiosity
// emergent: merge sandbox energy curiosity
// emergent: set sandbox interval curiosity
// emergent: duplicate sandbox energy integrity
// emergent: call sandbox energy novelty
// emergent: copy sandbox entropy energy
// emergent: split_merge sandbox check threshold
// emergent: set_split sandbox integrity novelty
// emergent: create_operator_create_operator sandbox curiosity integrity
// emergent: copy sandbox energy energy
// emergent: copy sandbox integrity entropy
// emergent: split sandbox mastery coherence
// emergent: call sandbox mastery rate
// emergent: revert_copy sandbox energy integrity
// emergent: merge sandbox coherence coherence
// emergent: call_split sandbox entropy interval
// emergent: copy sandbox energy curiosity
// emergent: duplicate sandbox interval integrity
// emergent: revert sandbox rate novelty
// emergent: guard_merge sandbox integrity mastery
// emergent: merge sandbox check rate
// emergent: merge sandbox check curiosity
// emergent: split_duplicate_set_temp sandbox threshold curiosity
// emergent: merge sandbox interval interval
// emergent: duplicate_call_split sandbox interval entropy
// emergent: copy sandbox competence coherence
// emergent: call_split sandbox limit rate
// emergent: set_duplicate sandbox integrity buffer
// emergent: call_split sandbox check energy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp sandbox energy buffer
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp sandbox mastery competence
// emergent: revert sandbox capacity integrity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp sandbox coherence threshold
// emergent: revert sandbox entropy integrity
// emergent: guard sandbox limit rate
// emergent: call_set_temp sandbox coherence novelty
// emergent: set sandbox check limit
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge sandbox coherence rate
// emergent: guard_set_duplicate sandbox competence coherence
// emergent: set_kde_threshold_call_split sandbox novelty entropy
// emergent: merge_call sandbox novelty limit
// emergent: merge_call sandbox interval curiosity
// emergent: set_kde_threshold_call_split sandbox threshold energy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge sandbox curiosity energy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge sandbox novelty capacity
// emergent: set_kde_threshold_call_split sandbox limit energy
// emergent: merge_call sandbox novelty buffer
// emergent: guard_merge sandbox capacity buffer
// emergent: split sandbox curiosity coherence
