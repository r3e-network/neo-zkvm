# Second Production Hardening Pass Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate remaining correctness and compatibility gaps identified in the second audit while keeping behavior stable for existing users.

**Architecture:** Tighten serialization/hash invariants in shared proof utilities, preserve backward compatibility for serialized proofs, and harden assembler behavior with explicit validation and unit tests.

**Tech Stack:** Rust workspace (`cargo`), bincode serialization, CLI assembler/disassembler modules, markdown docs.

### Task 1: Remove lossy proof-output hashing behavior

**Files:**
- Modify: `crates/neo-vm-guest/src/lib.rs`
- Modify: `crates/neo-zkvm-prover/src/lib.rs`
- Test: `crates/neo-vm-guest/src/lib.rs`

**Step 1: Write failing coverage tests**
- Add guest tests that construct oversized `ProofOutput` payloads and assert serialization/hash failure is explicit, not silent.

**Step 2: Run test to verify behavior gap**
- Run: `cargo test -p neo-vm-guest -- --nocapture`
- Expected: at least one new test fails before implementation.

**Step 3: Write minimal implementation**
- Introduce `try_hash_proof_output` and use it where fallible logic is needed.
- Ensure output consistency checks fail safely when output hashing is impossible.

**Step 4: Re-run tests**
- Run: `cargo test -p neo-vm-guest -- --nocapture`
- Expected: PASS.

### Task 2: Restore backward compatibility for serialized NeoProof

**Files:**
- Modify: `crates/neo-vm-guest/src/lib.rs`
- Test: `crates/neo-vm-guest/src/lib.rs`

**Step 1: Write migration test**
- Add a test that serializes a legacy proof shape without `proof_format_version` and deserializes into current `NeoProof`.

**Step 2: Implement compatibility default**
- Add serde default for `proof_format_version` using current format constant.

**Step 3: Re-run tests**
- Run: `cargo test -p neo-vm-guest -- --nocapture`
- Expected: PASS with compatibility preserved.

### Task 3: Harden assembler state and integer encoding safety

**Files:**
- Modify: `crates/neo-zkvm-cli/src/assembler.rs`
- Test: `crates/neo-zkvm-cli/src/assembler.rs`

**Step 1: Write failing unit tests**
- Add tests for:
  - out-of-range `PUSHINT8` should error.
  - state reset between consecutive `assemble` calls.
  - unterminated macro definition should error.

**Step 2: Implement minimal fixes**
- Clear per-assembly mutable state at the start of `assemble`.
- Detect unterminated macro in preprocessing.
- Enforce integer range checks for `PUSHINT8/16/32`.

**Step 3: Re-run tests**
- Run: `cargo test -p neo-zkvm-cli -- --nocapture`
- Expected: PASS.

### Task 4: Align CLI docs with actual gas-limit behavior

**Files:**
- Modify: `docs/cli.md`

**Step 1: Update docs text**
- Remove or correct references implying `NEO_ZKVM_GAS_LIMIT` is currently honored by CLI parsing.

**Step 2: Validate docs compile/tests unaffected**
- Run: `cargo test --doc --all`

### Task 5: Final verification

**Step 1: Run full production checks**
- Run: `bash scripts/test.sh`

**Step 2: Run workspace confidence checks**
- Run: `cargo test --workspace --all-targets`
- Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`

**Step 3: Inspect resulting change set**
- Run: `git status --short`
