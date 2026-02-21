# Fourth Production Hardening Pass Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Increase CI and release reliability by enforcing strict workspace parity, lockfile reproducibility, workflow least-privilege, and bounded execution timeouts.

**Architecture:** Harden pipeline configuration rather than runtime code paths: tighten workflow command flags (`--workspace`, `--all-targets`, `--all-features`, `--locked`), add permissions/timeouts to reduce operational risk, and keep local scripts aligned so developers reproduce CI checks exactly.

**Tech Stack:** GitHub Actions (`ci.yml`), Rust workspace cargo commands, cargo-deny, cargo-fuzz, shell gate scripts.

### Task 1: Enforce CI strictness parity and reproducibility

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Red-check current gap**
- Validate missing controls before changes:
  - `rg -n '^permissions:' .github/workflows/ci.yml`
  - `rg -n 'cargo clippy --workspace --all-targets --all-features --locked -- -D warnings' .github/workflows/ci.yml`
  - `rg -n 'cargo test --workspace --all-targets --locked --verbose' .github/workflows/ci.yml`
- Expected before implementation: no matches for strict patterns.

**Step 2: Harden workflow policy**
- Add top-level least-privilege block:
  - `permissions: contents: read`
- Add `timeout-minutes` to every job.
- Upgrade command strictness:
  - Clippy: workspace + all-targets + all-features + locked
  - Test: workspace + all-targets + locked
  - Build/doc/examples: workspace/locked parity where applicable
  - Audit: deny checks with `--locked`

**Step 3: Green-check policy**
- Re-run the 3 `rg` checks and ensure all match expected strict forms.

### Task 2: Align local production scripts with CI strict mode

**Files:**
- Modify: `scripts/test.sh`
- Modify: `scripts/verify-production.sh`

**Step 1: Tighten gate commands**
- Update clippy commands to: `--workspace --all-targets --all-features --locked`.
- Update test commands to: `--workspace --all-targets --locked`.
- Update cargo-deny commands to include `--locked`.

**Step 2: Verify scripts execute cleanly**
- Run:
  - `bash scripts/test.sh`
  - `bash scripts/verify-production.sh`
- Expected: all steps pass.

### Task 3: Keep contributor guidance aligned

**Files:**
- Modify: `CONTRIBUTING.md`

**Step 1: Replace stale setup guidance**
- Prefer `scripts/test.sh` and `scripts/verify-production.sh` over ad-hoc command snippets.

### Task 4: Final verification evidence

**Step 1: Workspace confidence checks**
- Run:
  - `cargo test --workspace --all-targets --locked`
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`

**Step 2: Dependency policy checks**
- Run:
  - `cargo deny --locked check advisories --show-stats`
  - `cargo deny --locked -L error check bans sources --hide-inclusion-graph --show-stats`

**Step 3: Fuzz smoke checks**
- Run:
  - `cd fuzz && cargo +nightly fuzz run fuzz_vm_execution -- -runs=128 -max_len=512`
  - `cd fuzz && cargo +nightly fuzz run fuzz_script_parser -- -runs=128 -max_len=512`
