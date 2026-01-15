//! Consensus layer integration via light client.

use alloy_primitives::{Address, B256, U256};
use eyre::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::config::Config;

pub struct ConsensusClient {
    #[allow(dead_code)]
    config: Config,
    state_root: Arc<RwLock<Option<B256>>>,
    block_number: Arc<RwLock<u64>>,
}

impl ConsensusClient {
    pub async fn new(config: &Config) -> Result<Self> {
        info!(network = %config.ethereum.network, "Initializing consensus client");

        Ok(Self {
            config: config.clone(),
            state_root: Arc::new(RwLock::new(None)),
            block_number: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn get_state_root(&self) -> Option<B256> {
        *self.state_root.read().await
    }

    pub async fn get_block_number(&self) -> u64 {
        *self.block_number.read().await
    }

    pub async fn verify_state_root(&self, state_root: B256, block_number: u64) -> Result<bool> {
        debug!(%state_root, block_number, "Verifying state root");
        // TODO: verify against light client
        Ok(true)
    }

    pub async fn get_balance(&self, address: Address, block: Option<u64>) -> Result<U256> {
        debug!(%address, ?block, "Getting balance");
        // TODO: fetch via light client
        Ok(U256::ZERO)
    }

    pub async fn get_storage_at(
        &self,
        address: Address,
        slot: B256,
        block: Option<u64>,
    ) -> Result<B256> {
        debug!(%address, %slot, ?block, "Getting storage");
        // TODO: fetch via light client
        Ok(B256::ZERO)
    }

    pub async fn wait_for_sync(&self, min_block: u64) -> Result<()> {
        info!(min_block, "Waiting for sync");
        Ok(())
    }

    pub async fn is_synced(&self) -> bool {
        false
    }
}
