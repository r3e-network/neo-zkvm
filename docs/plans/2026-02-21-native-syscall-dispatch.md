# Native Syscall Dispatch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire selected native contract methods into `NeoVM` syscall execution so VM bytecode can invoke built-in native functionality directly.

**Architecture:** Extend `engine::syscall` with native syscall IDs, route them through a `NativeRegistry` owned by `NeoVM`, and map native invocation errors into explicit VM faults. Keep scope narrow to stable one-argument methods first (`sha256`, `ripemd160`, `base64Encode`, `base64Decode`) and preserve existing syscall behavior.

**Tech Stack:** Rust, `neo-vm-core`, existing `NativeRegistry` / `StdLib` / `CryptoLib` abstractions, cargo test.

### Task 1: Add failing tests for native syscall behavior

**Files:**
- Create: `crates/neo-vm-core/tests/syscall_native_tests.rs`
- Reference: `crates/neo-vm-core/src/engine.rs`

**Step 1: Write failing tests**
- Add tests for:
  - `SYSCALL` native SHA-256 path returns 32-byte output.
  - `SYSCALL` native base64 encode/decode roundtrip works.
  - invalid argument type for a native syscall faults with native error path.

**Step 2: Run tests to verify failure**
- Run: `cargo test -p neo-vm-core --test syscall_native_tests`
- Expected: compile/runtime failure because native syscall constants/dispatch are missing.

### Task 2: Implement minimal VM native syscall wiring

**Files:**
- Modify: `crates/neo-vm-core/src/engine.rs`

**Step 1: Add VM plumbing**
- Add native syscall IDs in `syscall` module.
- Add native registry field on `NeoVM` and initialize it.
- Add helper to invoke native methods with argument popping and error mapping.

**Step 2: Extend syscall dispatcher**
- Add match arms in `execute_syscall` for selected native methods.
- Map native invocation errors to explicit `VMError` variant.

**Step 3: Run targeted tests**
- Run: `cargo test -p neo-vm-core --test syscall_native_tests`
- Expected: PASS.

### Task 3: Regression verification and integration safety

**Files:**
- Verify only

**Step 1: Core regression checks**
- Run: `cargo test -p neo-vm-core`

**Step 2: Workspace quality gates**
- Run: `cargo fmt --all -- --check`
- Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Run: `bash scripts/verify-production.sh`

**Step 3: Commit**
- Run:
  - `git add crates/neo-vm-core/src/engine.rs crates/neo-vm-core/tests/syscall_native_tests.rs docs/plans/2026-02-21-native-syscall-dispatch.md`
  - `git commit -m "refactor: wire native contract syscalls into vm dispatcher"`
