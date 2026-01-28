//! Neo VM Core - Zero Knowledge Virtual Machine
//! 
//! This crate implements the Neo N3 Virtual Machine
//! optimized for zero-knowledge proof generation.

pub mod opcode;
pub mod stack_item;
pub mod engine;

pub use opcode::OpCode;
pub use stack_item::StackItem;
pub use engine::{NeoVM, VMState, VMError};
