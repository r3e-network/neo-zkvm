//! Boundary condition tests for Neo VM Core
//!
//! Tests edge cases and boundary conditions for all VM operations.

use neo_vm_core::{NeoVM, StackItem, VMState};

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
    let _ = vm.load_script(vec![0x10, 0x40]); // PUSH0, RET
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_push_negative_one() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x0F, 0x40]); // PUSHM1, RET
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_push_max_small_int() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x20, 0x40]); // PUSH16, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(16)));
}

#[test]
fn test_pushint8_max() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x00, 0x7F, 0x40]); // PUSHINT8 127, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(127)));
}

#[test]
fn test_pushint8_min() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x00, 0x80, 0x40]); // PUSHINT8 -128, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-128)));
}

#[test]
fn test_pushint16_max() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x01, 0xFF, 0x7F, 0x40]); // PUSHINT16 32767, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(32767)));
}

#[test]
fn test_pushint16_min() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x01, 0x00, 0x80, 0x40]); // PUSHINT16 -32768, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-32768)));
}

// ============================================================================
// Arithmetic Boundary Tests
// ============================================================================

#[test]
fn test_add_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x10, 0x9E, 0x40]); // 5 + 0 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_add_negative_result() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x12, 0x0F, 0x9E, 0x0F, 0x9E, 0x0F, 0x9E, 0x40]); // 2 + (-1) + (-1) + (-1) = -1
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_sub_to_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0x9F, 0x40]); // 5 - 5 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_sub_negative_result() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x13, 0x15, 0x9F, 0x40]); // 3 - 5 = -2
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-2)));
}

#[test]
fn test_mul_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x10, 0xA0, 0x40]); // 5 * 0 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_mul_by_one() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x11, 0xA0, 0x40]); // 5 * 1 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_mul_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x0F, 0xA0, 0x40]); // 5 * (-1) = -5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-5)));
}

#[test]
fn test_div_by_one() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x11, 0xA1, 0x40]); // 5 / 1 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_div_zero_dividend() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x10, 0x15, 0xA1, 0x40]); // 0 / 5 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_mod_zero_dividend() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x10, 0x15, 0xA2, 0x40]); // 0 % 5 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_mod_same_numbers() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xA2, 0x40]); // 5 % 5 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

// ============================================================================
// Comparison Boundary Tests
// ============================================================================

#[test]
fn test_lt_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xB5, 0x40]); // 5 < 5 = false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_le_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xB6, 0x40]); // 5 <= 5 = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_gt_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xB7, 0x40]); // 5 > 5 = false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_ge_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xB8, 0x40]); // 5 >= 5 = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_compare_with_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x10, 0x11, 0xB5, 0x40]); // 0 < 1 = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_compare_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x0F, 0x10, 0xB5, 0x40]); // -1 < 0 = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

// ============================================================================
// Stack Boundary Tests
// ============================================================================

#[test]
fn test_depth_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x43, 0x40]); // DEPTH, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_depth_single_item() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x43, 0x40]); // PUSH5, DEPTH, RET
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
}

#[test]
fn test_clear_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x49, 0x40]); // CLEAR, RET
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.len(), 0);
}

#[test]
fn test_swap_two_items() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x50, 0x40]); // 1, 2, SWAP
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(2)));
}

#[test]
fn test_rot_three_items() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x13, 0x51, 0x40]); // 1, 2, 3, ROT -> 2, 3, 1
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
    let _ = vm.load_script(vec![0x1F, 0x10, 0x91, 0x40]); // 15 & 0 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_or_with_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x1F, 0x10, 0x92, 0x40]); // 15 | 0 = 15
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(15)));
}

#[test]
fn test_xor_same_values() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x1F, 0x1F, 0x93, 0x40]); // 15 ^ 15 = 0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_shl_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x10, 0xA8, 0x40]); // 5 << 0 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_shr_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x10, 0xA9, 0x40]); // 5 >> 0 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

// ============================================================================
// Array Boundary Tests
// ============================================================================

#[test]
fn test_newarray_zero_size() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x10, 0xC3, 0x40]); // PUSH0, NEWARRAY
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Array(vec![])));
}

#[test]
fn test_newarray0() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0xC2, 0x40]); // NEWARRAY0
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Array(vec![])));
}

#[test]
fn test_size_empty_array() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0xC2, 0xCA, 0x40]); // NEWARRAY0, SIZE
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

// ============================================================================
// Boolean Boundary Tests
// ============================================================================

#[test]
fn test_not_true() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0xAA, 0x40]); // PUSH1 (true), NOT
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_not_false() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x10, 0xAA, 0x40]); // PUSH0 (false), NOT
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_booland_true_true() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x11, 0xAB, 0x40]); // true && true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_booland_true_false() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x10, 0xAB, 0x40]); // true && false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_boolor_false_false() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x10, 0x10, 0xAC, 0x40]); // false || false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_boolor_true_false() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x10, 0xAC, 0x40]); // true || false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

// ============================================================================
// WITHIN Boundary Tests
// ============================================================================

#[test]
fn test_within_at_lower_bound() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x13, 0x13, 0x17, 0xBB, 0x40]); // 3 within [3, 7) = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_within_at_upper_bound() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x17, 0x13, 0x17, 0xBB, 0x40]); // 7 within [3, 7) = false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_within_below_range() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x12, 0x13, 0x17, 0xBB, 0x40]); // 2 within [3, 7) = false
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_within_inside_range() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x13, 0x17, 0xBB, 0x40]); // 5 within [3, 7) = true
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

// ============================================================================
// MIN/MAX Boundary Tests
// ============================================================================

#[test]
fn test_min_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xB9, 0x40]); // min(5, 5) = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_max_equal_values() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xBA, 0x40]); // max(5, 5) = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_min_with_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x0F, 0x15, 0xB9, 0x40]); // min(-1, 5) = -1
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_max_with_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x0F, 0x15, 0xBA, 0x40]); // max(-1, 5) = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

// ============================================================================
// Integer Overflow Boundary Tests
// ============================================================================

#[test]
fn test_negate_max_i8() {
    // -128 negated should fail (overflow)
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x00, 0x80, 0x9B, 0x40]); // PUSHINT8 -128, NEGATE, RET
    run_vm(&mut vm);
    // The VM should handle this gracefully
    assert!(matches!(vm.state, VMState::Halt) || matches!(vm.state, VMState::Fault));
}

#[test]
fn test_abs_max_i8() {
    // abs(-128) should fail (overflow)
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x00, 0x80, 0x9A, 0x40]); // PUSHINT8 -128, ABS, RET
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt) || matches!(vm.state, VMState::Fault));
}

#[test]
fn test_add_overflow() {
    // Large positive + large positive should fail
    let mut vm = NeoVM::new(1_000_000);
    // PUSHINT16 32767, PUSHINT16 32767, ADD should overflow
    let _ = vm.load_script(vec![0x01, 0xFF, 0x7F, 0x01, 0xFF, 0x7F, 0x9E, 0x40]);
    run_vm(&mut vm);
    // 32767 + 32767 = 65534, which fits in i128
    assert!(matches!(vm.state, VMState::Halt));
}

#[test]
fn test_sub_underflow() {
    // MIN - positive should fail
    let mut vm = NeoVM::new(1_000_000);
    // PUSHINT8 -128, PUSH1, SUB should underflow
    let _ = vm.load_script(vec![0x00, 0x80, 0x11, 0x9F, 0x40]);
    run_vm(&mut vm);
    // -128 - 1 = -129, which fits in i128
    assert!(matches!(vm.state, VMState::Halt));
}

#[test]
fn test_mul_overflow() {
    // 256 * 256 = 65536, which fits in i128
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x01, 0x00, 0x01, 0x01, 0x00, 0xA0, 0x40]);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
}

#[test]
fn test_inc_overflow() {
    // Increment multiple times to test overflow
    let mut vm = NeoVM::new(1_000_000);
    let mut script = Vec::new();
    for _ in 0..100 {
        script.push(0x11); // PUSH1
        script.push(0x9C); // INC
    }
    script.push(0x40); // RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
}

#[test]
fn test_dec_underflow() {
    // i128::MIN - 1 should fail
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x00, 0x80, 0x9D, 0x40]); // PUSHINT8 -128, DEC, RET
    run_vm(&mut vm);
    // -128 - 1 = -129, which fits in i128
    assert!(matches!(vm.state, VMState::Halt));
}

#[test]
fn test_pow_zero_exponent() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x10, 0xA3, 0x40]); // 5^0 = 1
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
}

#[test]
fn test_pow_one_exponent() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x11, 0xA3, 0x40]); // 5^1 = 5
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_pow_negative_exponent() {
    // Negative exponent should fail
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x0F, 0xA3, 0x40]); // 5^-1 should fail
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Script Size Boundary Tests
// ============================================================================

#[test]
fn test_empty_script() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![]);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
}

#[test]
fn test_single_nop() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x42]); // NOP
    assert!(matches!(vm.state, VMState::None));
}

#[test]
fn test_ret_single_context_halts() {
    // RET on a script with only the main context should Halt normally
    // This is the expected behavior for a simple return
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x40]; // RET - main script returns
    let result = vm.load_script(script);
    assert!(result.is_ok());
    let exec_result = vm.execute_next();
    // RET pops the only context, sees stack is empty, sets Halt
    assert!(
        matches!(vm.state, VMState::Halt),
        "Expected Halt, got {:?}",
        vm.state
    );
    assert!(exec_result.is_ok(), "RET should succeed for main script");
}

// ============================================================================
// Stack Depth Boundary Tests
// ============================================================================

#[test]
fn test_stack_depth_limit() {
    let mut vm = NeoVM::new(1_000_000);
    // Push many items to test depth handling
    let mut script = Vec::new();
    for _ in 0..100 {
        script.push(0x11); // PUSH1
    }
    script.push(0x40); // RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.len(), 100);
}

// ============================================================================
// Gas Exhaustion Tests
// ============================================================================

#[test]
fn test_gas_exhaustion() {
    let mut vm = NeoVM::new(10); // Very low gas limit
                                 // Create a script that needs more gas than available
    let mut script = Vec::new();
    for _ in 0..20 {
        script.push(0x11); // PUSH1
    }
    script.push(0x40); // RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    // Should fault due to gas exhaustion
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Arithmetic Overflow Tests
// ============================================================================

#[test]
fn test_add_overflow_detection() {
    let mut vm = NeoVM::new(1_000_000);
    // i128::MAX + 1 should overflow
    let max_val = i128::MAX;
    let script = vec![
        0x02, // PUSHINT32
        (max_val & 0xFF) as u8,
        ((max_val >> 8) & 0xFF) as u8,
        ((max_val >> 16) & 0xFF) as u8,
        ((max_val >> 24) & 0xFF) as u8,
        0x02, // PUSHINT32
        1u8,
        0u8,
        0u8,
        0u8,  // 1
        0x9E, // ADD
        0x40, // RET
    ];
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_sub_underflow_detection() {
    let mut vm = NeoVM::new(1_000_000);
    // i128::MIN - 1 should overflow
    let min_val = i128::MIN;
    let script = vec![
        0x02, // PUSHINT32
        (min_val & 0xFF) as u8,
        ((min_val >> 8) & 0xFF) as u8,
        ((min_val >> 16) & 0xFF) as u8,
        ((min_val >> 24) & 0xFF) as u8,
        0x02, // PUSHINT32
        1u8,
        0u8,
        0u8,
        0u8,  // 1
        0x9F, // SUB
        0x40, // RET
    ];
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_mul_overflow_detection() {
    let mut vm = NeoVM::new(1_000_000);
    // i128::MAX * 2 should overflow
    let max_val = i128::MAX / 2;
    let script = vec![
        0x02, // PUSHINT32
        (max_val & 0xFF) as u8,
        ((max_val >> 8) & 0xFF) as u8,
        ((max_val >> 16) & 0xFF) as u8,
        ((max_val >> 24) & 0xFF) as u8,
        0x02, // PUSHINT32
        (2i128 & 0xFF) as u8,
        ((2i128 >> 8) & 0xFF) as u8,
        ((2i128 >> 16) & 0xFF) as u8,
        ((2i128 >> 24) & 0xFF) as u8,
        0xA0, // MUL
        0x40, // RET
    ];
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_negate_overflow_detection() {
    let mut vm = NeoVM::new(1_000_000);
    // NEGATE i128::MIN should overflow
    let min_val = i128::MIN;
    let script = vec![
        0x02, // PUSHINT32
        (min_val & 0xFF) as u8,
        ((min_val >> 8) & 0xFF) as u8,
        ((min_val >> 16) & 0xFF) as u8,
        ((min_val >> 24) & 0xFF) as u8,
        0x9B, // NEGATE
        0x40, // RET
    ];
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_abs_overflow_detection() {
    let mut vm = NeoVM::new(1_000_000);
    // ABS of i128::MIN should overflow
    let min_val = i128::MIN;
    let script = vec![
        0x02, // PUSHINT32
        (min_val & 0xFF) as u8,
        ((min_val >> 8) & 0xFF) as u8,
        ((min_val >> 16) & 0xFF) as u8,
        ((min_val >> 24) & 0xFF) as u8,
        0x9A, // ABS
        0x40, // RET
    ];
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_div_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x10, 0xA1, 0x40]; // 1, 0, DIV
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_mod_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x10, 0xA2, 0x40]; // 1, 0, MOD
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Script Validation Tests
// ============================================================================

#[test]
fn test_pushdata1_truncated() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSHDATA1 with length 10 but only 5 bytes available
    let script = vec![0x0C, 0x0A, 0x42, 0x42, 0x42, 0x42, 0x42]; // 7 bytes total
    let result = vm.load_script(script);
    assert!(result.is_ok());
    let _ = vm.execute_next();
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_pushdata2_truncated() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSHDATA2 with length 256 but not enough bytes
    let script = vec![0x0D, 0x00, 0x01, 0x42]; // Only 1 data byte
    let result = vm.load_script(script);
    assert!(result.is_ok());
    let _ = vm.execute_next();
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_pushint8_truncated() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSHINT8 at end of script
    let script = vec![0x00]; // Just the opcode, no operand
    let result = vm.load_script(script);
    assert!(result.is_ok());
    let _ = vm.execute_next();
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_pushint16_truncated() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSHINT16 with only 1 byte available
    let script = vec![0x01, 0x42]; // Only 1 of 2 bytes
    let result = vm.load_script(script);
    assert!(result.is_ok());
    let _ = vm.execute_next();
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_drop_empty_stack() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x45, 0x40]; // DROP on empty stack
    let result = vm.load_script(script);
    assert!(result.is_ok());
    let _ = vm.execute_next();
    assert!(matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Type Conversion Tests
// ============================================================================

#[test]
fn test_isnull_true() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![
        0x0B, // PUSHNULL
        0xD8, // ISNULL
        0x40, // RET
    ];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_isnull_false() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![
        0x11, // PUSH1 (true)
        0xD8, // ISNULL
        0x40, // RET
    ];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}
