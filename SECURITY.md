# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| < 0.2   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in Neo zkVM, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please email security reports to: **security@neo.org**

Include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested fix (if any)

We aim to acknowledge reports within 48 hours and provide a fix timeline within 7 days.

## Scope

The following components are in scope for security reports:

- VM execution engine (`neo-vm-core`)
- Proof generation and verification (`neo-zkvm-prover`, `neo-zkvm-verifier`)
- Cryptographic operations (`CryptoLib`, hashing, Merkle proofs)
- Storage integrity (`TrackedStorage`, Merkle roots)
- Gas metering correctness

## Out of Scope

- SP1 framework internals (report to [Succinct](https://github.com/succinctlabs/sp1))
- Denial of service via gas exhaustion (by design, gas limits prevent this)

## Security Measures

- All dependencies audited via `cargo-deny` in CI
- Fuzz testing for VM execution and script parsing
- Stack and invocation depth limits to prevent resource exhaustion
- Deterministic execution for proof reproducibility
