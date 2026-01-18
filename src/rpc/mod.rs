//! JSON-RPC server with optional proof responses.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use eyre::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info};

use crate::config::Config;
use crate::consensus::ConsensusClient;
use crate::proof::ProofGenerator;
use crate::types::{RpcError, RpcRequest};
use crate::upstream::UpstreamClient;

mod handlers;

/// Shared application state.
pub struct AppState {
    pub config: Config,
    pub consensus: Option<ConsensusClient>,
    pub proof_generator: ProofGenerator,
    pub upstream: UpstreamClient,
}

/// Check if X-DVRPC-Proof header is set to true.
fn wants_proof(headers: &HeaderMap) -> bool {
    headers
        .get("X-DVRPC-Proof")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Main RPC handler - routes to method-specific handlers.
async fn rpc_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RpcRequest>,
) -> impl IntoResponse {
    let include_proof = wants_proof(&headers);
    debug!(method = %request.method, include_proof, "RPC request");

    let response = match request.method.as_str() {
        "eth_getBalance" => handlers::eth_get_balance(&state, &request, include_proof).await,
        "eth_getStorageAt" => handlers::eth_get_storage_at(&state, &request, include_proof).await,
        "eth_getTransactionCount" => {
            handlers::eth_get_transaction_count(&state, &request, include_proof).await
        }
        "eth_getCode" => handlers::eth_get_code(&state, &request, include_proof).await,
        "eth_getProof" => handlers::eth_get_proof(&state, &request).await,
        "eth_blockNumber" => handlers::eth_block_number(&state, &request).await,
        "eth_chainId" => handlers::eth_chain_id(&state, &request).await,
        _ => {
            error!(method = %request.method, "Method not found");
            serde_json::to_value(RpcError::method_not_found(request.id)).unwrap()
        }
    };

    (StatusCode::OK, Json(response))
}

/// Health check endpoint.
async fn health_handler() -> &'static str {
    "OK"
}

/// Start the RPC server.
pub async fn serve(
    addr: SocketAddr,
    config: Config,
    consensus: Option<ConsensusClient>,
    proof_generator: ProofGenerator,
) -> Result<()> {
    let upstream = UpstreamClient::new(&config.ethereum.execution_rpc);

    let state = Arc::new(AppState {
        config,
        consensus,
        proof_generator,
        upstream,
    });

    let app = Router::new()
        .route("/", post(rpc_handler))
        .route("/health", axum::routing::get(health_handler))
        .layer(CorsLayer::new().allow_origin(Any).allow_headers(Any))
        .with_state(state);

    info!(%addr, "RPC server starting");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("RPC server stopped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
    info!("Shutdown signal received");
}
