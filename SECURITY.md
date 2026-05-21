# Security Policy

## Scope

Security-sensitive code includes:

- `neo-vm-guest` proof input/output serialization and execution wrapper
- `neo-zkvm-prover` proof mode selection and public input construction
- `neo-zkvm-verifier` proof/public-input verification policy
- `neo-zkvm-cli` file parsing and operator-facing proof commands
- Shared execution semantics consumed from `neo-vm-rs`

## Supported Versions

Security updates are handled on the default branch in `r3e-network/neo-zkvm`.

## Reporting

Please report vulnerabilities privately to the project maintainers before public disclosure.

Include:

- Affected commit or release
- Reproduction steps
- Expected and observed behavior
- Impact assessment

## Security Requirements

- Production proof modes must not silently downgrade to mock proofs.
- Public input hashes must bind script, input, output, gas, success state, and verification key.
- Proof execution must use `neo-vm-rs` shared semantics, not a local forked VM engine.
- Host-dependent syscalls must be represented through explicit inputs or deterministic adapters.
- Test fixtures must not contain private keys, production secrets, or live credentials.