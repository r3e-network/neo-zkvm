# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Native contracts (StdLib, CryptoLib)
- Key-value storage with Merkle proofs
- ZK proof generation via SP1 integration
- Proof verification
- CLI tools (run, prove, asm, disasm, debug, inspect)
- Comprehensive test suite (292 tests)
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

#### Stack Depth Limits
```rust
// Default limits (recommended)
let vm = NeoVM::new(1_000_000);

// Custom limits (advanced)
let vm = NeoVM::with_limits(
    1_000_000,     // gas_limit
    2048,          // max_stack_depth
    1024           // max_invocation_depth
);
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
