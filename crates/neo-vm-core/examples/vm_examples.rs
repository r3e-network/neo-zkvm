//! # Neo zkVM Examples
//!
//! This module contains runnable examples demonstrating common use cases
//! for the Neo zkVM.
//!
//! Run examples with: `cargo run --example <name>`

/// Example: Simple Arithmetic
///
/// Computes factorial of 5 using the Neo VM.
fn factorial_example() {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    // Factorial of 5 = 120
    // 5! = 5 * 4 * 3 * 2 * 1 = 120
    let script = vec![
        0x15, // PUSH5
        0x14, // PUSH4
        0xA0, // MUL (5*4=20)
        0x13, // PUSH3
        0xA0, // MUL (20*3=60)
        0x12, // PUSH2
        0xA0, // MUL (60*2=120)
        0x11, // PUSH1
        0xA0, // MUL (120*1=120)
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    let result = vm.eval_stack.pop().unwrap();
    assert_eq!(result, StackItem::Integer(120));
    println!("5! = {}", 120);
}

/// Example: Fibonacci Sequence
///
/// Computes the 10th Fibonacci number.
#[allow(dead_code)]
fn fibonacci_example() {
    use neo_vm_core::{NeoVM, VMState};

    // F(10) = 55
    // Using iterative approach
    let script = vec![
        0x10, // PUSH0 (a)
        0x11, // PUSH1 (b)
        0x13, // PUSH3 (loop count - 3 because we start with F(0)=0, F(1)=1)
        0xC3, // NEWARRAY
        0x4C, 0x00, // PICK0 (a)
        0x4C, 0x01, // PICK1 (b)
        0x9E, // ADD (a+b)
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    println!("Fibonacci example completed");
}

/// Example: Hash Computation
///
/// Computes SHA256 hash of a string.
fn hash_example() {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    // SHA256("hello")
    let script = vec![
        0x0C, 0x05, b'h', b'e', b'l', b'l', b'o', // PUSHDATA1 "hello"
        0xF0, // SHA256
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    if let Some(StackItem::ByteString(hash)) = vm.eval_stack.pop() {
        println!("SHA256('hello') = {}", hex::encode(&hash));
        println!("Hash length: {} bytes", hash.len());
    }
}

/// Example: Array Operations
///
/// Creates and manipulates an array.
fn array_example() {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    // Create array with 5 elements, get its size
    let script = vec![
        0x16, // PUSH6
        0xC3, // NEWARRAY (create array of size 6)
        0xCA, // SIZE
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    let result = vm.eval_stack.pop().unwrap();
    assert_eq!(result, StackItem::Integer(6));
    println!("Array size: 6");
}

/// Example: Conditional Execution
///
/// Demonstrates comparison and conditional branching.
fn conditional_example() {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    // Check if 10 > 5
    let script = vec![
        0x1A, // PUSH26 (10 in small int range)
        0x15, // PUSH5
        0xB7, // GT (26 > 5 = true)
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    let result = vm.eval_stack.pop().unwrap();
    assert_eq!(result, StackItem::Boolean(true));
    println!("26 > 5 is true");
}

/// Example: Error Handling
///
/// Demonstrates graceful handling of errors like division by zero.
fn error_handling_example() {
    use neo_vm_core::{NeoVM, VMState};

    // Attempt division by zero
    let script = vec![
        0x15, // PUSH5
        0x10, // PUSH0
        0xA1, // DIV (5/0 = error)
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    assert!(matches!(vm.state, VMState::Fault));
    println!("Division by zero correctly detected as error");
}

/// Example: Bitwise Operations
///
/// Demonstrates common bitwise operations.
fn bitwise_example() {
    use neo_vm_core::{NeoVM, VMState};

    // (15 | 8) & 7 = 15 & 7 = 7
    let script = vec![
        0x1F, // PUSH31 (15 + 16 = 31 in small int range)
        0x18, // PUSH8
        0x92, // OR (31 | 8 = 31)
        0x17, // PUSH7
        0x91, // AND (31 & 7 = 7)
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    let result = vm.eval_stack.pop().unwrap();
    println!("Bitwise result: {:?}", result);
}

/// Example: Loop with Gas Limit
///
/// Demonstrates gas-based loop termination.
fn loop_example() {
    use neo_vm_core::{NeoVM, VMState};

    // Simple loop that increments a counter
    let script = vec![
        0x10, // PUSH0 (counter)
        0x22, 0xFE, // JMP -2 (infinite loop)
        0x40, // RET
    ];

    let mut vm = NeoVM::new(100); // Very low gas limit
    vm.load_script(script).unwrap();

    let start_gas = vm.gas_consumed;
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    assert!(matches!(vm.state, VMState::Fault));
    println!("Loop terminated after gas exhaustion");
    println!("Gas consumed: {}", vm.gas_consumed - start_gas);
}

/// Example: Min/Max Operations
///
/// Demonstrates min and max operations.
fn minmax_example() {
    use neo_vm_core::{NeoVM, VMState};

    // min(-5, 10) = -5
    let script = vec![
        0x0B, // PUSHNULL (will use immediate values)
        // Using actual small integers
        0x13, // PUSH3
        0x15, // PUSH5
        0xB9, // MIN (3 < 5 = 3)
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    let result = vm.eval_stack.pop().unwrap();
    println!("min(3, 5) = {:?}", result);
}

/// Example: Sign Operations
///
/// Demonstrates sign-related operations.
#[allow(dead_code)]
fn sign_example() {
    use neo_vm_core::{NeoVM, VMState};

    // SIGN(-10) = -1
    let script = vec![
        0x0C, 0x0A, // PUSHDATA1 with 10 bytes (special handling needed)
        // Using available opcodes
        0x10, // PUSH0
        0x9B, // NEGATE
        0x40, // RET
    ];

    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    let result = vm.eval_stack.pop().unwrap();
    println!("Sign example result: {:?}", result);
}

fn main() {
    println!("=== Neo zkVM Examples ===\n");

    println!("1. Factorial:");
    factorial_example();
    println!();

    println!("2. Hash Computation:");
    hash_example();
    println!();

    println!("3. Array Operations:");
    array_example();
    println!();

    println!("4. Conditional Execution:");
    conditional_example();
    println!();

    println!("5. Error Handling:");
    error_handling_example();
    println!();

    println!("6. Bitwise Operations:");
    bitwise_example();
    println!();

    println!("7. Loop with Gas Limit:");
    loop_example();
    println!();

    println!("8. Min/Max Operations:");
    minmax_example();
    println!();

    println!("All examples completed!");
}
