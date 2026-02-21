# Native Crypto Syscall Coverage Extension Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend VM-native syscall coverage to include additional CryptoLib methods required for production parity (`checkSig`, `murmur32`).

**Architecture:** Add new syscall IDs in `engine::syscall`, route both IDs through existing `NativeRegistry` dispatch, and validate behavior with focused syscall-level tests.

**Tech Stack:** Rust, `neo-vm-core`, existing VM syscall dispatcher, native contract registry.

### Task 1: Add failing native syscall tests

**Files:**
- Modify: `crates/neo-vm-core/tests/syscall_native_tests.rs`

**Step 1: Write failing tests**
- Add tests for:
  - murmur32 syscall returns 4-byte output.
  - checkSig syscall rejects invalid message type with native error path.

**Step 2: Verify RED**
- Run: `cargo test -p neo-vm-core --test syscall_native_tests`
- Expected: failures for unknown/missing syscall dispatch.

### Task 2: Implement minimal engine wiring

**Files:**
- Modify: `crates/neo-vm-core/src/engine.rs`

**Step 1: Add syscall IDs**
- Add `SYSTEM_CRYPTO_CHECKSIG` and `SYSTEM_CRYPTO_MURMUR32` constants.

**Step 2: Extend syscall dispatch**
- Route to native registry with correct method names and argument counts.

**Step 3: Verify GREEN**
- Run: `cargo test -p neo-vm-core --test syscall_native_tests`
- Expected: PASS.

### Task 3: Full verification and commit

**Step 1: Quality gates**
- Run: `cargo test -p neo-vm-core`
- Run: `cargo fmt --all -- --check`
- Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Run: `bash scripts/verify-production.sh`

**Step 2: Commit**
- Run:
  - `git add crates/neo-vm-core/src/engine.rs crates/neo-vm-core/tests/syscall_native_tests.rs docs/plans/2026-02-21-native-syscall-crypto-extension.md`
  - `git commit -m "refactor: extend native crypto syscall coverage"`
