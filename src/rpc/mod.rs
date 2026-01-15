//! JSON-RPC server.

use axum::{routing::get, Router};
use eyre::Result;
use jsonrpsee::server::ServerBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod methods;

use crate::config::Config;
use crate::consensus::ConsensusClient;
use crate::proof::ProofGenerator;

pub struct RpcState {
    pub config: Config,
    pub consensus: Option<ConsensusClient>,
    pub proof_generator: ProofGenerator,
}

pub async fn serve(
    addr: SocketAddr,
    config: Config,
    consensus: Option<ConsensusClient>,
    proof_generator: ProofGenerator,
) -> Result<()> {
    let state = Arc::new(RpcState {
        config,
        consensus,
        proof_generator,
    });

    let rpc_module = methods::create_rpc_module(state.clone())?;

    let server = ServerBuilder::default()
        .build(addr)
        .await
        .map_err(|e| eyre::eyre!("Failed to build RPC server: {}", e))?;

    let handle = server.start(rpc_module);

    let app = Router::new()
        .route("/health", get(health_handler))
        .layer(CorsLayer::new().allow_origin(Any));

    let health_addr: SocketAddr = format!("{}:{}", state.config.server.host, state.config.server.port + 1)
        .parse()
        .unwrap();

    info!(%health_addr, "Health endpoint");

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(health_addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    info!(%addr, "RPC server started");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down");

    handle.stop()?;

    Ok(())
}

async fn health_handler() -> &'static str {
    "OK"
}
