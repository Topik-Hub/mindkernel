mod emergent;
mod llm_tool;
mod sandbox;
mod self_applier;
mod lambda_function;
mod macro_drives;
mod micro_drives;
mod meta_cog;
mod nexus_client;
mod public_apis;
mod resource_manager;
mod self_model;
mod self_modifier;
mod stochastic_clock;
mod syscall;

use std::sync::Arc;
use tokio::sync::broadcast;
use tonic::transport::Server;
use tracing::info;

use crate::meta_cog::MetaCog;
use crate::nexus_client::NexusClient;
use crate::public_apis::PublicApisClient;
use crate::resource_manager::ResourceManager;
use crate::self_modifier::SelfModifier;
use crate::syscall::mindkernel::mind_kernel_server::MindKernelServer;
use crate::syscall::mindkernel::ThoughtEvent;
use crate::syscall::MindKernelService;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_writer(std::io::stdout).init();

    let (thought_tx, _) = broadcast::channel::<ThoughtEvent>(1024);

    let resource_manager = Arc::new(ResourceManager::new(3600.0));
    let meta_cog = Arc::new(MetaCog::new());

    let nexus_base_url = std::env::var("NEXUS_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
    let nexus_client = Arc::new(NexusClient::new(&nexus_base_url));

    let source_dir = std::env::var("MINDKERNEL_SRC_DIR")
        .unwrap_or_else(|_| "D:\\mindkernel".to_string());

    let self_modifier = Arc::new(SelfModifier::new(
        resource_manager.clone(),
        nexus_client.clone(),
        &source_dir,
    ));

    let public_apis_client = PublicApisClient::new();

    let life_clock_rm = resource_manager.clone();
    let life_clock_nexus = nexus_client.clone();
    let life_clock_src = source_dir.clone();
    tokio::spawn(async move {
        stochastic_clock::run_life_cycle(life_clock_rm, life_clock_nexus, &life_clock_src).await;
    });

    let service = MindKernelService {
        resource_manager: resource_manager.clone(),
        meta_cog: meta_cog.clone(),
        self_modifier: self_modifier.clone(),
        thought_tx: thought_tx.clone(),
        nexus_client: Arc::unwrap_or_clone(nexus_client.clone()),
        public_apis_client,
    };

    let addr = "0.0.0.0:50051".parse()?;
    info!("MindKernel gRPC server starting on {}", addr);

    Server::builder()
        .add_service(MindKernelServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
