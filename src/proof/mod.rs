//! EIP-1186 proof generation and verification.
//!
//! Implements Merkle Patricia Trie (MPT) verification for account and storage proofs.

use alloy_primitives::{Bytes, B256, U256};
use alloy_rlp::{Decodable, RlpDecodable};
use eyre::{bail, Result};
use sha3::{Digest, Keccak256};
use tracing::{debug, instrument, warn};

use crate::config::Config;
use crate::types::{ProofData, StorageProofData};

/// RLP-decoded account state.
#[derive(Debug, RlpDecodable)]
struct AccountState {
    nonce: u64,
    balance: U256,
    storage_root: B256,
    code_hash: B256,
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

    /// Verify an account proof against a state root.
    ///
    /// This verifies that the account data (balance, nonce, storage_hash, code_hash)
    /// is correctly included in the Merkle Patricia Trie with the given state root.
    #[instrument(skip(self, proof))]
    pub fn verify_account_proof(&self, state_root: B256, proof: &ProofData) -> Result<bool> {
        debug!(%proof.address, %state_root, "Verifying account proof");

        if proof.account_proof.is_empty() {
            warn!("Empty account proof");
            return Ok(false);
        }

        // The key in the account trie is keccak256(address)
        let key = keccak256(proof.address.as_slice());

        // Verify the Merkle proof
        let value = match verify_mpt_proof(state_root, &key, &proof.account_proof)? {
            Some(v) => v,
            None => {
                // Account doesn't exist - verify it's truly empty
                let nonce_u64: u64 = proof.nonce.to();
                if proof.balance == U256::ZERO
                    && nonce_u64 == 0
                    && proof.code_hash == EMPTY_CODE_HASH
                    && proof.storage_hash == EMPTY_ROOT_HASH
                {
                    debug!("Account verified as non-existent");
                    return Ok(true);
                }
                warn!("Proof shows non-existent account but data is non-empty");
                return Ok(false);
            }
        };

        // Decode the RLP-encoded account state
        let account = match AccountState::decode(&mut value.as_ref()) {
            Ok(a) => a,
            Err(e) => {
                warn!("Failed to decode account RLP: {}", e);
                return Ok(false);
            }
        };

        // Verify account fields match
        let proof_nonce: u64 = proof.nonce.to();
        if account.nonce != proof_nonce {
            warn!(
                expected = proof_nonce,
                got = account.nonce,
                "Nonce mismatch"
            );
            return Ok(false);
        }

        if account.balance != proof.balance {
            warn!(
                expected = %proof.balance,
                got = %account.balance,
                "Balance mismatch"
            );
            return Ok(false);
        }

        if account.storage_root != proof.storage_hash {
            warn!(
                expected = %proof.storage_hash,
                got = %account.storage_root,
                "Storage hash mismatch"
            );
            return Ok(false);
        }

        if account.code_hash != proof.code_hash {
            warn!(
                expected = %proof.code_hash,
                got = %account.code_hash,
                "Code hash mismatch"
            );
            return Ok(false);
        }

        debug!("Account proof verified successfully");
        Ok(true)
    }

    /// Verify a storage proof against a storage root.
    #[instrument(skip(self, proof))]
    pub fn verify_storage_proof(
        &self,
        storage_root: B256,
        proof: &StorageProofData,
    ) -> Result<bool> {
        debug!(%proof.key, %storage_root, "Verifying storage proof");

        if proof.proof.is_empty() {
            // Empty proof is valid for zero value with empty root
            if proof.value == U256::ZERO && storage_root == EMPTY_ROOT_HASH {
                return Ok(true);
            }
            warn!("Empty storage proof for non-zero value");
            return Ok(false);
        }

        // The key in the storage trie is keccak256(slot)
        let key = keccak256(proof.key.as_slice());

        // Verify the Merkle proof
        let value = match verify_mpt_proof(storage_root, &key, &proof.proof)? {
            Some(v) => v,
            None => {
                // Slot doesn't exist - verify value is zero
                if proof.value == U256::ZERO {
                    debug!("Storage slot verified as non-existent (zero value)");
                    return Ok(true);
                }
                warn!("Proof shows non-existent slot but value is non-zero");
                return Ok(false);
            }
        };

        // Decode RLP-encoded storage value
        let decoded_value = decode_storage_value(&value)?;

        if decoded_value != proof.value {
            warn!(
                expected = %proof.value,
                got = %decoded_value,
                "Storage value mismatch"
            );
            return Ok(false);
        }

        debug!("Storage proof verified successfully");
        Ok(true)
    }

    /// Verify a complete account proof including all storage proofs.
    #[instrument(skip(self, proof))]
    pub fn verify_complete_proof(&self, state_root: B256, proof: &ProofData) -> Result<bool> {
        // First verify the account proof
        if !self.verify_account_proof(state_root, proof)? {
            return Ok(false);
        }

        // Then verify each storage proof against the account's storage root
        for storage_proof in &proof.storage_proof {
            if !self.verify_storage_proof(proof.storage_hash, storage_proof)? {
                warn!(key = %storage_proof.key, "Storage proof verification failed");
                return Ok(false);
            }
        }

        debug!("Complete proof verified successfully");
        Ok(true)
    }
}

/// Empty account code hash (keccak256 of empty bytes)
const EMPTY_CODE_HASH: B256 = B256::new([
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);

/// Empty trie root hash (keccak256 of RLP-encoded empty string)
const EMPTY_ROOT_HASH: B256 = B256::new([
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
]);

/// Compute keccak256 hash.
fn keccak256(data: &[u8]) -> B256 {
    B256::from_slice(&Keccak256::digest(data))
}

/// Verify a Merkle Patricia Trie proof.
///
/// Returns the value at the key if it exists, or None for non-existence proofs.
fn verify_mpt_proof(root: B256, key: &B256, proof: &[Bytes]) -> Result<Option<Vec<u8>>> {
    if proof.is_empty() {
        if root == EMPTY_ROOT_HASH {
            return Ok(None);
        }
        bail!("Empty proof for non-empty root");
    }

    // Convert key to nibbles (4-bit values)
    let key_nibbles = bytes_to_nibbles(key.as_slice());
    let mut key_index = 0;

    let mut expected_hash = root;

    for (i, node) in proof.iter().enumerate() {
        // Verify this node's hash matches expected
        let node_hash = keccak256(node);
        if node_hash != expected_hash && node.len() >= 32 {
            bail!(
                "Node hash mismatch at index {}: expected {}, got {}",
                i,
                expected_hash,
                node_hash
            );
        }

        // For nodes < 32 bytes, they can be embedded directly (no hash)
        // This is only valid for the root node or when embedded in a branch
        if node.len() < 32 && i > 0 && node_hash != expected_hash {
            bail!("Invalid embedded node at index {}", i);
        }

        // Decode the RLP list
        let items = decode_rlp_list(node)?;

        match items.len() {
            17 => {
                // Branch node: 16 children + value
                if key_index >= key_nibbles.len() {
                    // We've consumed all key nibbles, return the value at index 16
                    if items[16].is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(items[16].clone()));
                }

                let nibble = key_nibbles[key_index] as usize;
                key_index += 1;

                if items[nibble].is_empty() {
                    // Path doesn't exist
                    return Ok(None);
                }

                // The child reference is either a 32-byte hash or an embedded node
                if items[nibble].len() == 32 {
                    expected_hash = B256::from_slice(&items[nibble]);
                } else if items[nibble].len() < 32 {
                    // Embedded node - should be the next item in proof
                    if i + 1 < proof.len() {
                        expected_hash = keccak256(&proof[i + 1]);
                    } else {
                        // This is an embedded leaf/extension
                        let embedded_items = decode_rlp_list(&items[nibble])?;
                        return process_leaf_extension(&embedded_items, &key_nibbles, key_index);
                    }
                }
            }
            2 => {
                // Leaf or extension node
                let (prefix_nibbles, is_leaf) = decode_hp_path(&items[0])?;

                // Check that the remaining key matches the path
                let remaining_key = &key_nibbles[key_index..];

                if is_leaf {
                    // Leaf node: path should match remaining key exactly
                    if prefix_nibbles != remaining_key {
                        return Ok(None); // Key not in trie
                    }
                    // Return the value
                    return Ok(Some(items[1].clone()));
                } else {
                    // Extension node: path is a prefix of remaining key
                    if remaining_key.len() < prefix_nibbles.len() {
                        return Ok(None);
                    }
                    if &remaining_key[..prefix_nibbles.len()] != prefix_nibbles.as_slice() {
                        return Ok(None);
                    }

                    key_index += prefix_nibbles.len();

                    // Follow to next node
                    if items[1].len() == 32 {
                        expected_hash = B256::from_slice(&items[1]);
                    } else {
                        // Embedded node
                        let embedded_items = decode_rlp_list(&items[1])?;
                        return process_leaf_extension(&embedded_items, &key_nibbles, key_index);
                    }
                }
            }
            _ => bail!("Invalid node with {} items", items.len()),
        }
    }

    // If we get here without returning, the proof is incomplete
    bail!("Incomplete proof - didn't reach a leaf");
}

/// Process a leaf or extension node that might be embedded.
fn process_leaf_extension(
    items: &[Vec<u8>],
    key_nibbles: &[u8],
    key_index: usize,
) -> Result<Option<Vec<u8>>> {
    if items.len() != 2 {
        bail!("Invalid embedded node");
    }

    let (prefix_nibbles, is_leaf) = decode_hp_path(&items[0])?;
    let remaining_key = &key_nibbles[key_index..];

    if is_leaf {
        if prefix_nibbles == remaining_key {
            return Ok(Some(items[1].clone()));
        }
        return Ok(None);
    }

    // Extension node pointing somewhere - shouldn't happen in a valid proof
    bail!("Incomplete proof - extension node at end");
}

/// Decode hex-prefix encoded path.
/// Returns (nibbles, is_leaf).
fn decode_hp_path(encoded: &[u8]) -> Result<(Vec<u8>, bool)> {
    if encoded.is_empty() {
        return Ok((vec![], false));
    }

    let first_nibble = encoded[0] >> 4;
    let is_leaf = first_nibble >= 2;
    let is_odd = first_nibble % 2 == 1;

    let mut nibbles = Vec::new();

    if is_odd {
        // Odd length: first nibble is part of path
        nibbles.push(encoded[0] & 0x0f);
    }

    // Rest of the bytes
    for &byte in &encoded[1..] {
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0f);
    }

    Ok((nibbles, is_leaf))
}

/// Convert bytes to nibbles.
fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0f);
    }
    nibbles
}

/// Decode an RLP list into its items.
fn decode_rlp_list(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    if data.is_empty() {
        return Ok(vec![]);
    }

    let (list_data, _) = decode_rlp_length(data)?;

    let mut items = Vec::new();
    let mut offset = 0;

    while offset < list_data.len() {
        let (item, item_len) = decode_rlp_item(&list_data[offset..])?;
        items.push(item);
        offset += item_len;
    }

    Ok(items)
}

/// Decode RLP length prefix, return (content, total_length).
fn decode_rlp_length(data: &[u8]) -> Result<(&[u8], usize)> {
    if data.is_empty() {
        bail!("Empty RLP data");
    }

    let prefix = data[0];

    if prefix <= 0x7f {
        // Single byte
        Ok((&data[0..1], 1))
    } else if prefix <= 0xb7 {
        // Short string (0-55 bytes)
        let len = (prefix - 0x80) as usize;
        if data.len() < 1 + len {
            bail!("RLP string truncated");
        }
        Ok((&data[1..1 + len], 1 + len))
    } else if prefix <= 0xbf {
        // Long string
        let len_bytes = (prefix - 0xb7) as usize;
        if data.len() < 1 + len_bytes {
            bail!("RLP length truncated");
        }
        let len = bytes_to_usize(&data[1..1 + len_bytes])?;
        if data.len() < 1 + len_bytes + len {
            bail!("RLP string truncated");
        }
        Ok((
            &data[1 + len_bytes..1 + len_bytes + len],
            1 + len_bytes + len,
        ))
    } else if prefix <= 0xf7 {
        // Short list (0-55 bytes)
        let len = (prefix - 0xc0) as usize;
        if data.len() < 1 + len {
            bail!("RLP list truncated");
        }
        Ok((&data[1..1 + len], 1 + len))
    } else {
        // Long list
        let len_bytes = (prefix - 0xf7) as usize;
        if data.len() < 1 + len_bytes {
            bail!("RLP length truncated");
        }
        let len = bytes_to_usize(&data[1..1 + len_bytes])?;
        if data.len() < 1 + len_bytes + len {
            bail!("RLP list truncated");
        }
        Ok((
            &data[1 + len_bytes..1 + len_bytes + len],
            1 + len_bytes + len,
        ))
    }
}

/// Decode a single RLP item, return (content, total_bytes_consumed).
fn decode_rlp_item(data: &[u8]) -> Result<(Vec<u8>, usize)> {
    if data.is_empty() {
        bail!("Empty RLP item");
    }

    let prefix = data[0];

    if prefix <= 0x7f {
        // Single byte value
        Ok((vec![prefix], 1))
    } else if prefix <= 0xb7 {
        // Short string
        let len = (prefix - 0x80) as usize;
        if data.len() < 1 + len {
            bail!("RLP item truncated");
        }
        Ok((data[1..1 + len].to_vec(), 1 + len))
    } else if prefix <= 0xbf {
        // Long string
        let len_bytes = (prefix - 0xb7) as usize;
        if data.len() < 1 + len_bytes {
            bail!("RLP length truncated");
        }
        let len = bytes_to_usize(&data[1..1 + len_bytes])?;
        if data.len() < 1 + len_bytes + len {
            bail!("RLP item truncated");
        }
        Ok((
            data[1 + len_bytes..1 + len_bytes + len].to_vec(),
            1 + len_bytes + len,
        ))
    } else if prefix <= 0xf7 {
        // Short list - return the whole encoded list
        let len = (prefix - 0xc0) as usize;
        if data.len() < 1 + len {
            bail!("RLP list truncated");
        }
        Ok((data[0..1 + len].to_vec(), 1 + len))
    } else {
        // Long list - return the whole encoded list
        let len_bytes = (prefix - 0xf7) as usize;
        if data.len() < 1 + len_bytes {
            bail!("RLP length truncated");
        }
        let len = bytes_to_usize(&data[1..1 + len_bytes])?;
        if data.len() < 1 + len_bytes + len {
            bail!("RLP list truncated");
        }
        Ok((data[0..1 + len_bytes + len].to_vec(), 1 + len_bytes + len))
    }
}

/// Convert big-endian bytes to usize.
fn bytes_to_usize(bytes: &[u8]) -> Result<usize> {
    if bytes.len() > std::mem::size_of::<usize>() {
        bail!("Length too large for usize");
    }
    let mut result: usize = 0;
    for &b in bytes {
        result = result
            .checked_shl(8)
            .ok_or_else(|| eyre::eyre!("Overflow"))?
            | (b as usize);
    }
    Ok(result)
}

/// Decode RLP-encoded storage value to U256.
fn decode_storage_value(data: &[u8]) -> Result<U256> {
    if data.is_empty() {
        return Ok(U256::ZERO);
    }

    // Storage values are RLP-encoded, but might just be raw bytes
    // First check if it looks like RLP
    if data[0] <= 0x7f {
        // Single byte value
        return Ok(U256::from(data[0]));
    }

    if data[0] >= 0x80 && data[0] <= 0xb7 {
        // Short string
        let len = (data[0] - 0x80) as usize;
        if len == 0 {
            return Ok(U256::ZERO);
        }
        if data.len() < 1 + len {
            bail!("Truncated storage value");
        }
        return Ok(U256::from_be_slice(&data[1..1 + len]));
    }

    // Try to interpret as raw bytes (not RLP)
    Ok(U256::from_be_slice(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_nibbles() {
        let bytes = vec![0xab, 0xcd];
        let nibbles = bytes_to_nibbles(&bytes);
        assert_eq!(nibbles, vec![0xa, 0xb, 0xc, 0xd]);
    }

    #[test]
    fn test_decode_hp_path_leaf_even() {
        // Leaf with even path: prefix 0x20
        let encoded = vec![0x20, 0xab, 0xcd];
        let (nibbles, is_leaf) = decode_hp_path(&encoded).unwrap();
        assert!(is_leaf);
        assert_eq!(nibbles, vec![0xa, 0xb, 0xc, 0xd]);
    }

    #[test]
    fn test_decode_hp_path_leaf_odd() {
        // Leaf with odd path: prefix 0x3X where X is first nibble
        let encoded = vec![0x3a, 0xbc];
        let (nibbles, is_leaf) = decode_hp_path(&encoded).unwrap();
        assert!(is_leaf);
        assert_eq!(nibbles, vec![0xa, 0xb, 0xc]);
    }

    #[test]
    fn test_decode_hp_path_extension_even() {
        // Extension with even path: prefix 0x00
        let encoded = vec![0x00, 0xab, 0xcd];
        let (nibbles, is_leaf) = decode_hp_path(&encoded).unwrap();
        assert!(!is_leaf);
        assert_eq!(nibbles, vec![0xa, 0xb, 0xc, 0xd]);
    }

    #[test]
    fn test_empty_root_hash() {
        // Verify the empty root hash constant
        let empty_rlp = hex::decode("80").unwrap();
        let hash = keccak256(&empty_rlp);
        assert_eq!(hash, EMPTY_ROOT_HASH);
    }

    #[test]
    fn test_empty_code_hash() {
        // Verify the empty code hash constant
        let hash = keccak256(&[]);
        assert_eq!(hash, EMPTY_CODE_HASH);
    }
}
