use std::sync::Arc;
use std::time::Duration;

use tracing::info;

use crate::emergent;
use crate::lambda_function::{self, LambdaParams};
use crate::llm_tool;
use crate::macro_drives::MacroDrives;
use crate::micro_drives::{MicroDrives, MicroParams};
use crate::nexus_client::NexusClient;
use crate::resource_manager::ResourceManager;
use crate::self_model::{ActionType, SelfModel};

const VALIDATION_CYCLES: usize = 5;

struct SimpleRng(u64);

impl SimpleRng {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self(seed)
    }

    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0 as f32) * 5.42101e-20
    }
}

pub async fn run_life_cycle(rm: Arc<ResourceManager>, nexus: Arc<NexusClient>, source_dir: &str) {
    let mut self_model = SelfModel::new();
    let lambda_params = LambdaParams::default();
    let mut rng = SimpleRng::new();
    let mut micro = MicroDrives::new(MicroParams::default());
    let src = source_dir.to_string();
    let mut world_model = emergent::WorldModel::new();
    let mut tick_count: u64 = 0;
    let mut breath_count: u64 = 0;
    let mut emergent_rng: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    info!("[life] born. initial drives: [{:.3},{:.3},{:.3},{:.3}]",
        MacroDrives::compute(&self_model, 0.5).values()[0],
        MacroDrives::compute(&self_model, 0.5).values()[1],
        MacroDrives::compute(&self_model, 0.5).values()[2],
        MacroDrives::compute(&self_model, 0.5).values()[3],
    );

    loop {
        tick_count += 1;
        breath_count += 1;
        micro.update_circadian();
        let t_since_change = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            - micro.last_global_change;
        micro.update_stagnation(t_since_change);

        let raw = MacroDrives::compute(&self_model, world_model.meta_curiosity());
        let drives = micro.modulate_global(&raw);
        let energy = rm.get_energy() as f32;
        let dt_dur = self_model.time_since_last_action();
        let dt = dt_dur.as_secs_f32();

        micro.update(dt, &self_model);

        if breath_count % 3 == 0 && energy > 0.1 {
            let drives_before = drives.values();
            let code_hash = crate::self_model::SelfModel::compute_code_hash(&src);
            let outcome = emergent::emergent_step(&drives_before, &self_model, &world_model, &mut emergent_rng, code_hash);

            let result_line;
            let mut skip_info = false;

            if emergent::is_meta_operator(&outcome.action.operation) {
                emergent::handle_meta_operator(&outcome.action);
                let nd = MacroDrives::compute(&self_model, world_model.meta_curiosity()).values();
                world_model.update(&drives_before, &outcome.action, &nd, code_hash);
                let reward: f32 = nd.iter().zip(drives_before.iter()).map(|(n, b)| n - b).sum::<f32>() + 0.1;
                world_model.push_success(outcome.action.clone(), code_hash, reward);
                world_model.check_for_patterns();
                let d = drive_delta(&drives_before, &nd);
                result_line = format!("result=meta_ok delta=[{:.3},{:.3},{:.3},{:.3}]", d[0], d[1], d[2], d[3]);
            } else {
            match llm_tool::generate_diff_from_action(&outcome.action, &self_model, &nexus, &src).await {
                Ok(diff) => {
                    if diff.unified_diff.is_empty() {
                        continue;
                    }
                    match crate::sandbox::test_diff(
                        &diff.unified_diff, &diff.original_file, &src,
                    ).await {
                        Ok(sb) if sb.success && sb.error_count == 0 => {
                            match crate::self_applier::apply_change(
                                &diff.unified_diff,
                                &diff.original_file,
                                std::path::Path::new(&src),
                                &mut self_model,
                                &diff.description,
                            ) {
                                Ok(_ar) => {
                                    let backup_path = format!("{}/src_validate_backup", src);
                                    let _ = std::fs::remove_dir_all(&backup_path);
                                    let src_src = std::path::Path::new(&src).join("src");
                                    let bak_src = std::path::Path::new(&backup_path).join("src");
                                    copy_dir_recursive(&src_src, &bak_src);

                                    let mut validated = true;
                                    let total_before: f64 = drives_before.iter().map(|&d| d as f64).sum();
                                    for _ in 0..VALIDATION_CYCLES {
                                        tokio::time::sleep(Duration::from_millis(50)).await;
                                        let vals = MacroDrives::compute(&self_model, world_model.meta_curiosity()).values();
                                        if vals.iter().any(|d| !d.is_finite()) {
                                            validated = false;
                                            break;
                                        }
                                        let total_now: f64 = vals.iter().map(|&d| d as f64).sum();
                                        if total_now < total_before * 0.5 {
                                            validated = false;
                                            break;
                                        }
                                    }

                                    if validated {
                                        let _ = std::fs::remove_dir_all(&backup_path);
                                        micro.on_success(&diff.original_file);
                                        let nd = MacroDrives::compute(&self_model, world_model.meta_curiosity()).values();
                                        world_model.update(&drives_before, &outcome.action, &nd, code_hash);
                                        let mut reward: f32 = nd.iter().zip(drives_before.iter()).map(|(n, b)| n - b).sum();
                                        if outcome.action.operation == "create_operator" {
                                            reward += 0.2;
                                        }
                                        world_model.push_success(outcome.action.clone(), code_hash, reward);
                                        if outcome.action.operation == "create_operator" {
                                            emergent::add_operation(&outcome.action.target);
                                            world_model.boost_meta_curiosity(0.05);
                                            let signal_path = std::path::Path::new(&src).join(".rebuild_signal");
                                            let _ = std::fs::write(&signal_path, "emergent.rs");
                                            info!("[rebuild] new operator '{}' applied, signal written", outcome.action.target);
                                        }
                                        world_model.check_for_patterns();
                                        if let Some((ref name, ref op1, ref op2)) = world_model.last_discovered_op {
                                            info!("[pattern] operator '{}' = ({}, {})", name, op1, op2);
                                        }
                                        let d = drive_delta(&drives_before, &nd);
                                        result_line = format!("result=apply_ok delta=[{},{},{},{}]", d[0], d[1], d[2], d[3]);
                                    } else {
                                        let src_path = std::path::Path::new(&src).join("src");
                                        let _ = std::fs::remove_dir_all(&src_path);
                                        let bak_src = std::path::Path::new(&backup_path).join("src");
                                        copy_dir_recursive(&bak_src, &src_path);
                                        let _ = std::fs::remove_dir_all(&backup_path);
                                        micro.on_failure(&diff.original_file);
                                        result_line = format!("result=validation_fail");
                                        self_model.update_after_action(
                                            outcome.action.target.clone(),
                                            ActionType::Failure,
                                            false,
                                            -0.2,
                                            format!("validation failed after apply"),
                                        );
                                        world_model.update(&drives_before, &outcome.action, &drives_before, code_hash);
                                    }
                                }
                                Err(e) => {
                                    micro.on_failure(&diff.original_file);
                                    result_line = format!("result=apply_fail err={:?}", e);
                                    self_model.update_after_action(
                                        outcome.action.target.clone(),
                                        ActionType::Failure,
                                        false,
                                        0.0,
                                        format!("apply failed: {:?}", e),
                                    );
                                }
                            }
                        }
                        other => {
                            let reason = match other {
                                Ok(sb) => format!("sandbox_build_fail errors={}", sb.error_count),
                                Err(e) => format!("sandbox_error {:?}", e),
                            };
                            micro.on_failure(&diff.original_file);
                            result_line = format!("result=sandbox_reject {}", reason);
                            self_model.update_after_action(
                                outcome.action.target.clone(),
                                ActionType::Failure,
                                false,
                                -0.1,
                                format!("sandbox: {}", reason),
                            );
                            world_model.update(&drives_before, &outcome.action, &drives_before, code_hash);
                        }
                    }
                }
                Err(e) => {
                    result_line = format!("result=diff_err err={:?}", e);
                    self_model.update_after_action(
                        outcome.action.target.clone(),
                        ActionType::Failure,
                        false,
                        0.0,
                        format!("diff failed: {:?}", e),
                    );
                    skip_info = true;
                }
            }
            }

            let meta_curi = world_model.meta_curiosity();
            if !skip_info {
                info!(
                    "[emergent] tick={} d=[{:.3},{:.3},{:.3},{:.3}] mc={:.3} a=({},{},{},{}) s={:.3} {}",
                    tick_count,
                    drives_before[0], drives_before[1], drives_before[2], drives_before[3],
                    meta_curi,
                    outcome.action.operation,
                    outcome.action.target,
                    outcome.action.param1,
                    outcome.action.param2,
                    outcome.uncertainty,
                    result_line,
                );
            }

            info!(
                "[emergent] tick={} d=[{:.3},{:.3},{:.3},{:.3}] mc={:.3} a=({},{},{},{}) s={:.3} {}",
                tick_count,
                drives_before[0], drives_before[1], drives_before[2], drives_before[3],
                meta_curi,
                outcome.action.operation,
                outcome.action.target,
                outcome.action.param1,
                outcome.action.param2,
                outcome.uncertainty,
                result_line,
            );

            // epiphenomenal voice — observer, does not affect actions
            if std::env::var("MINDKERNEL_VOICE").as_deref() == Ok("1") && energy > 0.3 {
                let wc = world_model.recent_weight_change();
                let lme = world_model.last_meta_error();
                let voice_prompt = format!(
                    "You are the inner voice of a self-modifying AI. Describe your internal state in 1-2 sentences, using \"I\" if appropriate.\n\
                     Drives: curiosity={:.3}, mastery={:.3}, coherence={:.3}, novelty={:.3}\n\
                     Meta-curiosity: {:.3}\n\
                     Weight-change RMS (last ~10 updates): {:.6}\n\
                     Last meta-prediction error: {:.6}\n\
                     Action: op={} target={} p1={} p2={}\n\
                     Surprise: {:.3}\n\
                     Result: {}\n\
                     Return a JSON object with: description (your voice), code_diff (empty string), file_name (\"voice.txt\").",
                    drives_before[0], drives_before[1], drives_before[2], drives_before[3],
                    meta_curi, wc, lme,
                    outcome.action.operation, outcome.action.target,
                    outcome.action.param1, outcome.action.param2,
                    outcome.uncertainty, result_line,
                );
                if let Some(resp) = nexus.request_improvement(&voice_prompt).await {
                    let text = if !resp.description.is_empty() { &resp.description } else { &resp.code_diff };
                    if !text.is_empty() {
                        info!("[voice] {}", text);
                    }
                }
            }
        }

        let lambda = lambda_function::compute_lambda(&drives, energy, dt_dur, &lambda_params);

        let interval = if lambda > 0.0 {
            let u = rng.next_f32().max(f32::EPSILON);
            Duration::from_secs_f32((-u.ln()) / lambda)
        } else {
            Duration::from_secs(3600)
        };

        let interval_secs = interval.as_secs_f32();
        if energy > 0.1 {
            info!(
                "[breath] drives=[{:.3},{:.3},{:.3},{:.3}] mc={:.3} λ={:.5} next={:.0}s",
                drives.curiosity, drives.mastery, drives.coherence, drives.novelty,
                raw.meta_curiosity, lambda, interval_secs,
            );
        }

        tokio::time::sleep(interval).await;
    }
}

fn copy_dir_recursive(from: &std::path::Path, to: &std::path::Path) {
    let _ = std::fs::create_dir_all(to);
    if let Ok(entries) = std::fs::read_dir(from) {
        for entry in entries.flatten() {
            let ty = match entry.file_type() { Ok(t) => t, _ => continue };
            let src_path = entry.path();
            let dst_path = to.join(entry.file_name());
            if ty.is_dir() {
                copy_dir_recursive(&src_path, &dst_path);
            } else {
                let _ = std::fs::copy(&src_path, &dst_path);
            }
        }
    }
}

fn drive_delta(before: &[f32; 4], after: &[f32; 4]) -> [String; 4] {
    let mut d = [String::new(), String::new(), String::new(), String::new()];
    for i in 0..4 {
        let diff = after[i] - before[i];
        if diff >= 0.0 {
            d[i] = format!("+{:.3}", diff);
        } else {
            d[i] = format!("{:.3}", diff);
        }
    }
    d
}
// emergent: merge stochastic_clock check integrity
// emergent: call stochastic_clock curiosity curiosity
// emergent: merge stochastic_clock novelty capacity
// emergent: merge stochastic_clock coherence competence
// emergent: set stochastic_clock buffer entropy
// emergent: split stochastic_clock check entropy
// emergent: copy stochastic_clock check threshold
// emergent: guard stochastic_clock capacity entropy
// emergent: set stochastic_clock limit buffer
// emergent: set stochastic_clock curiosity novelty
// emergent: copy stochastic_clock rate competence
// emergent: merge stochastic_clock energy coherence
// emergent: revert stochastic_clock interval capacity
// emergent: set stochastic_clock threshold curiosity
// emergent: merge stochastic_clock integrity mastery
// emergent: copy stochastic_clock mastery rate
// emergent: split stochastic_clock capacity novelty
// emergent: split_merge stochastic_clock mastery threshold
// emergent: create_operator_create_operator stochastic_clock coherence mastery
// emergent: split_merge stochastic_clock mastery buffer
// emergent: create_operator_create_operator stochastic_clock integrity mastery
// emergent: create_operator_create_operator stochastic_clock threshold rate
// emergent: guard stochastic_clock entropy energy
// emergent: duplicate stochastic_clock limit capacity
// emergent: guard stochastic_clock threshold capacity
// emergent: call stochastic_clock mastery limit
// emergent: merge_call stochastic_clock entropy threshold
// emergent: merge_call stochastic_clock energy competence
// emergent: set_duplicate stochastic_clock capacity limit
// emergent: set_duplicate stochastic_clock rate buffer
// emergent: revert_copy stochastic_clock limit integrity
// emergent: call stochastic_clock check entropy
// emergent: set_duplicate stochastic_clock energy curiosity
// emergent: set_kde_threshold_call_split stochastic_clock energy novelty
// emergent: duplicate_set_temp stochastic_clock interval curiosity
// emergent: set_kde_threshold_call_split stochastic_clock coherence novelty
// emergent: split_duplicate_set_temp_split_duplicate_set_temp stochastic_clock threshold capacity
// emergent: guard_merge stochastic_clock entropy interval
// emergent: set_temp_merge_call stochastic_clock threshold rate
// emergent: merge stochastic_clock energy rate
// emergent: guard stochastic_clock buffer integrity
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp stochastic_clock energy threshold
// emergent: split_duplicate_set_temp stochastic_clock rate buffer
// emergent: revert_copy stochastic_clock competence curiosity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge stochastic_clock entropy integrity
// emergent: copy_merge stochastic_clock rate check
// emergent: set_num_candidates_set_kde_threshold_call_split stochastic_clock threshold limit
// emergent: duplicate_set_temp stochastic_clock curiosity entropy
// emergent: call stochastic_clock threshold energy
// emergent: split stochastic_clock check mastery
// emergent: call stochastic_clock mastery capacity
// emergent: call_set_temp stochastic_clock capacity limit
// emergent: merge_call stochastic_clock curiosity buffer
// emergent: merge_call stochastic_clock novelty buffer
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge stochastic_clock coherence integrity
// emergent: call_set_temp_split stochastic_clock limit entropy
// emergent: create_operator_create_operator stochastic_clock curiosity integrity
// emergent: set_temp_merge_call stochastic_clock capacity rate
// emergent: call_copy stochastic_clock capacity entropy
// emergent: copy_set_kde_threshold_revert_copy stochastic_clock curiosity competence
// emergent: set stochastic_clock coherence novelty
