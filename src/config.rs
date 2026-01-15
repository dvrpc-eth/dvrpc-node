use eyre::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub ethereum: EthereumConfig,
    pub consensus: ConsensusConfig,
    pub proof: ProofConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
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
pub struct ConsensusConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub checkpoint: Option<String>,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProofConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
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
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

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
