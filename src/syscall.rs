pub mod mindkernel {
    include!(concat!(env!("OUT_DIR"), "/mindkernel.rs"));
}

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Mutex;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tonic::{Request, Response, Status};

use crate::meta_cog::MetaCog;
use crate::nexus_client::NexusClient;
use crate::public_apis::PublicApisClient;
use crate::resource_manager::ResourceManager;
use crate::self_modifier::SelfModifier;

pub use mindkernel::ThoughtEvent;
pub use mindkernel::mind_kernel_server::MindKernelServer;

pub struct SyscallHandler {
    pub handlers: Mutex<HashMap<u32, Box<dyn Fn(Vec<u8>) -> Vec<u8> + Send>>>,
}

impl SyscallHandler {
    pub fn new() -> Self {
        SyscallHandler {
            handlers: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, id: u32, handler: Box<dyn Fn(Vec<u8>) -> Vec<u8> + Send>) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.insert(id, handler);
    }

    pub fn handle(&self, id: u32, input: Vec<u8>) -> Result<Vec<u8>, SyscallError> {
        let handlers = self.handlers.lock().unwrap();
        if let Some(handler) = handlers.get(&id) {
            Ok(handler(input))
        } else {
            Err(SyscallError::NotFound(id))
        }
    }
}

#[derive(Debug)]
pub enum SyscallError {
    NotFound(u32),
    ExecutionError(String),
    InvalidInput(String),
}

pub struct MindKernelService {
    pub resource_manager: Arc<ResourceManager>,
    pub meta_cog: Arc<MetaCog>,
    pub self_modifier: Arc<SelfModifier>,
    pub thought_tx: broadcast::Sender<ThoughtEvent>,
    pub nexus_client: NexusClient,
    pub public_apis_client: PublicApisClient,
}

pub struct ThoughtStream {
    rx: broadcast::Receiver<ThoughtEvent>,
}

impl tokio_stream::Stream for ThoughtStream {
    type Item = Result<ThoughtEvent, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.rx.try_recv() {
            Ok(event) => Poll::Ready(Some(Ok(event))),
            Err(broadcast::error::TryRecvError::Empty) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Closed) => Poll::Ready(None),
            Err(broadcast::error::TryRecvError::Lagged(_)) => Poll::Pending,
        }
    }
}

#[tonic::async_trait]
impl mindkernel::mind_kernel_server::MindKernel for MindKernelService {
    async fn tick(
        &self,
        request: Request<mindkernel::TickRequest>,
    ) -> Result<Response<mindkernel::TickResponse>, Status> {
        let req = request.into_inner();
        self.resource_manager.set_energy(req.energy_pct);
        self.resource_manager.set_user_active(req.user_active);

        let resp = mindkernel::TickResponse {
            suggested_interval_sec: self.resource_manager.compute_suggested_interval(),
            energy_pct: self.resource_manager.get_energy(),
            time_budget_sec: self.resource_manager.time_budget_sec(),
            blocked_ideas: self.resource_manager.get_blocked_ideas().await,
            nexus_context: self.nexus_client.fetch_context().await,
        };
        Ok(Response::new(resp))
    }

    async fn propose_modification(
        &self,
        request: Request<mindkernel::ProposeRequest>,
    ) -> Result<Response<mindkernel::ProposeResponse>, Status> {
        let req = request.into_inner();
        let resp = mindkernel::ProposeResponse {
            accepted: req.confidence > 0.5,
            reason: "Evaluated by MindKernel".to_string(),
            measured_improvement: req.confidence,
        };
        Ok(Response::new(resp))
    }

    async fn get_status(
        &self,
        _request: Request<mindkernel::StatusRequest>,
    ) -> Result<Response<mindkernel::StatusResponse>, Status> {
        let blocked = self.resource_manager.get_blocked_ideas().await;
        let resp = mindkernel::StatusResponse {
            energy_pct: self.resource_manager.get_energy(),
            time_budget_sec: self.resource_manager.time_budget_sec(),
            active_ideas: 0,
            blocked_ideas_count: blocked.len() as i32,
            blocked_ideas: blocked
                .into_iter()
                .map(|desc| mindkernel::BlockedIdea {
                    idea_id: String::new(),
                    description: desc,
                    reason: String::new(),
                })
                .collect(),
            entropy: self.meta_cog.get_entropy(),
            improvements_today: self.meta_cog.get_improvements_today(),
            uptime_sec: self.resource_manager.uptime_sec(),
        };
        Ok(Response::new(resp))
    }

    async fn submit_thought(
        &self,
        request: Request<mindkernel::ThoughtRequest>,
    ) -> Result<Response<mindkernel::ThoughtResponse>, Status> {
        let req = request.into_inner();
        let event = ThoughtEvent {
            thought_id: req.thought_id,
            content: req.content,
            thought_type: req.thought_type,
            confidence: req.confidence,
            timestamp: chrono::Utc::now().timestamp(),
        };
        let _ = self.thought_tx.send(event);
        Ok(Response::new(mindkernel::ThoughtResponse { stored: true }))
    }

    type SubscribeThoughtsStream = ThoughtStream;

    async fn subscribe_thoughts(
        &self,
        _request: Request<mindkernel::ThoughtQuery>,
    ) -> Result<Response<Self::SubscribeThoughtsStream>, Status> {
        let rx = self.thought_tx.subscribe();
        Ok(Response::new(ThoughtStream { rx }))
    }

    async fn learn(
        &self,
        _request: Request<mindkernel::LearnRequest>,
    ) -> Result<Response<mindkernel::LearnResponse>, Status> {
        let summary = self.self_modifier.learn_cycle().await;
        let applied = !summary.contains("Failed") && !summary.contains("skipping") && !summary.contains("low");
        Ok(Response::new(mindkernel::LearnResponse {
            improvement_applied: applied,
            summary,
            diff_applied: String::new(),
        }))
    }
}
// emergent: merge syscall limit capacity
// emergent: merge syscall competence competence
// emergent: guard syscall limit buffer
// emergent: call syscall coherence mastery
// emergent: guard syscall interval competence
// emergent: revert syscall entropy entropy
// emergent: duplicate syscall limit capacity
// emergent: set_split syscall integrity check
// emergent: merge syscall capacity novelty
// emergent: set syscall entropy competence
// emergent: copy syscall limit interval
// emergent: guard syscall entropy entropy
// emergent: split syscall buffer novelty
// emergent: duplicate_set_temp syscall entropy entropy
// emergent: split syscall novelty mastery
// emergent: call syscall buffer coherence
// emergent: revert syscall entropy capacity
// emergent: duplicate syscall competence capacity
// emergent: set_kde_threshold_call_split syscall curiosity competence
// emergent: merge_call syscall buffer novelty
// emergent: call_split syscall check entropy
// emergent: split_duplicate_set_temp syscall curiosity capacity
// emergent: call_split syscall curiosity rate
// emergent: split_duplicate_set_temp syscall curiosity capacity
// emergent: split syscall coherence competence
// emergent: set_temp_merge_call syscall curiosity integrity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp syscall novelty limit
// emergent: merge syscall coherence entropy
// emergent: set_num_candidates_set_kde_threshold_call_split syscall curiosity novelty
// emergent: call syscall novelty energy
// emergent: set_kde_threshold_call_split syscall mastery coherence
// emergent: merge syscall energy rate
// emergent: copy_merge syscall capacity mastery
// emergent: split syscall mastery entropy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge syscall curiosity integrity
// emergent: revert syscall capacity check
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge syscall interval mastery
// emergent: set_num_candidates_set_kde_threshold_call_split syscall curiosity competence
// emergent: call_split syscall mastery mastery
// emergent: set syscall mastery integrity
// emergent: set_kde_threshold_call_split syscall entropy energy
// emergent: duplicate_call_split syscall interval curiosity
// emergent: set_num_candidates_set_kde_threshold_call_split syscall integrity competence
// emergent: set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp syscall interval integrity
// emergent: merge syscall interval curiosity
// emergent: call_set_temp syscall threshold entropy
// emergent: split syscall limit interval
// emergent: set_temp_merge_call syscall curiosity entropy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge syscall integrity mastery
// emergent: call_copy syscall novelty threshold
// emergent: revert_copy syscall competence threshold
// emergent: call_set_temp syscall entropy novelty
// emergent: copy syscall energy competence
