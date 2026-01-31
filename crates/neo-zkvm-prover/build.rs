//! Build script to embed SP1 guest program ELF
//!
//! This build script compiles the neo-zkvm-program and embeds the resulting
//! ELF binary into the prover crate for use with SP1 zkVM.
//!
//! To build the guest program:
//!   cargo build --package neo-zkvm-program --release
//!
//! The ELF will be copied to the prover's out directory and embedded.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Get the package directory
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Path to the guest program
    let guest_program = package_dir.parent().unwrap().join("neo-zkvm-program");

    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| "target/out".to_string());
    let out_dir = PathBuf::from(out_dir);

    // Create output directory
    std::fs::create_dir_all(&out_dir).unwrap_or_default();

    let elf_path = out_dir.join("neo-zkvm-program.bin");

    // Try to build the guest program
    println!("Checking for neo-zkvm-program build...");

    let status = Command::new("cargo")
        .args(vec![
            "build",
            "--package",
            "neo-zkvm-program",
            "--release",
            "--manifest-path",
            guest_program
                .join("Cargo.toml")
                .to_str()
                .unwrap_or("Cargo.toml"),
        ])
        .current_dir(&package_dir)
        .status();

    match status {
        Ok(s) if s.success() => {
            // Copy the ELF binary
            let release_elf = guest_program
                .join("target")
                .join("release")
                .join("neo-zkvm-program");

            // Try different possible binary names
            let source_elf = if release_elf.exists() {
                release_elf
            } else {
                guest_program
                    .join("target")
                    .join("release")
                    .join("neo_zkvm_program")
            };

            if source_elf.exists() {
                if let Err(e) = std::fs::copy(&source_elf, &elf_path) {
                    println!("Warning: Could not copy ELF: {}", e);
                } else {
                    println!("ELF binary embedded at: {:?}", elf_path);
                }
            } else {
                println!(
                    "Warning: Guest program binary not found at {:?}",
                    source_elf
                );
                println!("SP1 proof generation will use mock proofs only.");
            }
        }
        _ => {
            println!("Warning: Failed to build neo-zkvm-program");
            println!("SP1 proof generation will use mock proofs only.");
        }
    }

    // Tell cargo to rerun if the guest program source changes
    println!("cargo:rerun-if-changed={}", guest_program.display());
}
