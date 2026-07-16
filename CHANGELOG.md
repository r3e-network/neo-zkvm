# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fuzz target `fuzz_vm_execution` type error: arguments are now `StackItem::ByteString` (CI fuzz-smoke was broken)
- Legacy `NeoProof` deserialize no longer rejects valid Execute-mode proofs with zero `vkey_hash`
- `prove_strict` rejects execution faults in all modes (not only SP1/Plonk/Groth16)
- CLI `debug` exits non-zero on Fault (parity with `run`)
- Packaging docs/scripts clarify that crates.io publish requires published `neo-vm-rs`
- Workspace `neo-vm-rs` dependency declares a version for packaging metadata
- `docs/api-reference.md` no longer teaches `verify_for_mode(&proof, proof.proof_mode)` (security anti-pattern)
- Prover `build.rs` always overwrites dummy ELF markers (no stale real ELF under force-dummy)
- CLI rejects `.nef` containers instead of treating them as raw script
- CLI default prove mode is `mock` without SP1 feature (was always `sp1`)
- Assembler rejects `CHECKSIG` with a clear guest-unsupported error
- Examples/tests pin `verify_for_mode(..., Mock)` instead of bare `verify`
- `hash_data` / commitment docs corrected (domain-separated zkVM digests, not Neo Hash256)
- Opcode gas docs match runtime metering (1 gas per executed instruction)
- `verify_with_vkey` applies proof size cap and mode/type consistency
- Dropped unused `sp1-build` build-dep from `neo-zkvm-program`
- Privacy demos declare Mock/protocol limitations honestly

### Changed

- Guest execution meters gas at runtime (one unit per executed instruction via `on_instruction`), so loops cannot under-charge relative to static bytecode estimates
- CLI `prove` verifies with `verify_for_mode` pinned to the actual proof mode (never bare attacker-controlled dispatch)
- CLI `run` exits non-zero on Fault for scripting-friendly automation
- CLI argument parsing migrated to `clap` derive (same flags; generated `--help`)
- Deterministic crypto helpers extracted to `neo_vm_guest::crypto` and shared by guest + CLI debug host
- Mock proof serialization is fallible (no panic on encode failure)
- `verify_with_vkey` rejects Mock/Execute proofs (fail closed for succinct vkey verification)
- `neo-zkvm-verifier` depends on `neo-zkvm-prover` only under the optional `sp1` feature
- CLI `debug` accepts `--gas` and meters steps like the proof guest

### Added

- CLI `verify` subcommand for serialized `NeoProof` files (`prove -o` → `verify -m`)
- CLI `-o` / `--output` to write a serialized `NeoProof` after successful prove
- Deterministic guest crypto syscalls: `RIPEMD160`, `Hash160`, `Hash256` (in addition to `SHA256`)
- Assembler `HASH256` mnemonic for `System.Crypto.Hash256`
- Criterion benches in `neo-vm-guest` (`cargo bench -p neo-vm-guest --bench execute`)
- Release operator scripts: `verify-release-metadata.sh`, `verify-packaging.sh`, `release.sh`, `publish-crates.sh`
- Crates.io package metadata (keywords, categories, homepage, documentation) on publishable crates
- `rust-toolchain.toml` documenting the Rust 1.88 floor

### Fixed

- `verify-production.sh` referenced non-existent example binaries (`storage_example`, `native_contracts`); now runs the real example set
- README encoding corruption and stale security guidance
- Hardcoded CLI version string now uses `CARGO_PKG_VERSION`

## [0.2.2] - 2026-02-23

### Changed

- Comprehensive codebase review and production hardening across all crates
- Refactored engine.rs: improved opcode dispatch, error handling, and gas metering
- Refactored native.rs: simplified contract dispatch, reduced code duplication
- Hardened prover: panic-free input hashing, strict fallback policy
- Hardened verifier: enforced proof-mode consistency checks
- Improved assembler/disassembler coverage for full opcode set

### Fixed

- Documentation: updated architecture.md with missing CLI commands (debug, inspect)
- Documentation: corrected SP1_REFACTOR.md status markers (ELF build, precompile, Groth16 now ✅)
- Documentation: fixed whitepaper technical errors, API reference inaccuracies, CLI docs
- Documentation: corrected test counts in RELEASE.md and CHANGELOG.md
- Documentation: fixed opcodes.md gas costs and opcode descriptions
- Stack item serialization edge cases
- VM frame slot initialization for local/argument variables

### Improved

- CI pipeline: added fuzz-smoke, coverage, and cargo-deny audit jobs
- Production verification script with retry logic
- All 417 tests passing, zero clippy warnings, consistent formatting

## [0.2.1] - 2026-02-21

### Added

- Full Neo N3 opcode set implementation
- Storage and native contract syscall dispatch in VM
- Production usage examples with tests
- Assembler/disassembler coverage for all opcodes

### Changed

- Strict prover fallback policy: explicit SP1/PLONK/Groth16 modes fail instead of silently downgrading to mock (use `--allow-fallback` to opt in)
- Verifier mode checks enforce proof-mode consistency
- Prover input hashing is now panic-free (`try_hash_proof_input`)
- Build script fallback behavior hardened for missing SP1 toolchain

### Fixed

- Verifier bound to SP1 public values (was ignoring committed values)
- Prover public inputs aligned with SP1 committed values
- VM script parsing and CALL opcode handling hardened
- VM frame slot initialization for local/argument variables
- Opcode regressions in arithmetic and control flow paths

## [0.2.0] - 2026-01-31

### Added

#### SP1 Integration & Proof Modes

- **Multi-mode proof generation**: Execute, Mock, Sp1, Plonk, Groth16
- **SP1 builder pattern API**: Modern, chainable proof generation interface
- **Automatic fallback**: Falls back to mock mode when SP1 toolchain unavailable
- **SP1 precompiles**: SHA256 acceleration in guest program
- **Compressed proofs**: Support for SP1 compressed proof format

#### Security & Protection

- **Stack depth limiting**: Configurable max stack depth (default: 2048)
- **Invocation depth limiting**: Configurable max invocation depth (default: 1024)
- **New VMError variants**: `StackOverflow(usize)` and `InvocationDepthExceeded(usize)`
- **CALL opcode protection**: Invocation depth check on all call operations

#### Testing

- **18 new boundary tests**: Stack overflow, invocation depth, edge cases
- **310 total tests**: Up from 292 in v0.1.0
- **Enabled doc tests**: Gas metering and error handling examples now run
- **Comprehensive coverage**: All opcodes, error paths, and edge cases

#### Examples

- **Token contract example**: NEP-17 compatible token with transfers
- **AMM swap example**: Automated market maker with slippage protection
- **Multisig wallet example**: Multi-signature wallet implementation
- **Working examples**: All 4 examples fully functional and tested

#### Documentation

- **Production readiness report**: Comprehensive security and quality assessment
- **SP1 refactor documentation**: Architecture and migration guide
- **Getting started guide**: Step-by-step tutorial with working examples
- **API reference**: Complete public API documentation

### Changed

#### API Improvements

- **ProverConfig**: New configuration structure for proof modes
- **ProofMode enum**: Replaced ProveMode with more comprehensive options
- **NeoProver.verify()**: Inherent method for proof verification
- **Depth limits**: Constructor accepts `with_limits()` for custom limits

#### Performance

- **O(1) gas lookup**: Constant-time opcode cost retrieval
- **Pre-allocated vectors**: Default capacity for stack and invocation stacks
- **Inline annotations**: Hot path functions marked for inlining

### Fixed

#### Documentation

- Fixed hex string example in getting started guide
- Fixed `ProofMode` enum name (was `ProveMode`)
- Updated CLI output examples to match actual output
- Fixed proof generation code examples

#### Security

- Added missing invocation depth check to CALL opcode
- Fixed recursive `push()` bug in engine (was calling itself)

#### Code Quality

- All clippy warnings resolved
- Consistent error handling patterns
- Proper error propagation throughout

### Removed

- Legacy `ProveMode` (replaced with `ProofMode`)
- Unused benchmark dependencies (criterion - kept for future use)

## [0.1.0] - 2026-01-29

### Added

- Core VM engine with 100+ Neo N3 opcodes
- Gas metering with configurable limits
- Stack-based execution with overflow protection
- Control flow (jumps, calls, conditionals)
- Arithmetic with overflow checking
- Bitwise operations
- Shared VM semantics through `neo-vm-rs`
- Deterministic proof input/output binding
- ZK proof generation via SP1 integration
- Proof verification
- CLI tools (run, prove, asm, disasm, debug, inspect)
- Comprehensive test suite (417 tests)
- Initial documentation and examples

---

## Release Notes for 0.2.0

### Migration Guide from 0.1.0

#### Proof Mode Usage

```rust
// Before (0.1.0)
use neo_zkvm_prover::{ProverConfig, ProveMode};
let config = ProverConfig {
    prove_mode: ProveMode::Sp1,  // Old enum name
    max_cycles: 1_000_000,
};

// After (0.2.0)
use neo_zkvm_prover::{ProverConfig, ProofMode};
let config = ProverConfig {
    proof_mode: ProofMode::Sp1,  // New enum name
    ..Default::default()         // Uses sensible defaults
};
```

#### Shared Execution

```rust
use neo_vm_guest::{execute, ProofInput};

let output = execute(ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40],
    arguments: vec![],
    gas_limit: 1_000_000,
});
```

### Known Issues

- **SP1 Toolchain**: Optional dependency - project works without it using mock proofs
- **Benchmarks**: Criterion benchmarks exist but require additional setup

### Contributors

Thank you to all contributors who helped make this release possible!

### SHA256 Checksums

```
TBD - will be added during release tagging
```
