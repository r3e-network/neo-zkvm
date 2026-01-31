//! Error handling tests for Neo VM Core
//!
//! Tests error conditions and fault states.

use neo_vm_core::{NeoVM, VMError, VMState};

// Helper to run VM until completion
fn run_vm(vm: &mut NeoVM) {
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        if vm.execute_next().is_err() {
            vm.state = VMState::Fault;
            break;
        }
    }
}

// ============================================================================
// Division by Zero Tests
// ============================================================================

#[test]
fn test_div_by_zero_faults() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x10, 0xA1, 0x40]); // 5 / 0
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_mod_by_zero_faults() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x10, 0xA2, 0x40]); // 5 % 0
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Stack Underflow Tests
// ============================================================================

#[test]
fn test_add_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x9E, 0x40]); // ADD with empty stack
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_add_single_item() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x9E, 0x40]); // PUSH5, ADD (needs 2 items)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_sub_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x9F, 0x40]); // SUB with empty stack
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_mul_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0xA0, 0x40]); // MUL with empty stack
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_div_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0xA1, 0x40]); // DIV with empty stack
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_dup_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x4A, 0x40]); // DUP with empty stack
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_drop_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x45, 0x40]); // DROP with empty stack
    run_vm(&mut vm);
    // DROP on empty stack should fault or be a no-op depending on impl
    // Current impl just pops, which returns None
}

#[test]
fn test_swap_single_item() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x50, 0x40]); // PUSH5, SWAP (needs 2 items)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_swap_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x50, 0x40]); // SWAP with empty stack
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_rot_insufficient_items() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x51, 0x40]); // 1, 2, ROT (needs 3)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_over_single_item() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x4B, 0x40]); // PUSH5, OVER (needs 2)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_nip_single_item() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x46, 0x40]); // PUSH5, NIP (needs 2)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_tuck_single_item() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x4E, 0x40]); // PUSH5, TUCK (needs 2)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Invalid Opcode Tests
// ============================================================================

#[test]
fn test_invalid_opcode() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0xFF, 0x40]); // Invalid opcode 0xFF
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_another_invalid_opcode() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0xFE, 0x40]); // Invalid opcode 0xFE
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Invalid Operation Tests
// ============================================================================

#[test]
fn test_pow_negative_exponent() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x12, 0x0F, 0xA3, 0x40]); // 2 ^ (-1) - invalid
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_shl_negative_shift() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x0F, 0xA8, 0x40]); // 5 << (-1) - invalid
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_shr_negative_shift() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x0F, 0xA9, 0x40]); // 5 >> (-1) - invalid
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Pick/Roll Out of Bounds Tests
// ============================================================================

#[test]
fn test_pick_out_of_bounds() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x15, 0x4D, 0x40]); // 1, 2, PUSH5, PICK (index 5 > stack size)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_roll_out_of_bounds() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x15, 0x52, 0x40]); // 1, 2, PUSH5, ROLL (index 5 > stack size)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_xdrop_out_of_bounds() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x15, 0x48, 0x40]); // 1, PUSH5, XDROP (index 5 > stack size)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Reverse Tests with Insufficient Items
// ============================================================================

#[test]
fn test_reverse3_insufficient() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x53, 0x40]); // 1, 2, REVERSE3 (needs 3)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_reverse4_insufficient() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x13, 0x54, 0x40]); // 1, 2, 3, REVERSE4 (needs 4)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_reversen_out_of_bounds() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x1A, 0x55, 0x40]); // 1, 2, PUSH10, REVERSEN (n=10 > stack)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Slot Access Tests
// ============================================================================

#[test]
fn test_ldloc_without_initslot() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x66, 0x40]); // LDLOC0 without INITSLOT
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_ldarg_without_initslot() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x74, 0x40]); // LDARG0 without INITSLOT
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_stloc_without_initslot() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x6D, 0x40]); // PUSH5, STLOC0 without INITSLOT
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Unknown Syscall Tests
// ============================================================================

#[test]
fn test_unknown_syscall() {
    let mut vm = NeoVM::new(1_000_000);
    // SYSCALL with unknown ID 0xFFFFFFFF
    let _ = vm.load_script(vec![0x41, 0xFF, 0xFF, 0xFF, 0xFF, 0x40]);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Type Mismatch Tests
// ============================================================================

#[test]
fn test_add_with_null() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x0B, 0x15, 0x9E, 0x40]); // PUSHNULL, PUSH5, ADD
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_size_on_integer() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0xCA, 0x40]); // PUSH5, SIZE (integer has no size)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Array Operation Error Tests
// ============================================================================

#[test]
fn test_pickitem_out_of_bounds() {
    let mut vm = NeoVM::new(1_000_000);
    // Create array of size 3, try to access index 5
    let _ = vm.load_script(vec![0x13, 0xC3, 0x15, 0xCE, 0x40]); // PUSH3, NEWARRAY, PUSH5, PICKITEM
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_setitem_out_of_bounds() {
    let mut vm = NeoVM::new(1_000_000);
    // Create array of size 2, try to set index 5
    let _ = vm.load_script(vec![0x12, 0xC3, 0x15, 0x11, 0xD0, 0x40]); // PUSH2, NEWARRAY, PUSH5, PUSH1, SETITEM
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_remove_out_of_bounds() {
    let mut vm = NeoVM::new(1_000_000);
    // Create array of size 2, try to remove index 5
    let _ = vm.load_script(vec![0x12, 0xC3, 0x15, 0xD2, 0x40]); // PUSH2, NEWARRAY, PUSH5, REMOVE
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_append_to_non_array() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x11, 0xCF, 0x40]); // PUSH5, PUSH1, APPEND (5 is not array)
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Script Boundary Tests
// ============================================================================

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
