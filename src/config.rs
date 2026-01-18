use eyre::{Context, Result};
use serde::Deserialize;
use std::env;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub server: ServerConfig,
    pub ethereum: EthereumConfig,
    pub consensus: ConsensusConfig,
    pub proof: ProofConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EthereumConfig {
    pub network: Network,
    pub execution_rpc: String,
    pub consensus_rpc: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Sepolia,
    Holesky,
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Sepolia => write!(f, "sepolia"),
            Network::Holesky => write!(f, "holesky"),
        }
    }
}

impl std::str::FromStr for Network {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "sepolia" => Ok(Network::Sepolia),
            "holesky" => Ok(Network::Holesky),
            _ => eyre::bail!("Invalid network: {}. Must be mainnet, sepolia, or holesky", s),
        }
    }
}

#[allow(dead_code)]
impl Network {
    pub fn chain_id(&self) -> u64 {
        match self {
            Network::Mainnet => 1,
            Network::Sepolia => 11155111,
            Network::Holesky => 17000,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ConsensusConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub checkpoint: Option<String>,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ProofConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8545
}

fn default_max_connections() -> usize {
    100
}

fn default_chain_id() -> u64 {
    1
}

fn default_true() -> bool {
    true
}

fn default_data_dir() -> String {
    "./data/consensus".to_string()
}

fn default_cache_size() -> usize {
    128
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

impl Config {
    /// Load config from file
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        config.validate()?;

        Ok(config)
    }

    /// Load config from environment variables
    ///
    /// Environment variables:
    /// - DVRPC_HOST: Server host (default: 0.0.0.0)
    /// - DVRPC_PORT: Server port (default: 8545)
    /// - DVRPC_NETWORK: Network name (mainnet, sepolia, holesky)
    /// - DVRPC_EXECUTION_RPC: Execution layer RPC URL
    /// - DVRPC_CONSENSUS_RPC: Consensus layer RPC URL
    /// - DVRPC_CHAIN_ID: Chain ID (default: based on network)
    /// - DVRPC_CONSENSUS_ENABLED: Enable consensus verification (default: true)
    /// - DVRPC_CHECKPOINT: Beacon chain checkpoint hash
    pub fn from_env() -> Result<Self> {
        let network: Network = env::var("DVRPC_NETWORK")
            .unwrap_or_else(|_| "mainnet".to_string())
            .parse()?;

        let chain_id = env::var("DVRPC_CHAIN_ID")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| network.chain_id());

        let config = Config {
            server: ServerConfig {
                host: env::var("DVRPC_HOST").unwrap_or_else(|_| default_host()),
                port: env::var("DVRPC_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_port),
                max_connections: env::var("DVRPC_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_max_connections),
            },
            ethereum: EthereumConfig {
                network,
                execution_rpc: env::var("DVRPC_EXECUTION_RPC")
                    .with_context(|| "DVRPC_EXECUTION_RPC environment variable is required")?,
                consensus_rpc: env::var("DVRPC_CONSENSUS_RPC").unwrap_or_default(),
                chain_id,
            },
            consensus: ConsensusConfig {
                enabled: env::var("DVRPC_CONSENSUS_ENABLED")
                    .map(|s| s.to_lowercase() == "true" || s == "1")
                    .unwrap_or(true),
                checkpoint: env::var("DVRPC_CHECKPOINT").ok(),
                data_dir: env::var("DVRPC_DATA_DIR").unwrap_or_else(|_| default_data_dir()),
            },
            proof: ProofConfig {
                enabled: env::var("DVRPC_PROOF_ENABLED")
                    .map(|s| s.to_lowercase() == "true" || s == "1")
                    .unwrap_or(true),
                cache_size: env::var("DVRPC_CACHE_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_cache_size),
            },
            logging: LoggingConfig {
                level: env::var("DVRPC_LOG_LEVEL").unwrap_or_else(|_| default_log_level()),
                format: env::var("DVRPC_LOG_FORMAT").unwrap_or_else(|_| default_log_format()),
            },
        };

        config.validate()?;

        Ok(config)
    }

    /// Load config from file with environment variable overrides
    pub fn load_with_env(path: &Path) -> Result<Self> {
        let mut config = Self::load(path)?;

        // Override with environment variables if set
        if let Ok(host) = env::var("DVRPC_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = env::var("DVRPC_PORT") {
            if let Ok(p) = port.parse() {
                config.server.port = p;
            }
        }
        if let Ok(execution_rpc) = env::var("DVRPC_EXECUTION_RPC") {
            config.ethereum.execution_rpc = execution_rpc;
        }
        if let Ok(consensus_rpc) = env::var("DVRPC_CONSENSUS_RPC") {
            config.ethereum.consensus_rpc = consensus_rpc;
        }
        if let Ok(checkpoint) = env::var("DVRPC_CHECKPOINT") {
            config.consensus.checkpoint = Some(checkpoint);
        }
        if let Ok(enabled) = env::var("DVRPC_CONSENSUS_ENABLED") {
            config.consensus.enabled = enabled.to_lowercase() == "true" || enabled == "1";
        }

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.ethereum.execution_rpc.is_empty() {
            eyre::bail!("execution_rpc must be configured");
        }

        if self.consensus.enabled && self.ethereum.consensus_rpc.is_empty() {
            eyre::bail!("consensus_rpc must be configured when consensus is enabled");
        }

        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            max_connections: default_max_connections(),
        }
    }
}
