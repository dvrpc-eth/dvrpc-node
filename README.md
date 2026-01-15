# DVRPC Node

Decentralized Verified RPC node for Ethereum. Serves Ethereum JSON-RPC responses with cryptographic proofs that clients can independently verify.

## Features

- **Verified JSON-RPC**: Standard Ethereum RPC with Merkle proofs (EIP-1186)
- **Light Client Consensus**: Integrates Helios for trustless consensus verification
- **Proof Generation**: Generates state proofs for account balances, storage, and contract code
- **Client Verification**: Responses include all data needed for client-side verification

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      DVRPC Node                             │
├─────────────────────────────────────────────────────────────┤
│  RPC Layer (axum + jsonrpsee)                               │
│  ├── eth_getBalance (+ proof)                               │
│  ├── eth_getStorageAt (+ proof)                             │
│  ├── eth_getProof (EIP-1186)                                │
│  └── eth_call (+ state proof)                               │
├─────────────────────────────────────────────────────────────┤
│  Consensus Layer (Helios)                                   │
│  ├── Light client sync                                      │
│  ├── Header verification                                    │
│  └── State root validation                                  │
├─────────────────────────────────────────────────────────────┤
│  Proof Layer                                                │
│  ├── Merkle Patricia Trie verification                      │
│  ├── Account proof generation                               │
│  └── Storage proof generation                               │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Access to an Ethereum execution client RPC (e.g., Infura, Alchemy)
- Access to a consensus client beacon API

### Build

```bash
cargo build --release
```

### Configure

```bash
cp config.example.toml config.toml
# Edit config.toml with your RPC endpoints
```

### Run

```bash
./target/release/dvrpc-node --config config.toml
```

## Configuration

See `config.example.toml` for all options:

```toml
[server]
host = "127.0.0.1"
port = 8545

[ethereum]
execution_rpc = "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY"
consensus_rpc = "https://www.lightclientdata.org"
network = "mainnet"
```

## API

DVRPC extends standard Ethereum JSON-RPC with proof data:

### `eth_getBalance` (with proof)

```json
{
  "jsonrpc": "2.0",
  "method": "dvrpc_getBalance",
  "params": ["0x...", "latest"],
  "id": 1
}
```

Response includes account proof for independent verification.

### `eth_getProof` (EIP-1186)

```json
{
  "jsonrpc": "2.0",
  "method": "eth_getProof",
  "params": ["0x...", ["0x0"], "latest"],
  "id": 1
}
```

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- --config config.toml

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT License - see [LICENSE](LICENSE)
