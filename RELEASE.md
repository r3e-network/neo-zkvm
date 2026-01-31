# Neo zkVM v0.2.0 Release Notes

**Release Date:** 2026-01-31  
**Version:** 0.2.0  
**Status:** Production Ready

---

## 🎯 Highlights

This release brings significant improvements to Neo zkVM, focusing on:

- **Multi-Mode Proof Generation** - Choose the right proof type for your use case
- **Enhanced Security** - Stack and invocation depth protection
- **Production Hardening** - 310 tests, comprehensive documentation, security review

---

## 🚀 New Features

### Multi-Mode Proof Generation

Choose from 5 proof modes based on your needs:

| Mode | Speed | Use Case |
|------|-------|----------|
| `Execute` | Instant | Development, debugging |
| `Mock` | Fast | Testing, CI/CD |
| `Sp1` | Slow | Off-chain verification (compressed) |
| `Plonk` | Slowest | On-chain verification (Ethereum) |
| `Groth16` | Slowest | On-chain verification (smallest proof) |

```rust
use neo_zkvm_prover::{NeoProver, ProverConfig, ProofMode};

// For production Ethereum verification
let prover = NeoProver::new(ProverConfig {
    proof_mode: ProofMode::Groth16,
    ..Default::default()
});
```

### Security Enhancements

- **Stack Depth Limit**: Configurable maximum stack depth (default: 2048)
- **Invocation Depth Limit**: Configurable maximum call depth (default: 1024)
- **CALL Opcode Protection**: All recursive calls are depth-limited

```rust
// Default limits (recommended)
let vm = NeoVM::new(1_000_000);

// Custom limits
let vm = NeoVM::with_limits(
    1_000_000,  // gas_limit
    2048,       // max_stack_depth
    1024        // max_invocation_depth
);
```

---

## 📊 Test Coverage

- **310 tests** (up from 292 in v0.1.0)
- **100% opcode coverage**
- **Edge case testing** for all arithmetic, bitwise, and control flow operations
- **Security tests** for overflow, underflow, and depth limits

```bash
$ cargo test --all
running 310 tests
test result: ok. 310 passed; 0 failed; 0 ignored
```

---

## 📚 Documentation

- **Getting Started Guide**: Step-by-step tutorial
- **API Reference**: Complete public API documentation
- **Architecture Guide**: Deep dive into system design
- **Production Readiness Report**: Security and quality assessment
- **Migration Guide**: From v0.1.0 to v0.2.0

---

## 🛠️ CLI Improvements

```bash
# Run a script
neo-zkvm run 12139E40

# Generate proof
neo-zkvm prove 12139E40

# Interactive debugger
neo-zkvm debug 12139E40

# Disassemble
neo-zkvm disasm 12139E40

# Assemble
neo-zkvm asm "PUSH2 PUSH3 ADD RET"
```

---

## 📦 Installation

### From Source

```bash
git clone https://github.com/neo-project/neo-zkvm
cd neo-zkvm
cargo build --release
```

### CLI Tool

```bash
cargo install --path crates/neo-zkvm-cli
neo-zkvm --version
```

### As Dependency

```toml
[dependencies]
neo-vm-core = "0.2"
neo-zkvm-prover = "0.2"
neo-zkvm-verifier = "0.2"
```

---

## 🔄 Migration from v0.1.0

### Proof Mode Changes

```rust
// Before
use neo_zkvm_prover::{ProverConfig, ProveMode};

// After
use neo_zkvm_prover::{ProverConfig, ProofMode};
```

### API Changes

```rust
// Before
let config = ProverConfig {
    prove_mode: ProveMode::Sp1,
    max_cycles: 1_000_000,
};

// After
let config = ProverConfig {
    proof_mode: ProofMode::Sp1,
    ..Default::default()
};
```

---

## 🐛 Bug Fixes

- Fixed recursive `push()` bug that caused stack overflow
- Fixed documentation examples
- Added missing invocation depth check to CALL opcode

---

## 📈 Performance

- O(1) gas cost lookup
- Pre-allocated vectors for hot paths
- SP1 precompiles for SHA256 (100x faster)

---

## 🔒 Security

| Feature | Status |
|---------|--------|
| Integer overflow protection | ✅ |
| Stack depth limiting | ✅ |
| Invocation depth limiting | ✅ |
| Gas metering | ✅ |
| Input validation | ✅ |
| Comprehensive test coverage | ✅ |

---

## 📁 Files

```
neo-zkvm/
├── crates/
│   ├── neo-vm-core/       # Core VM engine
│   ├── neo-vm-guest/      # Guest I/O types
│   ├── neo-zkvm-prover/   # SP1 prover
│   ├── neo-zkvm-verifier/ # SP1 verifier
│   ├── neo-zkvm-program/  # Guest ELF
│   ├── neo-zkvm-cli/      # CLI tool
│   └── neo-zkvm-examples/ # Usage examples
├── docs/                  # Documentation
├── examples/              # More examples
└── tests/                 # Integration tests
```

---

## 🙏 Contributors

Thank you to all contributors who helped make this release possible!

---

## 🔗 Links

- **Repository**: https://github.com/neo-project/neo-zkvm
- **Documentation**: https://github.com/neo-project/neo-zkvm/tree/main/docs
- **Issues**: https://github.com/neo-project/neo-zkvm/issues
- **Changelog**: https://github.com/neo-project/neo-zkvm/blob/main/CHANGELOG.md

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

---

## 🎉 Thank You!

Thank you for using Neo zkVM! We look forward to your feedback and contributions.

**Full Changelog**: https://github.com/neo-project/neo-zkvm/blob/main/CHANGELOG.md
