# Build Script Resilience Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove panic-based failure paths from `neo-zkvm-prover` build orchestration so builds degrade gracefully to deterministic dummy ELF markers.

**Architecture:** Introduce shared dummy ELF marker constants in source code, use those markers consistently in both runtime diagnostics and `build.rs`, and replace panic/unwrap behavior in `build.rs` with warning + mock-ELF fallback.

**Tech Stack:** Rust, Cargo build script (`build.rs`), `neo-zkvm-prover` unit tests.

### Task 1: Add failing tests for unavailable-reason classification

**Files:**
- Modify: `crates/neo-zkvm-prover/src/lib.rs`

**Step 1: Write failing tests**
- Add tests asserting unavailable-reason mapping for:
  - `DUMMY_ELF_BUILD_FAILED`
  - `DUMMY_ELF_NOT_FOR_PRODUCTION`

**Step 2: Verify RED**
- Run: `cargo test -p neo-zkvm-prover sp1_unavailable_reason_reports`
- Expected: compile/test failure before helper exists.

### Task 2: Introduce shared marker constants and runtime reason mapping

**Files:**
- Create: `crates/neo-zkvm-prover/src/elf_markers.rs`
- Modify: `crates/neo-zkvm-prover/src/lib.rs`

**Step 1: Add marker constants**
- Centralize marker byte constants used by both runtime and build script.

**Step 2: Add helper mapping in prover**
- Implement marker-based reason function used by `sp1_unavailable_reason`.

**Step 3: Verify GREEN**
- Run: `cargo test -p neo-zkvm-prover sp1_unavailable_reason_reports`
- Expected: PASS.

### Task 3: Remove panic paths in build script

**Files:**
- Modify: `crates/neo-zkvm-prover/build.rs`

**Step 1: Reuse shared marker constants**
- Include shared marker module in build script.

**Step 2: Replace panic behavior**
- Replace `panic!`/`unwrap_or_else` copy paths with `cargo:warning` + dummy marker fallback.
- Respect `SP1_SKIP_PROGRAM_BUILD=true` by skipping guest build step.

**Step 3: Validate resilience path**
- Run: `SP1_TOOLCHAIN_AVAILABLE=true SP1_SKIP_PROGRAM_BUILD=true cargo build -p neo-zkvm-prover`
- Expected: successful build without panic.

### Task 4: Full verification and commit

**Files:**
- Verify only

**Step 1: Quality gates**
- Run: `cargo fmt --all -- --check`
- Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Run: `bash scripts/verify-production.sh`

**Step 2: Commit**
- Run:
  - `git add crates/neo-zkvm-prover/build.rs crates/neo-zkvm-prover/src/elf_markers.rs crates/neo-zkvm-prover/src/lib.rs docs/plans/2026-02-21-build-script-resilience.md`
  - `git commit -m "refactor: harden prover build script fallback behavior"`
