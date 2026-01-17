use clap::Parser;
use eyre::Result;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

mod config;
mod consensus;
mod proof;
mod rpc;
mod types;
mod upstream;

use config::Config;

#[derive(Parser, Debug)]
#[command(name = "dvrpc-node")]
#[command(about = "Decentralized Verified RPC node for Ethereum")]
#[command(version)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Override server host
    #[arg(long)]
    host: Option<String>,

    /// Override server port
    #[arg(long)]
    port: Option<u16>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: Level,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(args.log_level.to_string()));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .init();

    info!("Starting DVRPC Node");

    // Load configuration
    let mut config = Config::load(&args.config)?;

    // Apply CLI overrides
    if let Some(host) = args.host {
        config.server.host = host;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }

    info!(
        network = %config.ethereum.network,
        "Configuration loaded"
    );

    // Initialize consensus client
    let consensus_client = if config.consensus.enabled {
        info!("Initializing consensus client");
        let client = consensus::ConsensusClient::new(&config).await?;
        client.wait_for_sync().await?;
        Some(client)
    } else {
        info!("Consensus verification disabled");
        None
    };

    // Initialize proof generator
    let proof_generator = proof::ProofGenerator::new(&config);

    // Build RPC server
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    info!(%addr, "Starting RPC server");

    rpc::serve(addr, config, consensus_client, proof_generator).await?;

    Ok(())
}
