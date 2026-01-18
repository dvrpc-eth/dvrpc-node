use clap::Parser;
use eyre::Result;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{info, warn, Level};
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
    /// Path to configuration file (optional, uses env vars if not provided)
    #[arg(short, long)]
    config: Option<PathBuf>,

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
    let mut config = match &args.config {
        Some(path) if path.exists() => {
            info!(path = %path.display(), "Loading config from file with env overrides");
            Config::load_with_env(path)?
        }
        Some(path) => {
            return Err(eyre::eyre!("Config file not found: {}", path.display()));
        }
        None => {
            info!("No config file specified, using environment variables");
            Config::from_env()?
        }
    };

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
        match consensus::ConsensusClient::new(&config).await {
            Ok(client) => match client.wait_for_sync().await {
                Ok(()) => {
                    info!("Consensus client synced successfully");
                    Some(client)
                }
                Err(e) => {
                    warn!(
                        "Consensus sync failed: {}. Continuing without consensus verification.",
                        e
                    );
                    warn!("Proofs will still be fetched but not verified against light client.");
                    None
                }
            },
            Err(e) => {
                warn!(
                    "Failed to initialize consensus client: {}. Continuing without consensus.",
                    e
                );
                None
            }
        }
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
