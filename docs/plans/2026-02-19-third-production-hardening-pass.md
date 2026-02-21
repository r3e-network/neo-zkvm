# Third Production Hardening Pass Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close remaining CI/release-process gaps by executing all shipped examples in CI, introducing bounded fuzz smoke coverage, and extending dependency policy checks beyond advisories.

**Architecture:** Keep correctness hardening low risk by making focused configuration changes: expand CI workflow coverage, make fuzz targets independently buildable, and add non-fatal policy checks (`bans`/`sources`) while preserving current advisory gating behavior.

**Tech Stack:** GitHub Actions, Rust workspace (`cargo`), `cargo-fuzz`, `cargo-deny`, shell verification scripts.

### Task 1: Complete CI example coverage

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Identify missing examples**
- Confirm `examples` job does not run `batch_verification`, `private_inputs`, and `tamper_resistance`.

**Step 2: Validate current behavior locally**
- Run:
  - `cargo run --bin batch_verification`
  - `cargo run --bin private_inputs`
  - `cargo run --bin tamper_resistance`
- Expected: all succeed.

**Step 3: Expand CI examples job**
- Add dedicated steps for the three missing binaries.

### Task 2: Add bounded fuzz smoke coverage

**Files:**
- Modify: `fuzz/Cargo.toml`
- Modify: `fuzz/fuzz_targets/fuzz_vm_execution.rs`
- Modify: `fuzz/fuzz_targets/fuzz_script_parser.rs`
- Modify: `.github/workflows/ci.yml`

**Step 1: Reproduce current fuzz manifest gap**
- Run: `cargo check --manifest-path fuzz/Cargo.toml`
- Expected before fix: workspace-layout error.

**Step 2: Implement minimal manifest fixes**
- Correct fuzz dependency paths to `../crates/...`.
- Add local `[workspace]` section to isolate fuzz crate from root workspace assumptions.

**Step 3: Clean fuzz target warnings**
- Explicitly consume `vm.load_script(...)` result in both fuzz targets.

**Step 4: Add CI fuzz-smoke job**
- Add `fuzz-smoke` job to install `cargo-fuzz` and run:
  - `cargo +nightly fuzz run fuzz_vm_execution -- -runs=128 -max_len=512`
  - `cargo +nightly fuzz run fuzz_script_parser -- -runs=128 -max_len=512`

### Task 3: Extend dependency policy checks without high-noise failures

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `scripts/test.sh`
- Modify: `scripts/verify-production.sh`

**Step 1: Keep advisories as hard gate**
- Preserve: `cargo deny check advisories --show-stats`.

**Step 2: Add bounded `bans`/`sources` pass**
- Add: `cargo deny -L error check bans sources --hide-inclusion-graph --show-stats`.
- Rationale: catches source/policy issues with concise output while avoiding massive warning graphs.

**Step 3: Align local scripts and CI**
- Update both scripts to include the same 5-step gate sequence and accurate step counters.

### Task 4: Final verification

**Step 1: Fuzz-specific verification**
- Run: `cargo check --manifest-path fuzz/Cargo.toml`
- Run:
  - `cd fuzz && cargo +nightly fuzz run fuzz_vm_execution -- -runs=128 -max_len=512`
  - `cd fuzz && cargo +nightly fuzz run fuzz_script_parser -- -runs=128 -max_len=512`

**Step 2: Full production gates**
- Run: `bash scripts/test.sh`
- Run: `bash scripts/verify-production.sh`

**Step 3: Workspace confidence checks**
- Run: `cargo test --workspace --all-targets`
- Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Run:
  - `cargo deny check advisories --show-stats`
  - `cargo deny -L error check bans sources --hide-inclusion-graph --show-stats`
