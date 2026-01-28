//! # Neo VM Core
//! 
//! Core virtual machine implementation for Neo zkVM.
//! 
//! ## Features
//! 
//! - Full Neo N3 opcode support
//! - Gas metering
//! - Execution tracing for proof generation
//! - Cryptographic operations (SHA256, RIPEMD160, ECDSA)
//! 
//! ## Example
//! 
//! ```rust
//! use neo_vm_core::{NeoVM, VMState, StackItem};
//! 
//! let mut vm = NeoVM::new(1_000_000);
//! vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]); // 2 + 3
//! 
//! while !matches!(vm.state, VMState::Halt | VMState::Fault) {
//!     vm.execute_next().unwrap();
//! }
//! 
//! assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
//! ```

pub mod opcode;
pub mod stack_item;
pub mod engine;

pub use opcode::OpCode;
pub use stack_item::StackItem;
pub use engine::{NeoVM, VMState, VMError};
