//! EIP-1186 proof generation and verification.

use alloy_primitives::{Address, Bytes, B256, U256};
use eyre::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use tracing::{debug, instrument};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountProof {
    pub address: Address,
    pub balance: U256,
    pub code_hash: B256,
    pub nonce: U256,
    pub storage_hash: B256,
    pub account_proof: Vec<Bytes>,
    pub storage_proof: Vec<StorageProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    pub key: B256,
    pub value: U256,
    pub proof: Vec<Bytes>,
}

pub struct ProofGenerator {
    #[allow(dead_code)]
    config: Config,
}

impl ProofGenerator {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_account_proof(
        &self,
        address: Address,
        storage_keys: Vec<B256>,
    ) -> Result<AccountProof> {
        debug!(%address, keys = storage_keys.len(), "Fetching account proof");

        // TODO: fetch via eth_getProof
        Ok(AccountProof {
            address,
            balance: U256::ZERO,
            code_hash: B256::ZERO,
            nonce: U256::ZERO,
            storage_hash: B256::ZERO,
            account_proof: vec![],
            storage_proof: storage_keys
                .into_iter()
                .map(|key| StorageProof {
                    key,
                    value: U256::ZERO,
                    proof: vec![],
                })
                .collect(),
        })
    }

    #[instrument(skip(self, proof))]
    pub fn verify_account_proof(&self, proof: &AccountProof) -> Result<bool> {
        debug!(%proof.address, "Verifying account proof");

        if proof.account_proof.is_empty() {
            return Ok(false);
        }

        // TODO: implement MPT verification
        Ok(true)
    }

    #[instrument(skip(self, proof))]
    pub fn verify_storage_proof(&self, storage_root: B256, proof: &StorageProof) -> Result<bool> {
        debug!(%proof.key, "Verifying storage proof");

        if proof.proof.is_empty() {
            return Ok(false);
        }

        // TODO: implement MPT verification
        Ok(true)
    }

    #[instrument(skip(self))]
    pub async fn get_code_proof(&self, address: Address) -> Result<(Bytes, AccountProof)> {
        debug!(%address, "Fetching code proof");
        let proof = self.get_account_proof(address, vec![]).await?;
        Ok((Bytes::new(), proof))
    }
}

fn keccak256(data: &[u8]) -> B256 {
    B256::from_slice(&Keccak256::digest(data))
}
