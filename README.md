# DVRPC Node

Decentralized Verified RPC node for Ethereum. Drop-in replacement for standard RPC endpoints that returns cryptographically verified responses.

## What It Does

- **Drop-in RPC replacement**: Swap your RPC URL, get verified responses
- **Standard eth_* methods**: Full compatibility with existing tooling
- **EIP-1186 proofs**: Merkle proofs for account state and storage
- **Pluggable consensus**: Configurable light client backend for state root verification

## Quick Start

```bash
cargo build --release
cp config.example.toml config.toml
./target/release/dvrpc-node --config config.toml
```

## Configuration

```toml
[server]
host = "127.0.0.1"
port = 8545

[ethereum]
network = "mainnet"
execution_rpc = "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY"
consensus_rpc = "https://www.lightclientdata.org"

[consensus]
enabled = true
```

## API

### Standard Methods (drop-in compatible)

### Extended Methods (with proofs)

## Development

```bash
cargo test
cargo fmt
cargo clippy
RUST_LOG=debug cargo run -- --config config.toml
```

## License

MIT
