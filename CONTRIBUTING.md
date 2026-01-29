# Contributing to Neo zkVM

Thank you for your interest in contributing to Neo zkVM!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/neo-zkvm`
3. Create a branch: `git checkout -b feature/your-feature`

## Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run tests
cargo test --all

# Run clippy
cargo clippy --all
```

## Code Style

- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Add tests for new features
- Update documentation as needed

## Pull Request Process

1. Update README.md if needed
2. Add tests for new functionality
3. Ensure CI passes
4. Request review from maintainers

## Reporting Issues

- Use GitHub Issues
- Include reproduction steps
- Provide system information

## License

By contributing, you agree that your contributions will be licensed under MIT.
