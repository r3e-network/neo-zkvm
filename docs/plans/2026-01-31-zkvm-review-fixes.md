# Neo zkVM Review Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bind SP1 proofs to their committed public values and harden VM script parsing to prevent panics/OOM from malformed scripts.

**Architecture:** Add explicit decoding of SP1 public values with size limits in prover/verifier and enforce equality with claimed public inputs. Centralize VM byte-reading with bounds checks, validate relative jump targets, and reject negative sizes to avoid unbounded allocations.

**Tech Stack:** Rust, sp1-sdk/sp1-primitives, bincode, neo-vm-core.

### Task 1: Verifier public-values binding

**Files:**
- Modify: `crates/neo-zkvm-verifier/src/lib.rs`
- Test: `crates/neo-zkvm-verifier/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_decode_public_inputs_roundtrip() {
    use sp1_primitives::io::SP1PublicValues;

    let inputs = PublicInputs {
        script_hash: [1u8; 32],
        input_hash: [2u8; 32],
        output_hash: [3u8; 32],
        gas_consumed: 42,
        execution_success: true,
    };

    let mut pv = SP1PublicValues::new();
    pv.write(&inputs);

    let decoded = decode_public_inputs(&pv).expect("decode should succeed");
    assert_eq!(decoded.script_hash, inputs.script_hash);
    assert_eq!(decoded.input_hash, inputs.input_hash);
    assert_eq!(decoded.output_hash, inputs.output_hash);
    assert_eq!(decoded.gas_consumed, inputs.gas_consumed);
    assert_eq!(decoded.execution_success, inputs.execution_success);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p neo-zkvm-verifier test_decode_public_inputs_roundtrip`
Expected: FAIL with missing `decode_public_inputs` or undefined symbol

**Step 3: Write minimal implementation**

```rust
fn decode_public_inputs(values: &SP1PublicValues) -> Result<PublicInputs, String> {
    bincode_options()
        .deserialize(values.as_slice())
        .map_err(|e| format!("Failed to decode public values: {e}"))
}
```

Update `verify_sp1_proof` to decode public values, compare against `proof.public_inputs`, and return invalid with a clear error if mismatched. Update `verify_with_vkey` to use `bincode_options` and the same public-values check. Move the `proof.output.state` pre-check so SP1 proofs are validated solely against public values.

**Step 4: Run test to verify it passes**

Run: `cargo test -p neo-zkvm-verifier test_decode_public_inputs_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/neo-zkvm-verifier/src/lib.rs
git commit -m "fix: bind verifier to SP1 public values"
```

### Task 2: Prover uses SP1 public values and stable input hashing

**Files:**
- Modify: `crates/neo-zkvm-prover/src/lib.rs`
- Test: `crates/neo-zkvm-prover/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_guest_input_hash_matches_serialized_guest_input() {
    let input = ProofInput {
        script: vec![0x12, 0x13, 0x9E, 0x40],
        arguments: vec![neo_vm_core::StackItem::Integer(7)],
        gas_limit: 123,
    };

    let guest = build_guest_input(&input);
    let bytes = bincode::serialize(&guest).expect("serialize");
    let hash = NeoProver::hash_data(&bytes);

    assert_eq!(hash, NeoProver::hash_guest_input(&input));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p neo-zkvm-prover test_guest_input_hash_matches_serialized_guest_input`
Expected: FAIL with missing `build_guest_input`/`hash_guest_input`

**Step 3: Write minimal implementation**

```rust
fn build_guest_input(&self, input: &ProofInput) -> GuestInput { /* map args */ }
fn hash_guest_input(input: &ProofInput) -> [u8; 32] { /* hash bincode(GuestInput) */ }
```

Use `hash_guest_input` in `prove` for `input_hash`. For SP1 proof modes, decode public inputs from `SP1ProofWithPublicValues.public_values` and replace `public_inputs` so they match the proof. If decoding fails, treat it as proof generation failure and fall back to mock mode.

**Step 4: Run test to verify it passes**

Run: `cargo test -p neo-zkvm-prover test_guest_input_hash_matches_serialized_guest_input`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/neo-zkvm-prover/src/lib.rs
git commit -m "fix: align prover public inputs with SP1 values"
```

### Task 3: VM script parsing hardening and CALL fix

**Files:**
- Modify: `crates/neo-vm-core/src/engine.rs`
- Test: `crates/neo-vm-core/tests/error_handling_tests.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn test_jmp_missing_offset_faults() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x22]).unwrap(); // JMP with no offset
    let err = vm.execute_next().unwrap_err();
    assert!(matches!(err, VMError::InvalidScript));
}

#[test]
fn test_syscall_missing_bytes_faults() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x41, 0x01, 0x02]).unwrap(); // SYSCALL with 2 bytes
    let err = vm.execute_next().unwrap_err();
    assert!(matches!(err, VMError::InvalidScript));
}

#[test]
fn test_newarray_negative_size_faults() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x0F, 0xC3]).unwrap(); // PUSHM1, NEWARRAY
    let err = vm.execute_next().and_then(|_| vm.execute_next()).unwrap_err();
    assert!(matches!(err, VMError::InvalidOperation));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p neo-vm-core test_jmp_missing_offset_faults test_syscall_missing_bytes_faults test_newarray_negative_size_faults`
Expected: FAIL due to missing bounds/negative checks

**Step 3: Write minimal implementation**

Add byte-reading helpers in `NeoVM` (u8/i8/u16/u32) with bounds checks, a `pop_usize_nonneg` helper, and a `validate_target_ip` helper. Update all opcodes that read immediates or compute jump targets to use these helpers. Fix CALL to consume the offset byte, update the caller IP to the return address, enforce `max_invocation_depth`, and validate target IP.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p neo-vm-core test_jmp_missing_offset_faults test_syscall_missing_bytes_faults test_newarray_negative_size_faults`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/neo-vm-core/src/engine.rs crates/neo-vm-core/tests/error_handling_tests.rs
git commit -m "fix: harden VM script parsing and call handling"
```

### Task 4: Full regression

**Files:**
- Test: repo

**Step 1: Run full test suite**

Run: `cargo test`
Expected: PASS (note SP1 toolchain warnings are acceptable)

**Step 2: Commit (if needed)**

```bash
git status -sb
```
Expected: clean working tree
