//! RPC types and response structures.

use alloy_primitives::{Bytes, B256, U256, U64};
use serde::{Deserialize, Serialize};

/// Consensus proof containing state root and sync committee attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsensusProof {
    pub state_root: B256,
    pub slot: u64,
    pub block_number: u64,
}

/// Account proof data (EIP-1186).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofData {
    pub address: alloy_primitives::Address,
    pub balance: U256,
    pub code_hash: B256,
    pub nonce: U64,
    pub storage_hash: B256,
    pub account_proof: Vec<Bytes>,
    pub storage_proof: Vec<StorageProofData>,
}

/// Storage slot proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProofData {
    pub key: B256,
    pub value: U256,
    pub proof: Vec<Bytes>,
}

/// Standard JSON-RPC request.
#[derive(Debug, Clone, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// JSON-RPC response with optional proof extension.
#[derive(Debug, Clone, Serialize)]
pub struct RpcResponse<T: Serialize> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<ProofData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus: Option<ConsensusProof>,
}

/// JSON-RPC error response.
#[derive(Debug, Clone, Serialize)]
pub struct RpcError {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub error: RpcErrorData,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcErrorData {
    pub code: i32,
    pub message: String,
}

impl<T: Serialize> RpcResponse<T> {
    pub fn new(id: serde_json::Value, result: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result,
            proof: None,
            consensus: None,
        }
    }

    pub fn with_proof(mut self, proof: ProofData, consensus: ConsensusProof) -> Self {
        self.proof = Some(proof);
        self.consensus = Some(consensus);
        self
    }
}

impl RpcError {
    pub fn new(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            error: RpcErrorData {
                code,
                message: message.into(),
            },
        }
    }

    pub fn internal(id: serde_json::Value, message: impl Into<String>) -> Self {
        Self::new(id, -32603, message)
    }

    pub fn method_not_found(id: serde_json::Value) -> Self {
        Self::new(id, -32601, "Method not found")
    }

    pub fn invalid_params(id: serde_json::Value, message: impl Into<String>) -> Self {
        Self::new(id, -32602, message)
    }
}
