//! Comprehensive Neo VM Tests - Production Grade

use neo_vm_core::{NeoVM, StackItem, VMState};

// === Arithmetic Tests ===

#[test]
fn test_add_positive() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x17, 0x9E, 0x40]); // 5 + 7 = 12
    vm.run();
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(12)));
}

#[test]
fn test_add_negative() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x0F, 0x9E, 0x40]); // 5 + (-1) = 4
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(4)));
}

#[test]
fn test_sub() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x1A, 0x13, 0x9F, 0x40]); // 10 - 3 = 7
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(7)));
}

#[test]
fn test_mul() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x16, 0x17, 0xA0, 0x40]); // 6 * 7 = 42
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(42)));
}

#[test]
fn test_div() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x1F, 0x15, 0xA1, 0x40]); // 15 / 5 = 3
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
}

#[test]
fn test_div_by_zero() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x10, 0xA1, 0x40]); // 5 / 0
    vm.run();
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_mod() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x1A, 0x13, 0xA2, 0x40]); // 10 % 3 = 1
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
}

// === Comparison Tests ===

#[test]
fn test_lt_true() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x13, 0x15, 0xB5, 0x40]); // 3 < 5 = true
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_lt_false() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x13, 0xB5, 0x40]); // 5 < 3 = false
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(false)));
}

#[test]
fn test_le() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xB6, 0x40]); // 5 <= 5 = true
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_gt() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x17, 0x13, 0xB7, 0x40]); // 7 > 3 = true
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

#[test]
fn test_ge() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x15, 0xB8, 0x40]); // 5 >= 5 = true
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
}

// === Stack Operation Tests ===

#[test]
fn test_dup() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x15, 0x4A, 0x40]); // 5, DUP
    vm.run();
    assert_eq!(vm.eval_stack.len(), 2);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_swap() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x50, 0x40]); // 1, 2, SWAP
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(2)));
}

#[test]
fn test_drop() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x45, 0x40]); // 1, 2, DROP
    vm.run();
    assert_eq!(vm.eval_stack.len(), 1);
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
}

#[test]
fn test_depth() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x12, 0x13, 0x43, 0x40]); // 1,2,3,DEPTH
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
}

// === Flow Control Tests ===

#[test]
fn test_jmp() {
    let mut vm = NeoVM::new(1_000_000);
    // JMP +4, PUSH1, RET, PUSH2, RET
    // Offset is relative to JMP opcode position
    let _ = vm.load_script(vec![0x22, 0x04, 0x11, 0x40, 0x12, 0x40]);
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(2)));
}

#[test]
fn test_jmpif_true() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSH1(true), JMPIF +4, PUSH5, RET, PUSH9, RET
    let _ = vm.load_script(vec![0x11, 0x24, 0x04, 0x15, 0x40, 0x19, 0x40]);
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(9)));
}

#[test]
fn test_jmpif_false() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSH0(false), JMPIF +4, PUSH5, RET, PUSH9, RET
    let _ = vm.load_script(vec![0x10, 0x24, 0x04, 0x15, 0x40, 0x19, 0x40]);
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_assert_pass() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x39, 0x15, 0x40]); // PUSH1, ASSERT, PUSH5, RET
    vm.run();
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
}

#[test]
fn test_assert_fail() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x10, 0x39, 0x15, 0x40]); // PUSH0, ASSERT, PUSH5, RET
    vm.run();
    assert!(matches!(vm.state, VMState::Fault));
}

// === Bitwise Tests ===

#[test]
fn test_and() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x1F, 0x17, 0x91, 0x40]); // 15 & 7 = 7
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(7)));
}

#[test]
fn test_or() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x18, 0x13, 0x92, 0x40]); // 8 | 3 = 11
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(11)));
}

#[test]
fn test_xor() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x1F, 0x19, 0x93, 0x40]); // 15 ^ 9 = 6
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(6)));
}

#[test]
fn test_shl() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x11, 0x13, 0xA8, 0x40]); // 1 << 3 = 8
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(8)));
}

#[test]
fn test_shr() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x20, 0x12, 0xA9, 0x40]); // 16 >> 2 = 4
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(4)));
}

// === Array Tests ===

#[test]
fn test_newarray() {
    let mut vm = NeoVM::new(1_000_000);
    let _ = vm.load_script(vec![0x13, 0xC3, 0x40]); // PUSH3, NEWARRAY
    vm.run();
    if let Some(StackItem::Array(arr)) = vm.eval_stack.pop() {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_pack_unpack() {
    let mut vm = NeoVM::new(1_000_000);
    // PUSH1, PUSH2, PUSH3, PUSH3, PACK, UNPACK
    let _ = vm.load_script(vec![0x11, 0x12, 0x13, 0x13, 0xC0, 0xC1, 0x40]);
    vm.run();
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3))); // count
}

// === Gas Limit Tests ===

#[test]
fn test_gas_limit() {
    let mut vm = NeoVM::new(5); // Very low gas
    let _ = vm.load_script(vec![0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x40]);
    vm.run();
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_sufficient_gas() {
    let mut vm = NeoVM::new(100);
    let _ = vm.load_script(vec![0x11, 0x12, 0x9E, 0x40]); // 1+2
    vm.run();
    assert!(matches!(vm.state, VMState::Halt));
}
