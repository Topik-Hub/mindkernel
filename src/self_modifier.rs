use crate::nexus_client::NexusClient;
use crate::resource_manager::ResourceManager;
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;

const LEARN_CYCLE_MIN_ENERGY_FOR_LEARNING_PCT: f64 = 0.15;

pub struct SelfModifier {
    resource_manager: Arc<ResourceManager>,
    nexus_client: Arc<NexusClient>,
    improvements_today: std::sync::atomic::AtomicI32,
    source_dir: String,
    knowledge_file: String,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub idea_id: String,
    pub description: String,
    pub code_diff: String,
    pub confidence: f64,
    pub proposer_model: String,
    pub current_metric: f64,
    pub proposed_metric: f64,
}

#[derive(Debug)]
pub struct ProposalResult {
    pub accepted: bool,
    pub reason: String,
    pub measured_improvement: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct KnowledgeEntry {
    pub timestamp: String,
    pub description: String,
    pub diff: String,
    pub model_used: String,
    pub success: bool,
}

impl SelfModifier {
    pub fn new(
        resource_manager: Arc<ResourceManager>,
        nexus_client: Arc<NexusClient>,
        source_dir: &str,
    ) -> Self {
        let knowledge_file = format!("{}/knowledge.json", source_dir);
        Self {
            resource_manager,
            nexus_client,
            improvements_today: std::sync::atomic::AtomicI32::new(0),
            source_dir: source_dir.to_string(),
            knowledge_file,
        }
    }

    pub fn read_own_source(&self) -> String {
        match std::fs::read_dir(Path::new(&self.source_dir).join("src")) {
            Ok(entries) => entries
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false))
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let len =
                        std::fs::read_to_string(e.path()).map(|s| s.lines().count()).unwrap_or(0);
                    Some(format!("{}({}L)", name, len))
                })
                .collect::<Vec<_>>()
                .join(", "),
            Err(_) => String::new(),
        }
    }

    pub fn load_knowledge(&self) -> Vec<KnowledgeEntry> {
        let path = Path::new(&self.knowledge_file);
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(entries) = serde_json::from_str::<Vec<KnowledgeEntry>>(&data) {
                    return entries;
                }
            }
        }
        Vec::new()
    }

    pub fn save_knowledge(&self, entry: KnowledgeEntry) {
        let mut entries = self.load_knowledge();
        entries.push(entry);
        if entries.len() > 1000 {
            entries.remove(0);
        }
        if let Ok(data) = serde_json::to_string_pretty(&entries) {
            let _ = std::fs::write(&self.knowledge_file, data);
        }
    }

    pub fn get_learned_summary(&self) -> String {
        let entries = self.load_knowledge();
        if entries.is_empty() {
            return "No improvements learned yet.".to_string();
        }
        let success_count = entries.iter().filter(|e| e.success).count();
        let total = entries.len();
        format!(
            "Learned {} improvements ({} successful). Latest: {}",
            total,
            success_count,
            entries.last().map(|e| e.description.as_str()).unwrap_or("none")
        )
    }

    pub async fn learn_cycle(&self) -> String {
        let energy = self.resource_manager.get_energy();
        if energy < LEARN_CYCLE_MIN_ENERGY_FOR_LEARNING_PCT {
            return format!("Self: energy too low ({:.2}). State: dormant.", energy);
        }

        let file_list = self.read_own_source();
        let knowledge = self.get_learned_summary();
        let uptime = self.resource_manager.uptime_sec();
        let improvements = self.improvements_today.load(std::sync::atomic::Ordering::Relaxed);

        let self_state = format!(
            "Self-state: energy={:.2}, uptime={}s, files=[{}], improvements_today={}, knowledge=[{}]",
            energy, uptime, file_list, improvements, knowledge
        );

        self.save_knowledge(KnowledgeEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            description: format!("Internal reflection: {}", self_state),
            diff: String::new(),
            model_used: "mindkernel::self".to_string(),
            success: true,
        });

        tracing::info!("[self] {}", self_state);
        self_state
    }

    async fn apply_improvement(&self, imp: &ImprovementResponse) -> Result<String, String> {
        let file_path = Path::new(&self.source_dir).join("src").join(&imp.file_name);
        if !file_path.exists() {
            return Err(format!("File {} not found", imp.file_name));
        }
        let backup = format!("{}.bak", file_path.display());
        let _ = std::fs::copy(&file_path, &backup);

        let new_content = if imp.code_diff.starts_with("--- ") {
            let original = std::fs::read_to_string(&file_path)
                .map_err(|e| format!("Read failed: {}", e))?;
            apply_unified_diff(&original, &imp.code_diff)?
        } else {
            imp.code_diff.clone()
        };

        std::fs::write(&file_path, &new_content)
            .map_err(|e| format!("Write failed: {}", e))?;

        let signal_path = Path::new(&self.source_dir).join(".rebuild_signal");
        std::fs::write(&signal_path, imp.file_name.clone())
            .map_err(|e| format!("Signal write failed: {}", e))?;

        Ok(format!("Applied {}, rebuild signal sent.", imp.file_name))
    }
}

pub fn apply_unified_diff(original: &str, diff: &str) -> Result<String, String> {
    let original_lines: Vec<&str> = original.lines().collect();
    let diff_lines: Vec<&str> = diff.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    let mut orig_idx = 0usize;

    while i < diff_lines.len() {
        let line = diff_lines[i];

        if line.starts_with("--- ") || line.starts_with("+++ ") {
            i += 1;
            continue;
        }

        if !line.starts_with("@@") {
            i += 1;
            continue;
        }

        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() < 2 {
            i += 1;
            continue;
        }

        let old_range = parts[1].trim_start_matches('-');
        let old_start: usize = old_range
            .split(',')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        i += 1;

        let hunk_start = old_start.saturating_sub(1);
        while orig_idx < hunk_start && orig_idx < original_lines.len() {
            result.push(original_lines[orig_idx].to_string());
            orig_idx += 1;
        }

        while i < diff_lines.len() {
            let dl = diff_lines[i];
            if dl.starts_with("@@") {
                break;
            }

            if dl.starts_with(' ') {
                if orig_idx < original_lines.len() {
                    result.push(original_lines[orig_idx].to_string());
                    orig_idx += 1;
                }
                i += 1;
            } else if dl.starts_with('-') {
                orig_idx += 1;
                i += 1;
            } else if dl.starts_with('+') {
                result.push(dl[1..].to_string());
                i += 1;
            } else {
                result.push(dl.to_string());
                i += 1;
            }
        }
    }

    while orig_idx < original_lines.len() {
        result.push(original_lines[orig_idx].to_string());
        orig_idx += 1;
    }

    Ok(result.join("\n") + "\n")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImprovementResponse {
    pub description: String,
    pub code_diff: String,
    pub file_name: String,
    #[serde(deserialize_with = "deser_f64")]
    pub confidence: f64,
    pub metric_name: String,
    #[serde(deserialize_with = "deser_f64")]
    pub metric_improvement: f64,
    pub model_used: String,
}

fn deser_f64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    use serde::de::Error;
    struct F;
    impl<'de> serde::de::Visitor<'de> for F {
        type Value = f64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a float or string")
        }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<f64, E> { Ok(v) }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<f64, E> { Ok(v as f64) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<f64, E> { Ok(v as f64) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<f64, E> {
            v.replace(',', ".").parse().map_err(Error::custom)
        }
    }
    d.deserialize_any(F)
}
// emergent: copy self_modifier limit check
// emergent: merge self_modifier check mastery
// emergent: merge self_modifier check integrity
// emergent: revert self_modifier rate integrity
// emergent: merge self_modifier competence novelty
// emergent: merge self_modifier threshold threshold
// emergent: set self_modifier buffer mastery
// emergent: guard self_modifier interval coherence
// emergent: set self_modifier mastery energy
// emergent: merge self_modifier curiosity limit
// emergent: split self_modifier entropy interval
// emergent: call self_modifier coherence mastery
// emergent: duplicate self_modifier curiosity check
// emergent: set_split self_modifier interval rate
// emergent: set_split self_modifier integrity entropy
// emergent: set_split self_modifier interval entropy
// emergent: split self_modifier capacity limit
// emergent: duplicate self_modifier entropy check
// emergent: duplicate self_modifier curiosity entropy
// emergent: call self_modifier entropy threshold
// emergent: revert self_modifier limit threshold
// emergent: set_kde_threshold_call_split self_modifier capacity rate
// emergent: merge_call self_modifier energy integrity
// emergent: call_split self_modifier limit curiosity
// emergent: revert self_modifier buffer limit
// emergent: set_num_candidates_set_kde_threshold_call_split self_modifier novelty limit
// emergent: set_duplicate self_modifier mastery entropy
// emergent: split_duplicate_set_temp self_modifier integrity competence
// emergent: set_num_candidates_set_kde_threshold_call_split self_modifier rate integrity
// emergent: split self_modifier energy check
// emergent: call_split self_modifier competence competence
// emergent: set self_modifier interval interval
// emergent: copy self_modifier limit integrity
// emergent: call_split self_modifier threshold curiosity
// emergent: call self_modifier entropy threshold
// emergent: call self_modifier integrity coherence
// emergent: call self_modifier rate threshold
// emergent: duplicate_set_temp self_modifier curiosity threshold
// emergent: guard_merge self_modifier capacity buffer
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp self_modifier energy threshold
// emergent: set_num_candidates_set_kde_threshold_call_split self_modifier energy interval
// emergent: split_duplicate_set_temp_split_duplicate_set_temp self_modifier entropy curiosity
// emergent: copy self_modifier limit competence
// emergent: set_duplicate self_modifier competence interval
// emergent: merge self_modifier energy integrity
// emergent: set_duplicate self_modifier curiosity entropy
// emergent: guard self_modifier rate curiosity
// emergent: guard_set_duplicate self_modifier novelty check
// emergent: call_set_temp self_modifier threshold integrity
// emergent: set_kde_threshold_call_split self_modifier curiosity check
// emergent: copy_set_kde_threshold_revert_copy self_modifier buffer coherence
// emergent: copy_set_kde_threshold_revert_copy self_modifier competence energy
// emergent: copy self_modifier capacity energy
// emergent: split_duplicate_set_temp self_modifier mastery buffer
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_call self_modifier novelty mastery
// emergent: copy_merge self_modifier rate threshold
// emergent: duplicate self_modifier integrity rate
// emergent: copy_merge self_modifier capacity rate
// emergent: merge_call_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge self_modifier integrity limit
// emergent: merge self_modifier mastery buffer
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge self_modifier check mastery
// emergent: call_set_temp self_modifier check limit
// emergent: call self_modifier limit capacity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge self_modifier mastery coherence
// emergent: split self_modifier rate buffer
