# Prover Input Hash API Hardening Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove panic behavior from the public `NeoProver::hash_proof_input` API so untrusted oversized inputs cannot crash the prover.

**Architecture:** Convert `hash_proof_input` into a fallible wrapper over `try_hash_proof_input` and update internal tests/callers to handle `Result` explicitly.

**Tech Stack:** Rust, `neo-zkvm-prover`, existing bincode-bound serialization helpers.

### Task 1: Add failing regression test

**Files:**
- Modify: `crates/neo-zkvm-prover/src/lib.rs`

**Step 1: Write failing test**
- Add test that invokes `hash_proof_input` with >10MB argument payload.
- Assert the call is panic-free and returns an error.

**Step 2: Verify RED**
- Run: `cargo test -p neo-zkvm-prover test_hash_proof_input_does_not_panic_on_oversized_arguments`
- Expected: failure before API refactor.

### Task 2: Refactor API to be fallible

**Files:**
- Modify: `crates/neo-zkvm-prover/src/lib.rs`

**Step 1: Change signature**
- Update `hash_proof_input` to return `Result<[u8; 32], bincode::Error>`.

**Step 2: Update tests/callers**
- Adjust existing tests that call `hash_proof_input` to unwrap/expect success where appropriate.

**Step 3: Verify GREEN**
- Run: `cargo test -p neo-zkvm-prover`
- Expected: PASS.

### Task 3: Full verification and commit

**Step 1: Quality gates**
- Run: `cargo fmt --all -- --check`
- Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Run: `bash scripts/verify-production.sh`

**Step 2: Commit**
- Run:
  - `git add crates/neo-zkvm-prover/src/lib.rs docs/plans/2026-02-21-prover-hash-input-fallible.md`
  - `git commit -m "refactor: make prover input hashing panic-free"`
