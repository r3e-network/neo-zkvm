//! Canonical NeoVM opcode metadata shared with the N4 RISC-V profile.
//!
//! The execution engine in this crate still contains legacy zkVM crypto
//! pseudo-opcodes for compatibility. Public opcode decoding and metadata should
//! use the canonical table from `neo-vm-rs`.

pub use neo_vm_rs::OpCode;
