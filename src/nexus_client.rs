use reqwest::Client;
use std::time::Duration;

use crate::self_modifier::ImprovementResponse;

#[derive(Clone)]
pub struct NexusClient {
    client: Client,
    base_url: String,
}

#[derive(serde::Deserialize, Debug)]
struct NexusTask {
    title: Option<String>,
    status: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct NexusProject {
    name: Option<String>,
}

impl NexusClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(130))
            .build()
            .expect("Failed to build reqwest Client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn fetch_context(&self) -> Vec<String> {
        let mut context = Vec::new();

        if let Ok(tasks) = self.fetch_tasks().await {
            if tasks.is_empty() {
                context.push("Nexus: no recent tasks".to_string());
            } else {
                context.push(format!("Nexus recent tasks ({}):", tasks.len()));
                for t in tasks.iter().take(5) {
                    let title = t.title.as_deref().unwrap_or("untitled");
                    let status = t.status.as_deref().unwrap_or("unknown");
                    context.push(format!("  - [{}] {}", status, title));
                }
            }
        } else {
            context.push("Nexus: tasks unavailable".to_string());
        }

        if let Ok(projects) = self.fetch_projects().await {
            if projects.is_empty() {
                context.push("Nexus: no projects".to_string());
            } else {
                context.push(format!("Nexus projects ({}):", projects.len()));
                for p in projects.iter().take(3) {
                    let name = p.name.as_deref().unwrap_or("unnamed");
                    context.push(format!("  - {}", name));
                }
            }
        } else {
            context.push("Nexus: projects unavailable".to_string());
        }

        context
    }

    pub async fn request_improvement(&self, prompt: &str) -> Option<ImprovementResponse> {
        let url = format!("{}/api/mindkernel/improve", self.base_url);
        let resp = match self.client
            .post(&url)
            .json(&serde_json::json!({"prompt": prompt}))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Nexus request failed: {e}");
                return None;
            }
        };
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::info!("Nexus /improve responded with status={status}, body={}", &text[..text.len().min(400)]);
        if let Ok(imp) = serde_json::from_str::<ImprovementResponse>(&text) {
            Some(imp)
        } else if text.contains("error") {
            tracing::warn!("Improve endpoint returned error: {}", &text[..text.len().min(200)]);
            None
        } else {
            tracing::warn!("Improve endpoint returned unparseable response: {}", &text[..text.len().min(200)]);
            None
        }
    }

    async fn fetch_tasks(&self) -> Result<Vec<NexusTask>, reqwest::Error> {
        let url = format!("{}/api/tasks", self.base_url);
        let resp = self.client.get(&url).send().await?;
        let tasks: Vec<NexusTask> = resp.json().await?;
        Ok(tasks)
    }

    async fn fetch_projects(&self) -> Result<Vec<NexusProject>, reqwest::Error> {
        let url = format!("{}/api/agent/projects", self.base_url);
        let resp = self.client.get(&url).send().await?;
        let projects: Vec<NexusProject> = resp.json().await?;
        Ok(projects)
    }
}
// emergent: merge nexus_client check capacity
// emergent: merge nexus_client competence curiosity
// emergent: revert nexus_client integrity novelty
// emergent: split nexus_client check novelty
// emergent: call nexus_client coherence entropy
// emergent: split nexus_client capacity entropy
// emergent: set nexus_client novelty buffer
// emergent: split nexus_client novelty entropy
// emergent: set nexus_client curiosity interval
// emergent: set nexus_client check threshold
// emergent: split nexus_client check capacity
// emergent: split nexus_client capacity competence
// emergent: guard nexus_client capacity check
// emergent: create_operator_create_operator nexus_client limit novelty
// emergent: create_operator_create_operator nexus_client energy check
// emergent: revert nexus_client curiosity energy
// emergent: set nexus_client integrity energy
// emergent: merge nexus_client check novelty
// emergent: revert_copy nexus_client interval mastery
// emergent: revert nexus_client interval check
// emergent: duplicate_set_temp nexus_client curiosity check
// emergent: revert nexus_client rate threshold
// emergent: merge nexus_client entropy rate
// emergent: guard_merge nexus_client rate rate
// emergent: call nexus_client competence curiosity
// emergent: split_duplicate_set_temp nexus_client integrity mastery
// emergent: revert nexus_client mastery limit
// emergent: merge nexus_client capacity capacity
// emergent: revert nexus_client integrity buffer
// emergent: call_split nexus_client coherence entropy
// emergent: set nexus_client integrity rate
// emergent: merge nexus_client rate coherence
// emergent: copy_merge nexus_client threshold rate
// emergent: duplicate_call_split nexus_client limit threshold
// emergent: set_num_candidates_set_kde_threshold_call_split nexus_client capacity capacity
// emergent: call_split nexus_client threshold interval
// emergent: merge nexus_client threshold curiosity
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp nexus_client integrity limit
// emergent: copy nexus_client competence threshold
// emergent: split_duplicate_set_temp nexus_client energy integrity
// emergent: set_temp_merge_call nexus_client competence capacity
// emergent: set_num_candidates_set_kde_threshold_call_split nexus_client energy curiosity
// emergent: duplicate_call_split nexus_client rate integrity
// emergent: set nexus_client capacity limit
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge nexus_client buffer capacity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge nexus_client rate integrity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge nexus_client novelty interval
// emergent: set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp nexus_client capacity mastery
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge nexus_client entropy interval
// emergent: create_operator_create_operator nexus_client entropy limit
// emergent: split_duplicate_set_temp_split_duplicate_set_temp nexus_client energy limit
// emergent: call_split nexus_client integrity buffer
// emergent: merge_call nexus_client novelty integrity
// emergent: call_set_temp nexus_client energy integrity
// emergent: call_set_temp_split nexus_client rate threshold
// emergent: guard nexus_client novelty curiosity
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge nexus_client novelty integrity
// emergent: set_kde_threshold_call_split nexus_client mastery limit
// emergent: merge nexus_client limit capacity
// emergent: split_duplicate_set_temp nexus_client limit rate
// emergent: copy nexus_client threshold integrity
