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

Please report vulnerabilities privately via GitHub Security Advisories on
[r3e-network/neo-zkvm](https://github.com/r3e-network/neo-zkvm/security/advisories/new)
before public disclosure.

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
- Value-bearing verifiers must pin a succinct mode (`Sp1` / `Plonk` / `Groth16`) with
  `verify_for_mode` / `verify_with_vkey` using a **compile-time constant** expected mode —
  never `proof.proof_mode`. Bare `verify` accepts forgeable Mock/Execute proofs.
- Runtime gas is charged per executed instruction; gas limits must bound adversarial loops.