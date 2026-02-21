# Proof Mode Strictness and Verifier Policy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate silent cryptographic proof fallback by default and add verifier APIs that enforce caller-expected proof mode.

**Architecture:** Introduce an explicit fallback policy in `ProverConfig` so cryptographic mode failures return failed proofs unless fallback is explicitly enabled. Add new verifier entry points that validate `proof.proof_mode` against an expected mode before cryptographic verification.

**Tech Stack:** Rust, cargo test, clippy, existing neo-zkvm crates.

### Task 1: Add failing prover tests (fallback policy)

**Files:**
- Modify: `crates/neo-zkvm-prover/src/lib.rs`
- Test: `crates/neo-zkvm-prover/src/lib.rs` (unit tests)

**Step 1: Write failing test for strict default behavior in crypto mode**

```rust
#[test]
fn test_prove_crypto_mode_without_fallback_returns_failed_proof_in_debug() {
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Sp1,
        allow_mock_fallback: false,
        ..Default::default()
    });

    let input = ProofInput {
        script: vec![0x12, 0x13, 0x9E, 0x40],
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let proof = prover.prove(input);
    if cfg!(debug_assertions) {
        assert_ne!(proof.output.state, 0);
        assert_eq!(proof.proof_mode, ProofMode::Sp1);
    }
}
```

**Step 2: Run test to verify RED**

Run: `cargo test -p neo-zkvm-prover test_prove_crypto_mode_without_fallback_returns_failed_proof_in_debug`
Expected: FAIL because prover still falls back to mock.

**Step 3: Write failing test for explicit fallback opt-in**

```rust
#[test]
fn test_prove_crypto_mode_with_fallback_opt_in_preserves_mock_fallback() {
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Sp1,
        allow_mock_fallback: true,
        ..Default::default()
    });
    // in debug, still falls back to mock
}
```

**Step 4: Run test to verify RED**

Run: `cargo test -p neo-zkvm-prover test_prove_crypto_mode_with_fallback_opt_in_preserves_mock_fallback`
Expected: PASS/FAIL depending on current behavior; keep this as regression lock after implementation.

**Step 5: Commit**

```bash
git add crates/neo-zkvm-prover/src/lib.rs docs/plans/2026-02-21-proof-mode-strictness-and-verifier-policy.md
git commit -m "test: pin prover fallback policy behavior"
```

### Task 2: Add failing verifier tests (expected mode policy)

**Files:**
- Modify: `crates/neo-zkvm-verifier/src/lib.rs`
- Test: `crates/neo-zkvm-verifier/src/lib.rs` (unit tests)

**Step 1: Write failing test for mode mismatch rejection**

```rust
#[test]
fn test_verify_for_mode_rejects_mode_mismatch() {
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });
    let input = ProofInput { /* ... */ };
    let proof = prover.prove(input);

    let result = verify_detailed_for_mode(&proof, ProofMode::Sp1);
    assert!(!result.valid);
    assert!(result.error.unwrap_or_default().contains("Proof mode mismatch"));
}
```

**Step 2: Run test to verify RED**

Run: `cargo test -p neo-zkvm-verifier test_verify_for_mode_rejects_mode_mismatch`
Expected: FAIL because API does not exist yet.

**Step 3: Write passing test for matching mode**

```rust
#[test]
fn test_verify_for_mode_accepts_matching_mode() {
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });
    let input = ProofInput { /* ... */ };
    let proof = prover.prove(input);

    assert!(verify_for_mode(&proof, ProofMode::Mock));
}
```

**Step 4: Run tests to verify RED/GREEN transition boundary**

Run: `cargo test -p neo-zkvm-verifier test_verify_for_mode_`
Expected: one failing due missing implementation before code changes.

**Step 5: Commit**

```bash
git add crates/neo-zkvm-verifier/src/lib.rs
git commit -m "test: pin verifier expected-mode policy behavior"
```

### Task 3: Implement fallback policy + expected-mode verifier API

**Files:**
- Modify: `crates/neo-zkvm-prover/src/lib.rs`
- Modify: `crates/neo-zkvm-verifier/src/lib.rs`
- Modify: `crates/neo-zkvm-cli/src/main.rs`
- Modify: `crates/neo-zkvm-cli/tests/integration_tests.rs` (if default behavior changes)

**Step 1: Add `allow_mock_fallback` to `ProverConfig`**

```rust
pub allow_mock_fallback: bool,
```

Default to `false` and document that production callers must opt in explicitly.

**Step 2: Refactor `NeoProver::prove` fallback branches**

- If fallback is enabled: keep existing mock fallback behavior with warning.
- If fallback is disabled: return `failed_proof(...)` for SP1 generation errors/unavailability/input mismatch.

**Step 3: Harden `prove_strict` semantics**

- For cryptographic modes, return `Err` if proof output is failed (`state != 0`) even when proof mode matches.
- Keep existing mismatch error path.

**Step 4: Implement verifier expected-mode APIs**

```rust
pub fn verify_for_mode(proof: &NeoProof, expected_mode: ProofMode) -> bool;
pub fn verify_detailed_for_mode(proof: &NeoProof, expected_mode: ProofMode) -> VerificationResult;
```

Mode mismatch returns structured invalid result without attempting cryptographic verification.

**Step 5: Wire CLI prove path for explicit fallback policy**

- Set `allow_mock_fallback` from `--allow-fallback` in `ProverConfig`.

**Step 6: Run tests for GREEN**

Run:
- `cargo test -p neo-zkvm-prover`
- `cargo test -p neo-zkvm-verifier`
- `cargo test -p neo-zkvm-cli`

Expected: all pass.

**Step 7: Commit**

```bash
git add crates/neo-zkvm-prover/src/lib.rs crates/neo-zkvm-verifier/src/lib.rs crates/neo-zkvm-cli/src/main.rs crates/neo-zkvm-cli/tests/integration_tests.rs
git commit -m "refactor: enforce explicit prover fallback policy and verifier proof-mode checks"
```

### Task 4: Full verification + push

**Files:**
- No code changes unless verification requires follow-up fixes.

**Step 1: Format check**

Run: `cargo fmt --all -- --check`
Expected: no diff.

**Step 2: Lint check**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: no warnings.

**Step 3: Production verification script**

Run: `bash scripts/verify-production.sh`
Expected: all checks pass.

**Step 4: Push commits**

Run: `git push origin master`
Expected: remote updated.
