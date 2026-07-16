//! Library surface for Neo zkVM CLI components (assembler, disassembler, etc.).
//!
//! The `neo-zkvm` binary is a thin clap front-end over these modules. Exporting
//! them as a library lets fuzz targets and integration tools reuse the same
//! implementation without spawning the CLI process.

pub mod assembler;
pub mod disassembler;
pub mod inspector;
pub mod trace_host;
pub mod trace_step;

pub use assembler::Assembler;
pub use disassembler::Disassembler;
pub use inspector::Inspector;
