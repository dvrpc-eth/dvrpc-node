# Contributing to DVRPC Node

Thank you for your interest in contributing to DVRPC Node! This document provides guidelines for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.75+ (edition 2021)
- Git

### Getting Started

```bash
# Clone the repository
git clone https://github.com/dvrpc-eth/dvrpc-node.git
cd dvrpc-node

# Build the project
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- --config config.example.toml
```

### Docker Development

```bash
# Build and run with docker-compose
docker-compose up --build

# Or build the image directly
docker build -t dvrpc-node .
```

## Code Style

We follow standard Rust conventions enforced by `rustfmt` and `clippy`.

### Before Submitting

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test
```

### Conventions

- Use `rustfmt` default settings
- No `clippy` warnings allowed
- Write descriptive commit messages
- Add tests for new functionality
- Document public APIs with doc comments

## Pull Request Process

1. **Fork** the repository and create your branch from `main`
2. **Name your branch** descriptively: `feature/add-eth-call`, `fix/proof-verification`
3. **Make your changes** with clear, focused commits
4. **Test** your changes thoroughly
5. **Update documentation** if needed
6. **Submit PR** with a clear description of changes

### PR Requirements

- [ ] Code passes `cargo fmt --check`
- [ ] Code passes `cargo clippy -- -D warnings`
- [ ] All tests pass (`cargo test`)
- [ ] New code has appropriate tests
- [ ] Documentation updated if needed

### PR Description Template

```markdown
## Summary
Brief description of changes

## Changes
- Change 1
- Change 2

## Testing
How was this tested?

## Related Issues
Fixes #123
```

## Issue Reporting

### Bug Reports

Include:
- DVRPC Node version
- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs (with `RUST_LOG=debug`)

### Feature Requests

Include:
- Use case description
- Proposed solution (if any)
- Alternatives considered

## Project Structure

```
src/
├── main.rs          # Entry point
├── config.rs        # Configuration handling
├── consensus/       # Helios light client integration
├── proof/           # MPT verification
├── rpc/             # JSON-RPC handlers
├── types.rs         # Shared types
└── upstream.rs      # Upstream RPC client
```

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Use discussions for questions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
