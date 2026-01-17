//! RPC method handlers.

use alloy_primitives::{Address, B256, U256, U64};
use tracing::{debug, error};

use super::AppState;
use crate::types::{RpcError, RpcRequest, RpcResponse};

/// Parse address and block tag from params.
fn parse_address_block(params: &serde_json::Value) -> Result<(Address, String), String> {
    let params = params
        .as_array()
        .ok_or_else(|| "params must be an array".to_string())?;

    if params.is_empty() {
        return Err("missing address parameter".to_string());
    }

    let address: Address = serde_json::from_value(params[0].clone())
        .map_err(|e| format!("invalid address: {}", e))?;

    let block = params
        .get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("latest")
        .to_string();

    Ok((address, block))
}

/// eth_getBalance - Get account balance with optional proof.
pub async fn eth_get_balance(
    state: &AppState,
    request: &RpcRequest,
    include_proof: bool,
) -> serde_json::Value {
    let (address, block) = match parse_address_block(&request.params) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::to_value(RpcError::invalid_params(request.id.clone(), e)).unwrap()
        }
    };

    debug!(%address, %block, include_proof, "eth_getBalance");

    // Fetch proof from upstream
    let proof_data = match state.upstream.eth_get_proof(address, vec![], &block).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch proof: {}", e);
            return serde_json::to_value(RpcError::internal(
                request.id.clone(),
                format!("Failed to fetch proof: {}", e),
            ))
            .unwrap();
        }
    };

    // Get consensus proof if we have a consensus client
    let consensus_proof = if let Some(ref consensus) = state.consensus {
        match consensus.get_consensus_proof().await {
            Ok(cp) => Some(cp),
            Err(e) => {
                error!("Failed to get consensus proof: {}", e);
                None
            }
        }
    } else {
        None
    };

    // TODO: Verify proof against consensus state root
    // if let Some(ref cp) = consensus_proof {
    //     verify_account_proof(&proof_data, cp.state_root)?;
    // }

    let balance = proof_data.balance;

    if include_proof {
        if let Some(cp) = consensus_proof {
            let response = RpcResponse::new(request.id.clone(), balance).with_proof(proof_data, cp);
            serde_json::to_value(response).unwrap()
        } else {
            serde_json::to_value(RpcResponse::new(request.id.clone(), balance)).unwrap()
        }
    } else {
        serde_json::to_value(RpcResponse::new(request.id.clone(), balance)).unwrap()
    }
}

/// eth_getStorageAt - Get storage value with optional proof.
pub async fn eth_get_storage_at(
    state: &AppState,
    request: &RpcRequest,
    include_proof: bool,
) -> serde_json::Value {
    let params = match request.params.as_array() {
        Some(p) => p,
        None => {
            return serde_json::to_value(RpcError::invalid_params(
                request.id.clone(),
                "params must be an array",
            ))
            .unwrap()
        }
    };

    if params.len() < 2 {
        return serde_json::to_value(RpcError::invalid_params(
            request.id.clone(),
            "missing parameters",
        ))
        .unwrap();
    }

    let address: Address = match serde_json::from_value(params[0].clone()) {
        Ok(a) => a,
        Err(e) => {
            return serde_json::to_value(RpcError::invalid_params(
                request.id.clone(),
                format!("invalid address: {}", e),
            ))
            .unwrap()
        }
    };

    let slot: B256 = match serde_json::from_value(params[1].clone()) {
        Ok(s) => s,
        Err(e) => {
            return serde_json::to_value(RpcError::invalid_params(
                request.id.clone(),
                format!("invalid slot: {}", e),
            ))
            .unwrap()
        }
    };

    let block = params
        .get(2)
        .and_then(|v| v.as_str())
        .unwrap_or("latest")
        .to_string();

    debug!(%address, %slot, %block, include_proof, "eth_getStorageAt");

    // Fetch proof with storage key
    let proof_data = match state
        .upstream
        .eth_get_proof(address, vec![slot], &block)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch proof: {}", e);
            return serde_json::to_value(RpcError::internal(
                request.id.clone(),
                format!("Failed to fetch proof: {}", e),
            ))
            .unwrap();
        }
    };

    let consensus_proof = if let Some(ref consensus) = state.consensus {
        consensus.get_consensus_proof().await.ok()
    } else {
        None
    };

    // Extract storage value
    let value = proof_data
        .storage_proof
        .first()
        .map(|sp| sp.value)
        .unwrap_or(U256::ZERO);

    // Convert U256 to B256 for storage response
    let value_b256 = B256::from(value);

    if include_proof {
        if let Some(cp) = consensus_proof {
            let response =
                RpcResponse::new(request.id.clone(), value_b256).with_proof(proof_data, cp);
            serde_json::to_value(response).unwrap()
        } else {
            serde_json::to_value(RpcResponse::new(request.id.clone(), value_b256)).unwrap()
        }
    } else {
        serde_json::to_value(RpcResponse::new(request.id.clone(), value_b256)).unwrap()
    }
}

/// eth_getTransactionCount - Get account nonce with optional proof.
pub async fn eth_get_transaction_count(
    state: &AppState,
    request: &RpcRequest,
    include_proof: bool,
) -> serde_json::Value {
    let (address, block) = match parse_address_block(&request.params) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::to_value(RpcError::invalid_params(request.id.clone(), e)).unwrap()
        }
    };

    debug!(%address, %block, include_proof, "eth_getTransactionCount");

    let proof_data = match state.upstream.eth_get_proof(address, vec![], &block).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch proof: {}", e);
            return serde_json::to_value(RpcError::internal(
                request.id.clone(),
                format!("Failed to fetch proof: {}", e),
            ))
            .unwrap();
        }
    };

    let consensus_proof = if let Some(ref consensus) = state.consensus {
        consensus.get_consensus_proof().await.ok()
    } else {
        None
    };

    let nonce = proof_data.nonce;

    if include_proof {
        if let Some(cp) = consensus_proof {
            let response = RpcResponse::new(request.id.clone(), nonce).with_proof(proof_data, cp);
            serde_json::to_value(response).unwrap()
        } else {
            serde_json::to_value(RpcResponse::new(request.id.clone(), nonce)).unwrap()
        }
    } else {
        serde_json::to_value(RpcResponse::new(request.id.clone(), nonce)).unwrap()
    }
}

/// eth_getCode - Get contract code with optional proof.
pub async fn eth_get_code(
    _state: &AppState,
    request: &RpcRequest,
    _include_proof: bool,
) -> serde_json::Value {
    let (address, block) = match parse_address_block(&request.params) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::to_value(RpcError::invalid_params(request.id.clone(), e)).unwrap()
        }
    };

    debug!(%address, %block, "eth_getCode");

    // TODO: Implement code fetching with proof
    // For now, return empty code
    serde_json::to_value(RpcResponse::new(
        request.id.clone(),
        alloy_primitives::Bytes::new(),
    ))
    .unwrap()
}

/// eth_getProof - Standard EIP-1186 proof response.
pub async fn eth_get_proof(state: &AppState, request: &RpcRequest) -> serde_json::Value {
    let params = match request.params.as_array() {
        Some(p) => p,
        None => {
            return serde_json::to_value(RpcError::invalid_params(
                request.id.clone(),
                "params must be an array",
            ))
            .unwrap()
        }
    };

    if params.len() < 2 {
        return serde_json::to_value(RpcError::invalid_params(
            request.id.clone(),
            "missing parameters",
        ))
        .unwrap();
    }

    let address: Address = match serde_json::from_value(params[0].clone()) {
        Ok(a) => a,
        Err(e) => {
            return serde_json::to_value(RpcError::invalid_params(
                request.id.clone(),
                format!("invalid address: {}", e),
            ))
            .unwrap()
        }
    };

    let storage_keys: Vec<B256> = match serde_json::from_value(params[1].clone()) {
        Ok(k) => k,
        Err(e) => {
            return serde_json::to_value(RpcError::invalid_params(
                request.id.clone(),
                format!("invalid storage keys: {}", e),
            ))
            .unwrap()
        }
    };

    let block = params
        .get(2)
        .and_then(|v| v.as_str())
        .unwrap_or("latest")
        .to_string();

    debug!(%address, ?storage_keys, %block, "eth_getProof");

    let proof_data = match state
        .upstream
        .eth_get_proof(address, storage_keys, &block)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch proof: {}", e);
            return serde_json::to_value(RpcError::internal(
                request.id.clone(),
                format!("Failed to fetch proof: {}", e),
            ))
            .unwrap();
        }
    };

    serde_json::to_value(RpcResponse::new(request.id.clone(), proof_data)).unwrap()
}

/// eth_blockNumber - Get current block number.
pub async fn eth_block_number(state: &AppState, request: &RpcRequest) -> serde_json::Value {
    let block_number = if let Some(ref consensus) = state.consensus {
        consensus.get_block_number().await
    } else {
        0
    };

    serde_json::to_value(RpcResponse::new(
        request.id.clone(),
        U64::from(block_number),
    ))
    .unwrap()
}

/// eth_chainId - Get chain ID.
pub async fn eth_chain_id(state: &AppState, request: &RpcRequest) -> serde_json::Value {
    serde_json::to_value(RpcResponse::new(
        request.id.clone(),
        U64::from(state.config.ethereum.chain_id),
    ))
    .unwrap()
}
