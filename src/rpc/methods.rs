//! JSON-RPC method implementations.

use alloy_primitives::{Address, Bytes, B256, U256, U64};
use eyre::Result;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, instrument};

use super::RpcState;
use crate::proof::AccountProof;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockTag {
    Number(U64),
    Tag(String),
}

impl Default for BlockTag {
    fn default() -> Self {
        BlockTag::Tag("latest".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedBalanceResponse {
    pub balance: U256,
    pub block_number: U64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<AccountProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedStorageResponse {
    pub value: B256,
    pub block_number: U64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<AccountProof>,
}

#[rpc(server, namespace = "eth")]
pub trait DvrpcRpc {
    #[method(name = "getBalance")]
    async fn get_balance(&self, address: Address, block: Option<BlockTag>) -> RpcResult<U256>;

    #[method(name = "getStorageAt")]
    async fn get_storage_at(
        &self,
        address: Address,
        slot: B256,
        block: Option<BlockTag>,
    ) -> RpcResult<B256>;

    #[method(name = "getProof")]
    async fn get_proof(
        &self,
        address: Address,
        storage_keys: Vec<B256>,
        block: Option<BlockTag>,
    ) -> RpcResult<AccountProof>;

    #[method(name = "call")]
    async fn call(&self, tx: CallRequest, block: Option<BlockTag>) -> RpcResult<Bytes>;

    #[method(name = "blockNumber")]
    async fn block_number(&self) -> RpcResult<U64>;

    #[method(name = "chainId")]
    async fn chain_id(&self) -> RpcResult<U64>;
}

#[rpc(server, namespace = "dvrpc")]
pub trait DvrpcExtRpc {
    #[method(name = "getBalance")]
    async fn get_balance_with_proof(
        &self,
        address: Address,
        block: Option<BlockTag>,
    ) -> RpcResult<VerifiedBalanceResponse>;

    #[method(name = "getStorageAt")]
    async fn get_storage_with_proof(
        &self,
        address: Address,
        slot: B256,
        block: Option<BlockTag>,
    ) -> RpcResult<VerifiedStorageResponse>;

    #[method(name = "verifyProof")]
    async fn verify_proof(&self, proof: AccountProof) -> RpcResult<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRequest {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub gas: Option<U64>,
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<U256>,
    pub value: Option<U256>,
    pub data: Option<Bytes>,
}

pub struct DvrpcRpcImpl {
    state: Arc<RpcState>,
}

impl DvrpcRpcImpl {
    pub fn new(state: Arc<RpcState>) -> Self {
        Self { state }
    }
}

#[jsonrpsee::core::async_trait]
impl DvrpcRpcServer for DvrpcRpcImpl {
    #[instrument(skip(self))]
    async fn get_balance(&self, address: Address, block: Option<BlockTag>) -> RpcResult<U256> {
        debug!(%address, ?block, "eth_getBalance");
        // TODO: fetch and verify
        Ok(U256::ZERO)
    }

    #[instrument(skip(self))]
    async fn get_storage_at(
        &self,
        address: Address,
        slot: B256,
        block: Option<BlockTag>,
    ) -> RpcResult<B256> {
        debug!(%address, %slot, ?block, "eth_getStorageAt");
        // TODO: fetch and verify
        Ok(B256::ZERO)
    }

    #[instrument(skip(self))]
    async fn get_proof(
        &self,
        address: Address,
        storage_keys: Vec<B256>,
        block: Option<BlockTag>,
    ) -> RpcResult<AccountProof> {
        debug!(%address, ?storage_keys, ?block, "eth_getProof");
        self.state
            .proof_generator
            .get_account_proof(address, storage_keys)
            .await
            .map_err(|e| ErrorObjectOwned::owned(-32000, e.to_string(), None::<()>))
    }

    #[instrument(skip(self))]
    async fn call(&self, tx: CallRequest, block: Option<BlockTag>) -> RpcResult<Bytes> {
        debug!(?tx, ?block, "eth_call");
        // TODO: execute and verify
        Ok(Bytes::new())
    }

    async fn block_number(&self) -> RpcResult<U64> {
        debug!("eth_blockNumber");
        if let Some(ref consensus) = self.state.consensus {
            Ok(U64::from(consensus.get_block_number().await))
        } else {
            Ok(U64::ZERO)
        }
    }

    async fn chain_id(&self) -> RpcResult<U64> {
        Ok(U64::from(self.state.config.ethereum.chain_id))
    }
}

#[jsonrpsee::core::async_trait]
impl DvrpcExtRpcServer for DvrpcRpcImpl {
    #[instrument(skip(self))]
    async fn get_balance_with_proof(
        &self,
        address: Address,
        block: Option<BlockTag>,
    ) -> RpcResult<VerifiedBalanceResponse> {
        debug!(%address, ?block, "dvrpc_getBalance");

        let balance = self.get_balance(address, block.clone()).await?;
        let proof = self
            .state
            .proof_generator
            .get_account_proof(address, vec![])
            .await
            .ok();

        Ok(VerifiedBalanceResponse {
            balance,
            block_number: U64::ZERO,
            proof,
        })
    }

    #[instrument(skip(self))]
    async fn get_storage_with_proof(
        &self,
        address: Address,
        slot: B256,
        block: Option<BlockTag>,
    ) -> RpcResult<VerifiedStorageResponse> {
        debug!(%address, %slot, ?block, "dvrpc_getStorageAt");

        let value = self.get_storage_at(address, slot, block.clone()).await?;
        let proof = self
            .state
            .proof_generator
            .get_account_proof(address, vec![slot])
            .await
            .ok();

        Ok(VerifiedStorageResponse {
            value,
            block_number: U64::ZERO,
            proof,
        })
    }

    #[instrument(skip(self))]
    async fn verify_proof(&self, proof: AccountProof) -> RpcResult<bool> {
        debug!(?proof.address, "dvrpc_verifyProof");
        self.state
            .proof_generator
            .verify_account_proof(&proof)
            .map_err(|e| ErrorObjectOwned::owned(-32000, e.to_string(), None::<()>))
    }
}

pub fn create_rpc_module(state: Arc<RpcState>) -> Result<RpcModule<()>> {
    let eth_impl = DvrpcRpcImpl::new(state.clone());
    let dvrpc_impl = DvrpcRpcImpl::new(state);

    let mut module = RpcModule::new(());

    module
        .merge(DvrpcRpcServer::into_rpc(eth_impl))
        .map_err(|e| eyre::eyre!("Failed to merge eth RPC: {}", e))?;

    module
        .merge(DvrpcExtRpcServer::into_rpc(dvrpc_impl))
        .map_err(|e| eyre::eyre!("Failed to merge dvrpc RPC: {}", e))?;

    Ok(module)
}
