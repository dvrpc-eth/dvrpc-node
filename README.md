# DVRPC Node

Decentralized Verified RPC node for Ethereum. A drop-in replacement for standard Ethereum RPC that returns cryptographically verified responses using light client consensus and Merkle proofs.

## How It Works

```
Client Request → DVRPC Node → Execution Layer
                     ↓
              Light Client (verified state root)
                     ↓
              MPT Verification (proof against state root)
                     ↓
              Verified Response
```

1. **Light Client**: Syncs with consensus layer to get verified state roots
2. **Proof Fetching**: Fetches EIP-1186 Merkle proofs from execution layer
3. **Verification**: Verifies proofs against light client state root
4. **Response**: Returns data only if verification passes

## Status

- [x] Project structure
- [x] Configuration system
- [x] RPC server
- [x] Light client integration
- [x] Proof fetching (eth_getProof)
- [x] MPT verification
- [ ] P2P network
- [ ] Client SDKs

## Supported Methods

| Method | Status | Verification |
|--------|--------|--------------|
| `eth_getBalance` | ✅ | Account proof |
| `eth_getStorageAt` | ✅ | Account + storage proof |
| `eth_getTransactionCount` | ✅ | Account proof |
| `eth_getProof` | ✅ | Passthrough |
| `eth_blockNumber` | ✅ | Light client |
| `eth_chainId` | ✅ | Config |
| `eth_getCode` | 🚧 | Planned |
| `eth_call` | 🚧 | Planned |

## Quick Start

```bash
# Build
cargo build --release

# Configure (copy and edit)
cp config.example.toml config.toml

# Run
cargo run --release -- --config config.toml
```

### Query

```bash
# Standard query
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045","latest"],"id":1}'

# With proof
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -H "X-DVRPC-Proof: true" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045","latest"],"id":1}'
```

## Documentation

**[DVRPC Documentation](https://github.com/dvrpc-eth/dvrpc-docs)**

## Development

```bash
cargo build
cargo test
RUST_LOG=debug cargo run -- --config config.toml
```

## License

MIT
