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

# Run core production checks (fmt + clippy + tests + dependency policy)
bash scripts/test.sh

# Run full production readiness checks
bash scripts/verify-production.sh
```

## Code Style

- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Ensure `cargo deny check licenses` passes before opening a PR
- Add tests for new features
- Update documentation as needed

## Local Verification Notes

- In some environments, `sp1-prover` build-time VK download can timeout and cause otherwise-correct test runs to fail. For reliable local verification, run tests with:
  - `DOCS_RS=1 cargo test --workspace`
- Some `clippy` runs that pull SP1 native crates can fail locally when static SP1 sys libraries are unavailable (for example `sp1-core-machine-sys` / `sp1-recursion-core-sys`).
  - If this happens, run clippy on directly modified crates that do not depend on those native artifacts and rely on CI/full toolchain environments for workspace-wide SP1 clippy coverage.

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
