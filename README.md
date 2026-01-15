# DVRPC Node

> **Work in Progress** - This project is under active development.

Decentralized Verified RPC node for Ethereum.

## Goal

A drop-in replacement for standard Ethereum RPC that returns cryptographically verified responses using light client consensus and EIP-1186 Merkle proofs.

## Status

- [x] Project structure
- [x] Configuration system
- [x] RPC server scaffold
- [ ] Light client integration
- [ ] Proof fetching (eth_getProof)
- [ ] Proof verification (MPT)
- [ ] Execution client integration

## Development

```bash
cargo build
cargo test
cargo run -- --config config.toml
```

## License

MIT
