//! Consensus layer integration via Helios light client.

use alloy_primitives::{Address, B256, U256};
use eyre::Result;
use helios_ethereum::{
    config::networks::Network as HeliosNetwork,
    database::ConfigDB,
    EthereumClient, EthereumClientBuilder,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::config::{Config, Network};

pub struct ConsensusClient {
    client: Arc<RwLock<EthereumClient>>,
}

impl ConsensusClient {
    pub async fn new(config: &Config) -> Result<Self> {
        info!(network = %config.ethereum.network, "Initializing Helios light client");

        let network = match config.ethereum.network {
            Network::Mainnet => HeliosNetwork::Mainnet,
            Network::Sepolia => HeliosNetwork::Sepolia,
            Network::Holesky => HeliosNetwork::Holesky,
        };

        let builder: EthereumClientBuilder<ConfigDB> = EthereumClientBuilder::new()
            .network(network)
            .execution_rpc(&config.ethereum.execution_rpc)?
            .consensus_rpc(&config.ethereum.consensus_rpc)?
            .load_external_fallback();

        let builder = if let Some(ref checkpoint) = config.consensus.checkpoint {
            let checkpoint_bytes = hex::decode(checkpoint.trim_start_matches("0x"))?;
            let checkpoint_hash = B256::from_slice(&checkpoint_bytes);
            builder.checkpoint(checkpoint_hash)
        } else {
            builder
        };

        let client = builder.build()?;

        info!("Helios client built");

        Ok(Self {
            client: Arc::new(RwLock::new(client)),
        })
    }

    pub async fn wait_for_sync(&self) -> Result<()> {
        info!("Waiting for Helios sync...");
        let client = self.client.read().await;
        client.wait_synced().await?;
        info!("Helios client synced");
        Ok(())
    }

    pub async fn get_block_number(&self) -> u64 {
        let client = self.client.read().await;
        match client.get_block_number().await {
            Ok(num) => num.to::<u64>(),
            Err(_) => 0,
        }
    }

    pub async fn get_balance(&self, address: Address, block: Option<u64>) -> Result<U256> {
        debug!(%address, ?block, "Getting balance via Helios");
        let client = self.client.read().await;
        let block_id = block.map(|b| b.into()).unwrap_or_default();
        let balance = client.get_balance(address, block_id).await?;
        Ok(balance)
    }

    pub async fn get_storage_at(
        &self,
        address: Address,
        slot: U256,
        block: Option<u64>,
    ) -> Result<B256> {
        debug!(%address, %slot, ?block, "Getting storage via Helios");
        let client = self.client.read().await;
        let block_id = block.map(|b| b.into()).unwrap_or_default();
        let value = client.get_storage_at(address, slot, block_id).await?;
        Ok(value)
    }
}
