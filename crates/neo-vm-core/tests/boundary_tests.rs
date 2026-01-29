//! Boundary condition tests for Neo VM Core
//!
//! Tests edge cases and boundary conditions for all VM operations.

use neo_vm_core::{NeoVM, StackItem, VMState, VMError};

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
// Integer Boundary Tests
// ============================================================================

#[test]
fn test_push_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x10, 0x40]); // PUSH0, RET
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_push_negative_one() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x0F, 0x40]); // PUSHM1, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_push_max_small_int() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x20, 0x40]); // PUSH16, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(16)));
}

#[test]
fn test_pushint8_max() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x00, 0x7F, 0x40]); // PUSHINT8 127, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(127)));
}

#[test]
fn test_pushint8_min() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x00, 0x80, 0x40]); // PUSHINT8 -128, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-128)));
}

#[test]
fn test_pushint16_max() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x01, 0xFF, 0x7F, 0x40]); // PUSHINT16 32767, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(32767)));
}

#[test]
fn test_pushint16_min() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x01, 0x00, 0x80, 0x40]); // PUSHINT16 -32768, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-32768)));
}

// ============================================================================
// Arithmetic Boundary Tests
// ============================================================================

#[test]
fn test_add_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x10, 0x9E, 0x40]); // 5 + 0 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_add_negative_result() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x12, 0x0F, 0x9E, 0x0F, 0x9E, 0x0F, 0x9E, 0x40]); // 2 + (-1) + (-1) + (-1) = -1
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_sub_to_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x15, 0x9F, 0x40]); // 5 - 5 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_sub_negative_result() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x13, 0x15, 0x9F, 0x40]); // 3 - 5 = -2
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-2)));
}

#[test]
fn test_mul_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x10, 0xA0, 0x40]); // 5 * 0 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_mul_by_one() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x11, 0xA0, 0x40]); // 5 * 1 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_mul_negative() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x0F, 0xA0, 0x40]); // 5 * (-1) = -5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-5)));
}

#[test]
fn test_div_by_one() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x11, 0xA1, 0x40]); // 5 / 1 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_div_zero_dividend() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x10, 0x15, 0xA1, 0x40]); // 0 / 5 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_mod_zero_dividend() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x10, 0x15, 0xA2, 0x40]); // 0 % 5 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_mod_same_numbers() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x15, 0xA2, 0x40]); // 5 % 5 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

// ============================================================================
// Comparison Boundary Tests
// ============================================================================

#[test]
fn test_lt_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x15, 0xB5, 0x40]); // 5 < 5 = false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_le_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x15, 0xB6, 0x40]); // 5 <= 5 = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_gt_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x15, 0xB7, 0x40]); // 5 > 5 = false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_ge_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x15, 0xB8, 0x40]); // 5 >= 5 = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_compare_with_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x10, 0x11, 0xB5, 0x40]); // 0 < 1 = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_compare_negative() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x0F, 0x10, 0xB5, 0x40]); // -1 < 0 = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

// ============================================================================
// Stack Boundary Tests
// ============================================================================

#[test]
fn test_depth_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x43, 0x40]); // DEPTH, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_depth_single_item() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x43, 0x40]); // PUSH5, DEPTH, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
}

#[test]
fn test_clear_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x49, 0x40]); // CLEAR, RET
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.len(), 0);
}

#[test]
fn test_swap_two_items() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x11, 0x12, 0x50, 0x40]); // 1, 2, SWAP
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(2)));
}

#[test]
fn test_rot_three_items() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x11, 0x12, 0x13, 0x51, 0x40]); // 1, 2, 3, ROT -> 2, 3, 1
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(2)));
}

// ============================================================================
// Bitwise Boundary Tests
// ============================================================================

#[test]
fn test_and_with_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x1F, 0x10, 0x91, 0x40]); // 15 & 0 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_or_with_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x1F, 0x10, 0x92, 0x40]); // 15 | 0 = 15
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(15)));
}

#[test]
fn test_xor_same_values() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x1F, 0x1F, 0x93, 0x40]); // 15 ^ 15 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_shl_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x10, 0xA8, 0x40]); // 5 << 0 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_shr_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x10, 0xA9, 0x40]); // 5 >> 0 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

// ============================================================================
// Array Boundary Tests
// ============================================================================

#[test]
fn test_newarray_zero_size() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x10, 0xC3, 0x40]); // PUSH0, NEWARRAY
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Array(vec![])));
}

#[test]
fn test_newarray0() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0xC2, 0x40]); // NEWARRAY0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Array(vec![])));
}

#[test]
fn test_size_empty_array() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0xC2, 0xCA, 0x40]); // NEWARRAY0, SIZE
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

// ============================================================================
// Boolean Boundary Tests
// ============================================================================

#[test]
fn test_not_true() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x11, 0xAA, 0x40]); // PUSH1 (true), NOT
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_not_false() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x10, 0xAA, 0x40]); // PUSH0 (false), NOT
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_booland_true_true() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x11, 0x11, 0xAB, 0x40]); // true && true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_booland_true_false() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x11, 0x10, 0xAB, 0x40]); // true && false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_boolor_false_false() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x10, 0x10, 0xAC, 0x40]); // false || false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_boolor_true_false() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x11, 0x10, 0xAC, 0x40]); // true || false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

// ============================================================================
// WITHIN Boundary Tests
// ============================================================================

#[test]
fn test_within_at_lower_bound() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x13, 0x13, 0x17, 0xBB, 0x40]); // 3 within [3, 7) = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_within_at_upper_bound() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x17, 0x13, 0x17, 0xBB, 0x40]); // 7 within [3, 7) = false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_within_below_range() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x12, 0x13, 0x17, 0xBB, 0x40]); // 2 within [3, 7) = false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_within_inside_range() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x13, 0x17, 0xBB, 0x40]); // 5 within [3, 7) = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

// ============================================================================
// MIN/MAX Boundary Tests
// ============================================================================

#[test]
fn test_min_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x15, 0xB9, 0x40]); // min(5, 5) = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_max_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x15, 0xBA, 0x40]); // max(5, 5) = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_min_with_negative() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x0F, 0x15, 0xB9, 0x40]); // min(-1, 5) = -1
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_max_with_negative() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x0F, 0x15, 0xBA, 0x40]); // max(-1, 5) = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}
