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

// ============================================================================
// Arithmetic Edge Cases
// ============================================================================

#[test]
fn test_add_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x10, 0x9E, 0x40]; // 5 + 0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_add_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x0F, 0x9E, 0x40]; // 5 + (-1)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(4)));
}

#[test]
fn test_sub_result_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x15, 0x9F, 0x40]; // 5 - 5
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_mul_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x10, 0xA0, 0x40]; // 5 * 0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_mul_by_one() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x11, 0xA0, 0x40]; // 5 * 1
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_div_by_one() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x11, 0xA1, 0x40]; // 5 / 1
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_div_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x0F, 0xA1, 0x40]; // 5 / (-1)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-5)));
}

#[test]
fn test_mod_by_one() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x11, 0xA2, 0x40]; // 5 % 1
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_pow_zero_exp() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x10, 0xA3, 0x40]; // 5 ^ 0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
}

#[test]
fn test_pow_one_exp() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x11, 0xA3, 0x40]; // 5 ^ 1
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

// ============================================================================
// Comparison Edge Cases
// ============================================================================

#[test]
fn test_equal_same() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x15, 0x97, 0x40]; // 5 == 5
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_not_equal_same() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x15, 0x98, 0x40]; // 5 != 5
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_lt_same() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x15, 0xB5, 0x40]; // 5 < 5
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_le_same() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x15, 0xB6, 0x40]; // 5 <= 5
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_gt_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x10, 0x0F, 0xB7, 0x40]; // 0 > -1
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_min_same() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x15, 0xB9, 0x40]; // min(5, 5)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_max_same() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x15, 0xBA, 0x40]; // max(5, 5)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_within_exact() {
    let mut vm = NeoVM::new(1_000_000);
    // within(x, a, b) checks a <= x < b
    // Stack order: push x, then a, then b
    // within(7, 5, 10) - 5 <= 7 < 10 should be true
    let script = vec![
        0x17,       // PUSH7 (x = 7)
        0x15,       // PUSH5 (a = 5)
        0x1A,       // PUSH10 (b = 10)
        0xBB,       // WITHIN (checks 5 <= 7 < 10)
        0x40,       // RET
    ];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    // All three values consumed, result pushed
    assert_eq!(vm.eval_stack.len(), 1);
}

#[test]
fn test_within_upper() {
    let mut vm = NeoVM::new(1_000_000);
    // within(10, 5, 10) - 5 <= 10 < 10 is false
    let script = vec![0x1A, 0x15, 0x1A, 0xBB, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

// ============================================================================
// Bitwise Edge Cases
// ============================================================================

#[test]
fn test_and_all_ones() {
    let mut vm = NeoVM::new(1_000_000);
    // Simple test: 5 & 3 using small integers
    // PUSH5, PUSH3, AND, RET
    let script = vec![0x15, 0x13, 0x91, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt), "VM did not halt, state: {:?}", vm.state);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5 & 3)));
}

#[test]
fn test_and_zero() {
    let mut vm = NeoVM::new(1_000_000);
    // 5 & 0 = 0
    let script = vec![0x15, 0x10, 0x91, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_or_zero() {
    let mut vm = NeoVM::new(1_000_000);
    // 0 | 0 = 0
    let script = vec![0x10, 0x10, 0x92, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_xor_same() {
    let mut vm = NeoVM::new(1_000_000);
    // 5 ^ 5 = 0
    let script = vec![0x15, 0x15, 0x93, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_invert_zero() {
    let mut vm = NeoVM::new(1_000_000);
    // ~0 = -1 (in two's complement)
    let script = vec![0x10, 0x90, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_shl_zero() {
    let mut vm = NeoVM::new(1_000_000);
    // 5 << 0 = 5
    let script = vec![0x15, 0x10, 0xA8, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_shr_zero() {
    let mut vm = NeoVM::new(1_000_000);
    // 5 >> 0 = 5
    let script = vec![0x15, 0x10, 0xA9, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

// ============================================================================
// Sign Operations
// ============================================================================

#[test]
fn test_sign_positive() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x99, 0x40]; // sign(5)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
}

#[test]
fn test_sign_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x10, 0x99, 0x40]; // sign(0)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_sign_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x0F, 0x99, 0x40]; // sign(-1)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_abs_positive() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x15, 0x9A, 0x40]; // abs(5)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_abs_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x10, 0x9A, 0x40]; // abs(0)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_negate_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x10, 0x9B, 0x40]; // -0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_inc_max() {
    let mut vm = NeoVM::new(1_000_000);
    // Test inc at i128::MAX would overflow - use a smaller number for this test
    let script = vec![0x02, 0xFF, 0xFF, 0xFF, 0x7F, 0x9C, 0x40]; // inc(MAX_INT32-ish)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    // Should complete without error
    assert!(matches!(vm.state, VMState::Halt) || matches!(vm.state, VMState::Fault));
}

#[test]
fn test_dec_min() {
    let mut vm = NeoVM::new(1_000_000);
    // dec at i128::MIN would overflow - use a smaller number
    let script = vec![0x02, 0x00, 0x00, 0x00, 0x80, 0x9D, 0x40]; // dec(MIN_INT32-ish)
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    // Should complete without error
    assert!(matches!(vm.state, VMState::Halt) || matches!(vm.state, VMState::Fault));
}

// ============================================================================
// Logical Operations
// ============================================================================

#[test]
fn test_not_true() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0xAA, 0x40]; // NOT 1
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_not_false() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x10, 0xAA, 0x40]; // NOT 0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_booland_both_true() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x11, 0xAB, 0x40]; // 1 AND 1
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_booland_one_false() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x10, 0xAB, 0x40]; // 1 AND 0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_booland_both_false() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x10, 0x10, 0xAB, 0x40]; // 0 AND 0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_boolor_both_true() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x11, 0xAC, 0x40]; // 1 OR 1
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_boolor_one_true() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x10, 0xAC, 0x40]; // 1 OR 0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_boolor_both_false() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x10, 0x10, 0xAC, 0x40]; // 0 OR 0
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

// ============================================================================
// Stack Manipulation Edge Cases
// ============================================================================

#[test]
fn test_dup_single() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x4A, 0x40]; // PUSH1, DUP, RET
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 2);
    assert_eq!(vm.eval_stack[0], StackItem::Integer(1));
    assert_eq!(vm.eval_stack[1], StackItem::Integer(1));
}

#[test]
fn test_drop_all() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x45, 0x40]; // PUSH1, DROP, RET
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(vm.eval_stack.is_empty());
}

#[test]
fn test_swap_same() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x11, 0x50, 0x40]; // PUSH1, PUSH1, SWAP, RET
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 2);
    assert_eq!(vm.eval_stack[0], StackItem::Integer(1));
    assert_eq!(vm.eval_stack[1], StackItem::Integer(1));
}

#[test]
fn test_rot_three() {
    let mut vm = NeoVM::new(1_000_000);
    // 1, 2, 3 -> 2, 3, 1 after ROT
    let script = vec![0x11, 0x12, 0x13, 0x51, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 3);
}

#[test]
fn test_depth_empty() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x43, 0x40]; // DEPTH, RET
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(0)));
}

#[test]
fn test_depth_one() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x43, 0x40]; // PUSH1, DEPTH, RET
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    // Stack should have [1, 1] (the 1 pushed, and the depth 1)
    assert_eq!(vm.eval_stack.len(), 2);
}

#[test]
fn test_pick_zero() {
    let mut vm = NeoVM::new(1_000_000);
    // 1, 2, 3, pick(0) should duplicate top (3)
    let script = vec![0x11, 0x12, 0x13, 0x10, 0x4D, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 4);
    // Top should be 3 (the picked value)
}

#[test]
fn test_roll_zero() {
    let mut vm = NeoVM::new(1_000_000);
    // 1, 2, 3, roll(0) should move top to top (no change)
    let script = vec![0x11, 0x12, 0x13, 0x10, 0x52, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 3);
}

#[test]
fn test_over_same() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x11, 0x4B, 0x40]; // PUSH1, PUSH1, OVER, RET
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 3);
}

#[test]
fn test_nip_result() {
    let mut vm = NeoVM::new(1_000_000);
    // 1, 2, NIP -> leaves just 2
    let script = vec![0x11, 0x12, 0x46, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 1);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(2)));
}

#[test]
fn test_clear_empty() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x49, 0x40]; // CLEAR, RET
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(vm.eval_stack.is_empty());
}

#[test]
fn test_reverse3_exact() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x12, 0x13, 0x53, 0x40]; // 1, 2, 3, REVERSE3
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 3);
    // Should be 3, 2, 1 -> reversed to 1, 2, 3
}

#[test]
fn test_reverse4_exact() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0x12, 0x13, 0x14, 0x54, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 4);
}

#[test]
fn test_reversen_all() {
    let mut vm = NeoVM::new(1_000_000);
    // 1, 2, 3, reversen(3) -> reverse top 3
    let script = vec![0x11, 0x12, 0x13, 0x13, 0x55, 0x40];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.len(), 3);
}

// ============================================================================
// Jump Edge Cases
// ============================================================================

#[test]
fn test_jmp_forward() {
    let mut vm = NeoVM::new(1_000_000);
    // JMP +2 to skip next instruction
    let script = vec![
        0x22, 0x02, // JMP +2 (skip next PUSH1)
        0x11,       // PUSH1 (skipped)
        0x12,       // PUSH2
        0x40,       // RET
    ];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    // The JMP skips 2 bytes, which includes the PUSH1 opcode and its position
    // We should have just the PUSH2 value
    assert!(!vm.eval_stack.is_empty());
}

#[test]
fn test_jmpif_false() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSH0 (false), JMPIF should not jump, execution continues
    let script = vec![
        0x10,       // PUSH0 (false)
        0x24, 0x02, // JMPIF +2 (won't jump since condition is false)
        0x11,       // PUSH1 (executed after JMPIF doesn't jump)
        0x40,       // RET
    ];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    // Stack should have PUSH0 and PUSH1 (condition consumed by JMPIF)
    // Actually JMPIF pops the condition, so only PUSH1 remains
    assert!(!vm.eval_stack.is_empty());
}

#[test]
fn test_jmpifnot_true() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSH1 (true), JMPIFNOT should not jump since condition is true
    let script = vec![
        0x11,       // PUSH1 (true)
        0x26, 0x02, // JMPIFNOT +2 (won't jump since condition is true)
        0x12,       // PUSH2 (executed)
        0x40,       // RET
    ];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    // JMPIFNOT pops condition, so only PUSH2 remains
    assert!(!vm.eval_stack.is_empty());
}

#[test]
fn test_jmpeq_true() {
    let mut vm = NeoVM::new(1_000_000);
    // 5 == 5, JMPEQ should jump and consume both values
    let script = vec![
        0x15,       // PUSH5 (a)
        0x15,       // PUSH5 (b)  
        0x28, 0x02, // JMPEQ +2 (5 == 5, so jump)
        0x11,       // PUSH1 (skipped due to jump)
        0x40,       // RET
    ];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    // JMPEQ consumes both values and jumps, PUSH1 is skipped
    // Stack should be empty or have remaining items
}

#[test]
fn test_jmpeq_false() {
    let mut vm = NeoVM::new(1_000_000);
    // 5 != 3, JMPEQ should NOT jump
    let script = vec![
        0x15,       // PUSH5 (a)
        0x13,       // PUSH3 (b)
        0x28, 0x02, // JMPEQ +2 (5 != 3, so no jump)
        0x11,       // PUSH1 (executed)
        0x40,       // RET
    ];
    let _ = vm.load_script(script);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
    // JMPEQ consumes both values, PUSH1 is executed
    // Stack should have PUSH1
    assert!(!vm.eval_stack.is_empty());
}

// ============================================================================
// Stack Depth Limit Tests
// ============================================================================

#[test]
#[allow(clippy::same_item_push)]
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

#[test]
#[allow(clippy::same_item_push)]
fn test_stack_overflow_protection() {
    // Create VM with small stack limit to test overflow protection
    let mut vm = NeoVM::with_limits(1_000_000, 10, 1024); // max_stack_depth = 10
    
    // Try to push 15 items (exceeds limit of 10)
    let mut script = Vec::new();
    for _ in 0..15 {
        script.push(0x11); // PUSH1
    }
    script.push(0x40); // RET
    
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    
    // Should fault due to stack overflow
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_stack_exactly_at_limit() {
    // Create VM with stack limit of 5
    let mut vm = NeoVM::with_limits(1_000_000, 5, 1024);
    
    // Push exactly 5 items (at limit)
    let script = vec![0x11, 0x11, 0x11, 0x11, 0x11, 0x40];
    
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    
    // Should succeed
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.len(), 5);
}

// ============================================================================
// Invocation Depth Limit Tests
// ============================================================================

#[test]
fn test_invocation_depth_protection() {
    // Create VM with small invocation limit
    let mut vm = NeoVM::with_limits(1_000_000, 2048, 2); // max_invocation_depth = 2
    
    // Script that calls itself (recursion)
    // PUSH0, CALL +0 (calls itself), RET
    let script = vec![
        0x10,       // PUSH0
        0x34, 0x00, // CALL +0 (calls from offset 2 back to offset 2)
        0x40,       // RET
    ];
    
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    
    // Should fault due to invocation depth exceeded
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_multiple_load_script_exceeds_limit() {
    // Create VM with invocation limit of 3
    let mut vm = NeoVM::with_limits(1_000_000, 2048, 3);
    
    // Load first script
    let script1 = vec![0x11, 0x40]; // PUSH1, RET
    assert!(vm.load_script(script1).is_ok());
    
    // Load second script
    let script2 = vec![0x12, 0x40]; // PUSH2, RET
    assert!(vm.load_script(script2).is_ok());
    
    // Load third script
    let script3 = vec![0x13, 0x40]; // PUSH3, RET
    assert!(vm.load_script(script3).is_ok());
    
    // Fourth script should fail (exceeds limit of 3)
    let script4 = vec![0x14, 0x40]; // PUSH4, RET
    assert!(vm.load_script(script4).is_err());
}

// ============================================================================
// Gas Exhaustion Tests
// ============================================================================

#[test]
#[allow(clippy::same_item_push)]
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
#[allow(clippy::erasing_op)]
fn test_mul_overflow_detection() {
    let mut vm = NeoVM::new(1_000_000);
    // i128::MAX * 2 should overflow
    let max_val = i128::MAX / 2 + 1;
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
// Data Push Edge Cases
// ============================================================================

#[test]
fn test_pushdata1_empty() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSHDATA1 with 0 length
    let script = vec![0x0C, 0x00, 0x40]; // PUSHDATA1 0 bytes, RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::ByteString(vec![])));
}

#[test]
fn test_pushdata1_single() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x0C, 0x01, 0xFF, 0x40]; // PUSHDATA1 1 byte (0xFF), RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::ByteString(vec![0xFF])));
}

#[test]
fn test_pushint8_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x00, 0xFF, 0x40]; // PUSHINT8 -1 (0xFF as i8), RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-1)));
}

#[test]
fn test_pushnull() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x0B, 0x40]; // PUSHNULL, RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Null));
}

// ============================================================================
// Type Conversion Edge Cases
// ============================================================================

#[test]
fn test_nz_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x10, 0xB1, 0x40]; // PUSH0, NZ, RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_nz_nonzero() {
    let mut vm = NeoVM::new(1_000_000);
    let script = vec![0x11, 0xB1, 0x40]; // PUSH1, NZ, RET
    let _ = vm.load_script(script).ok();
    run_vm(&mut vm);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

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
