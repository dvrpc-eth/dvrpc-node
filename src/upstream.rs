//! Upstream RPC client for fetching proofs from execution layer.

use alloy_primitives::{Address, Bytes, B256, U256, U64};
use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::types::{ProofData, StorageProofData};

pub struct UpstreamClient {
    client: reqwest::Client,
    url: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<T: Serialize> {
    jsonrpc: &'static str,
    method: &'static str,
    params: T,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EthProofResponse {
    address: Address,
    balance: U256,
    code_hash: B256,
    nonce: U64,
    storage_hash: B256,
    account_proof: Vec<Bytes>,
    storage_proof: Vec<StorageProofResponse>,
}

#[derive(Debug, Deserialize)]
struct StorageProofResponse {
    key: B256,
    value: U256,
    proof: Vec<Bytes>,
}

impl UpstreamClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        }
    }

    pub async fn eth_get_proof(
        &self,
        address: Address,
        storage_keys: Vec<B256>,
        block: &str,
    ) -> Result<ProofData> {
        debug!(%address, ?storage_keys, block, "Fetching proof from upstream");

        let params = (address, storage_keys, block);
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method: "eth_getProof",
            params,
            id: 1,
        };

        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .json::<JsonRpcResponse<EthProofResponse>>()
            .await?;

        if let Some(error) = response.error {
            eyre::bail!("Upstream RPC error {}: {}", error.code, error.message);
        }

        let proof = response
            .result
            .ok_or_else(|| eyre::eyre!("No result in upstream response"))?;

        Ok(ProofData {
            address: proof.address,
            balance: proof.balance,
            code_hash: proof.code_hash,
            nonce: proof.nonce,
            storage_hash: proof.storage_hash,
            account_proof: proof.account_proof,
            storage_proof: proof
                .storage_proof
                .into_iter()
                .map(|sp| StorageProofData {
                    key: sp.key,
                    value: sp.value,
                    proof: sp.proof,
                })
                .collect(),
        })
    }

    #[allow(dead_code)]
    pub async fn eth_get_block_by_number(&self, block: &str) -> Result<B256> {
        debug!(block, "Fetching block from upstream");

        let params = (block, false);
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method: "eth_getBlockByNumber",
            params,
            id: 1,
        };

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct BlockResponse {
            state_root: B256,
        }

        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .json::<JsonRpcResponse<BlockResponse>>()
            .await?;

        if let Some(error) = response.error {
            eyre::bail!("Upstream RPC error {}: {}", error.code, error.message);
        }

        let block = response
            .result
            .ok_or_else(|| eyre::eyre!("No result in upstream response"))?;

        Ok(block.state_root)
    }
}
