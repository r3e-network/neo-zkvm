//! Gas consumption tests for Neo VM Core
//!
//! Tests gas metering and limits.

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
// Basic Gas Consumption Tests
// ============================================================================

#[test]
fn test_gas_consumed_push() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x40]); // PUSH5, RET
    run_vm(&mut vm);
    assert!(vm.gas_consumed > 0);
    assert!(vm.gas_consumed < 10); // Push should be cheap
}

#[test]
fn test_gas_consumed_arithmetic() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x13, 0x9E, 0x40]); // 5 + 3
    run_vm(&mut vm);
    let gas_with_add = vm.gas_consumed;
    
    let mut vm2 = NeoVM::new(1_000_000);
    vm2.load_script(vec![0x15, 0x13, 0x40]); // Just push
    run_vm(&mut vm2);
    let gas_without_add = vm2.gas_consumed;
    
    assert!(gas_with_add > gas_without_add);
}

#[test]
fn test_gas_consumed_multiple_ops() {
    let mut vm = NeoVM::new(1_000_000);
    // 5 + 3 + 2 + 1
    vm.load_script(vec![0x15, 0x13, 0x9E, 0x12, 0x9E, 0x11, 0x9E, 0x40]);
    run_vm(&mut vm);
    assert!(vm.gas_consumed > 20); // Multiple operations
}

// ============================================================================
// Gas Limit Tests
// ============================================================================

#[test]
fn test_out_of_gas_simple() {
    let mut vm = NeoVM::new(1); // Very low gas limit
    vm.load_script(vec![0x15, 0x13, 0x9E, 0x40]);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_out_of_gas_loop() {
    let mut vm = NeoVM::new(50); // Limited gas
    // Many push operations to exhaust gas
    vm.load_script(vec![
        0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15,
        0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15,
        0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15,
        0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15,
        0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15,
        0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15,
        0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15, 0x15,
        0x40
    ]);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_exact_gas_limit() {
    // First, measure gas for a simple script
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x40]); // PUSH5, RET
    run_vm(&mut vm);
    let required_gas = vm.gas_consumed;
    
    // Now run with exact gas
    let mut vm2 = NeoVM::new(required_gas);
    vm2.load_script(vec![0x15, 0x40]);
    run_vm(&mut vm2);
    assert!(matches!(vm2.state, VMState::Halt));
}

#[test]
fn test_one_less_gas() {
    // First, measure gas for a simple script
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x13, 0x9E, 0x40]); // 5 + 3
    run_vm(&mut vm);
    let required_gas = vm.gas_consumed;
    
    // Now run with one less gas
    let mut vm2 = NeoVM::new(required_gas - 1);
    vm2.load_script(vec![0x15, 0x13, 0x9E, 0x40]);
    run_vm(&mut vm2);
    assert!(matches!(vm2.state, VMState::Fault));
}

// ============================================================================
// Gas Cost Comparison Tests
// ============================================================================

#[test]
fn test_hash_ops_cost_more() {
    // Hash operations should cost more than arithmetic
    let mut vm_hash = NeoVM::new(1_000_000);
    vm_hash.load_script(vec![0x0C, 0x05, b'h', b'e', b'l', b'l', b'o', 0xF0, 0x40]); // PUSHDATA1 "hello", SHA256
    run_vm(&mut vm_hash);
    let hash_gas = vm_hash.gas_consumed;
    
    let mut vm_add = NeoVM::new(1_000_000);
    vm_add.load_script(vec![0x15, 0x13, 0x9E, 0x40]); // 5 + 3
    run_vm(&mut vm_add);
    let add_gas = vm_add.gas_consumed;
    
    assert!(hash_gas > add_gas * 10, "Hash should cost significantly more than add");
}

#[test]
fn test_stack_ops_cheaper_than_arithmetic() {
    let mut vm_stack = NeoVM::new(1_000_000);
    vm_stack.load_script(vec![0x15, 0x4A, 0x40]); // PUSH5, DUP
    run_vm(&mut vm_stack);
    let stack_gas = vm_stack.gas_consumed;
    
    let mut vm_arith = NeoVM::new(1_000_000);
    vm_arith.load_script(vec![0x15, 0x13, 0x9E, 0x40]); // 5 + 3
    run_vm(&mut vm_arith);
    let arith_gas = vm_arith.gas_consumed;
    
    // Stack ops should be cheaper or similar
    assert!(stack_gas <= arith_gas + 5);
}

// ============================================================================
// Gas Tracking Tests
// ============================================================================

#[test]
fn test_gas_tracking_accuracy() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x40]); // PUSH5, RET
    
    let initial_gas = vm.gas_consumed;
    assert_eq!(initial_gas, 0);
    
    vm.execute_next().unwrap(); // PUSH5
    let after_push = vm.gas_consumed;
    assert!(after_push > 0);
    
    vm.execute_next().unwrap(); // RET
    let after_ret = vm.gas_consumed;
    assert!(after_ret >= after_push);
}

#[test]
fn test_gas_not_consumed_after_halt() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x40]); // PUSH5, RET
    run_vm(&mut vm);
    
    let gas_at_halt = vm.gas_consumed;
    
    // Try to execute more (should do nothing)
    let _ = vm.execute_next();
    
    assert_eq!(vm.gas_consumed, gas_at_halt);
}

// ============================================================================
// Complex Script Gas Tests
// ============================================================================

#[test]
fn test_complex_script_gas() {
    let mut vm = NeoVM::new(1_000_000);
    // Complex script: push, dup, add, mul, compare
    vm.load_script(vec![
        0x15,       // PUSH5
        0x4A,       // DUP
        0x9E,       // ADD (5+5=10)
        0x12,       // PUSH2
        0xA0,       // MUL (10*2=20)
        0x20,       // PUSH16
        0xB7,       // GT (20 > 16)
        0x40        // RET
    ]);
    run_vm(&mut vm);
    
    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
    assert!(vm.gas_consumed > 0);
}

#[test]
fn test_array_operations_gas() {
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![
        0x13,       // PUSH3
        0xC3,       // NEWARRAY
        0xCA,       // SIZE
        0x40        // RET
    ]);
    run_vm(&mut vm);
    
    assert!(matches!(vm.state, VMState::Halt));
    assert!(vm.gas_consumed > 0);
}

// ============================================================================
// Gas Limit Edge Cases
// ============================================================================

#[test]
fn test_zero_gas_limit() {
    let mut vm = NeoVM::new(0);
    vm.load_script(vec![0x15, 0x40]);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Fault));
}

#[test]
fn test_max_gas_limit() {
    let mut vm = NeoVM::new(u64::MAX);
    vm.load_script(vec![0x15, 0x13, 0x9E, 0x40]);
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));
}

#[test]
fn test_gas_consumed_equals_limit() {
    // Find exact gas needed
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(vec![0x15, 0x40]);
    run_vm(&mut vm);
    let exact_gas = vm.gas_consumed;
    
    // Run with exact gas
    let mut vm2 = NeoVM::new(exact_gas);
    vm2.load_script(vec![0x15, 0x40]);
    run_vm(&mut vm2);
    
    assert!(matches!(vm2.state, VMState::Halt));
    assert_eq!(vm2.gas_consumed, exact_gas);
}
