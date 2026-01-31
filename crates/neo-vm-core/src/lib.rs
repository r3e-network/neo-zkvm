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
//! ## Quick Start
//!
//! ```rust
//! use neo_vm_core::{NeoVM, VMState, StackItem};
//!
//! // Create a VM with 1M gas limit
//! let mut vm = NeoVM::new(1_000_000);
//!
//! // Load a script: 2 + 3 = 5
//! vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]).unwrap();
//!
//! // Execute until halt
//! while !matches!(vm.state, VMState::Halt | VMState::Fault) {
//!     vm.execute_next().unwrap();
//! }
//!
//! // Get the result
//! assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
//! ```
//!
//! ## Script Format
//!
//! Scripts are byte vectors containing Neo N3 opcodes. Common operations:
//!
//! - `0x10` - `0x20`: Push integers 0-16
//! - `0x0F`: Push -1
//! - `0x0B`: Push Null
//! - `0x9E`: ADD
//! - `0x9F`: SUB
//! - `0xA0`: MUL
//! - `0xA1`: DIV
//! - `0xA2`: MOD
//! - `0x40`: RET (return)
//!
//! ## Example: Simple Arithmetic
//!
//! ```rust
//! use neo_vm_core::{NeoVM, VMState, StackItem};
//!
//! // Compute 5 * 4 = 20
//! let script = vec![0x15, 0x14, 0xA0, 0x40];
//!
//! let mut vm = NeoVM::new(1_000_000);
//! vm.load_script(script).unwrap();
//!
//! while !matches!(vm.state, VMState::Halt | VMState::Fault) {
//!     vm.execute_next().unwrap();
//! }
//!
//! assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(20)));
//! ```
//!
//! ## Example: Hash Computation
//!
//! ```rust
//! use neo_vm_core::{NeoVM, VMState, StackItem};
//!
//! // Compute SHA256 of "hello"
//! let script = vec![
//!     0x0C, 0x05, b'h', b'e', b'l', b'l', b'o', // PUSHDATA1 "hello"
//!     0xF0, // SHA256
//!     0x40, // RET
//! ];
//!
//! let mut vm = NeoVM::new(1_000_000);
//! vm.load_script(script).unwrap();
//!
//! while !matches!(vm.state, VMState::Halt | VMState::Fault) {
//!     vm.execute_next().unwrap();
//! }
//!
//! if let Some(StackItem::ByteString(hash)) = vm.eval_stack.pop() {
//!     assert_eq!(hash.len(), 32); // SHA256 produces 32 bytes
//! }
//! ```
//!
//! ## Example: Gas Metering
//!
//! ```rust,ignore
//! use neo_vm_core::{NeoVM, VMState};
//!
//! let mut vm = NeoVM::new(10); // Very low gas limit
//!
//! // Create a loop that will exhaust gas
//! let script = vec![0x22, 0xFE]; // Infinite loop: JMP -2
//! vm.load_script(script).unwrap();
//!
//! // Execute until out of gas
//! while !matches!(vm.state, VMState::Halt | VMState::Fault) {
//!     let _ = vm.execute_next();
//! }
//!
//! assert!(matches!(vm.state, VMState::Fault));
//! assert!(vm.gas_consumed > 0);
//! ```
//!
//! ## Example: Error Handling
//!
//! The VM correctly handles errors like division by zero:
//!
//! ```rust,ignore
//! use neo_vm_core::{NeoVM, VMState};
//!
//! let mut vm = NeoVM::new(1_000_000);
//!
//! // Division by zero should cause a fault
//! let script = vec![0x15, 0x10, 0xA1, 0x40]; // 5, 0, DIV
//! vm.load_script(script).unwrap();
//!
//! while !matches!(vm.state, VMState::Halt | VMState::Fault) {
//!     let _ = vm.execute_next();
//! }
//!
//! assert!(matches!(vm.state, VMState::Fault));
//! ```
//!

pub mod engine;
pub mod native;
pub mod opcode;
pub mod stack_item;
pub mod storage;

pub use engine::{NeoVM, VMError, VMState};
pub use native::{CryptoLib, NativeContract, NativeRegistry, StdLib};
pub use opcode::OpCode;
pub use stack_item::StackItem;
pub use storage::{MemoryStorage, StorageBackend, StorageContext, TrackedStorage};
