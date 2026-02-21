# Production Hardening Pass Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove high-impact correctness gaps in proof binding/verification, close missing critical tests, and align developer validation with production gates.

**Architecture:** Keep behavior and APIs stable while tightening correctness invariants in prover/verifier. Add focused tests near affected modules and raise local automation to match production checks.

**Tech Stack:** Rust workspace (`cargo`), SP1 SDK integration, GitHub Actions, shell validation scripts.

### Task 1: Fix proof input hash reliability in prover

**Files:**
- Modify: `crates/neo-zkvm-prover/src/lib.rs`
- Test: `crates/neo-zkvm-prover/src/lib.rs`

**Step 1: Write the failing test**
- Add a unit test that builds `ProofInput` with argument payload > 10MB and asserts proving returns an input-serialization error instead of silently succeeding with zero hash.

**Step 2: Run test to verify it fails**
- Run: `cargo test -p neo-zkvm-prover test_prove_rejects_oversized_input_arguments -- --nocapture`
- Expected: FAIL before implementation.

**Step 3: Write minimal implementation**
- Replace lossy hash fallback in `hash_proof_input` with fallible serialization.
- Ensure proving path returns typed error when hashing input fails.

**Step 4: Run test to verify it passes**
- Run: `cargo test -p neo-zkvm-prover test_prove_rejects_oversized_input_arguments -- --nocapture`
- Expected: PASS.

### Task 2: Enforce verifying key binding during verification

**Files:**
- Modify: `crates/neo-zkvm-verifier/src/lib.rs`
- Test: `crates/neo-zkvm-verifier/src/lib.rs`

**Step 1: Write the failing test**
- Add a test that tampers `proof.vkey_hash` and expects verifier rejection with a dedicated key-mismatch error path.

**Step 2: Run test to verify it fails**
- Run: `cargo test -p neo-zkvm-verifier test_verify_rejects_mismatched_vkey_hash -- --nocapture`
- Expected: FAIL before implementation.

**Step 3: Write minimal implementation**
- Compute local verification key hash and compare with `proof.vkey_hash` before cryptographic verify.
- Return a clear error when mismatch is detected.

**Step 4: Run test to verify it passes**
- Run: `cargo test -p neo-zkvm-verifier test_verify_rejects_mismatched_vkey_hash -- --nocapture`
- Expected: PASS.

### Task 3: Close missing CLI/native and guest helper test coverage

**Files:**
- Modify: `crates/neo-zkvm-cli/tests/integration_tests.rs`
- Modify: `crates/neo-vm-guest/src/lib.rs`

**Step 1: Write failing tests**
- Add concrete assertions in `test_native_stdlib_serialize`.
- Add guest helper unit tests for deterministic hashing and commitment behavior.

**Step 2: Run tests to verify failures**
- Run: `cargo test -p neo-zkvm-cli test_native_stdlib_serialize -- --nocapture`
- Run: `cargo test -p neo-vm-guest -- --nocapture`
- Expected: FAIL before implementation where applicable.

**Step 3: Write minimal implementation/supporting changes**
- Implement any missing wiring needed by tests without altering external behavior.

**Step 4: Run tests to verify pass**
- Re-run above commands.

### Task 4: Align local validation script with production checks

**Files:**
- Modify: `scripts/test.sh`

**Step 1: Write expected command sequence**
- Add `cargo fmt --all -- --check` and `cargo deny check advisories --show-stats` to match production validation intent.

**Step 2: Validate script behavior**
- Run: `bash scripts/test.sh`
- Expected: All checks run in order and pass.

### Task 5: Verification and regression sweep

**Files:**
- Modify: `docs/plans/2026-02-19-production-hardening-pass.md` (mark execution notes if needed)

**Step 1: Run targeted workspace checks**
- Run: `cargo test --workspace --all-targets`
- Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`

**Step 2: Confirm no unintended changes**
- Run: `git status --short`
- Ensure only intended files changed.
