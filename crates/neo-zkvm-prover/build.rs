//! Build script for SP1 integration
//!
//! Uses `sp1-build` to compile the guest program and generate the ELF binary.
//! Falls back to a dummy ELF when the SP1 program source is not available
//! (e.g. when building from a published crate outside this workspace).

use std::path::{Path, PathBuf};

#[path = "src/elf_markers.rs"]
mod elf_markers;

fn write_dummy_elf(elf_path: &Path, marker: &[u8]) {
    // Always overwrite: a previous successful guest build must not leave a
    // real ELF when SP1_FORCE_DUMMY / missing toolchain / build failure
    // selects a dummy marker. Stale real ELFs would make is_elf_available()
    // report true under force-dummy CI paths.
    if let Err(e) = std::fs::write(elf_path, marker) {
        println!(
            "cargo:warning=Failed to write dummy ELF marker to {}: {e}",
            elf_path.display()
        );
    }
}

fn enable_mock_elf() {
    // Non-colliding cfg so consumers can detect dummy-ELF builds without
    // bypassing Cargo feature resolution.
    println!("cargo:rustc-cfg=mock_elf_available");
    if std::env::var("CARGO_FEATURE_MOCK_ELF").is_ok() {
        println!("cargo:warning=neo-zkvm-prover built with feature mock-elf (dummy ELF path)");
    }
}

#[cfg(feature = "sp1")]
fn resolve_program_dir(manifest_dir: &Path) -> Option<PathBuf> {
    if let Ok(from_env) = std::env::var("NEO_ZKVM_PROGRAM_DIR") {
        let path = PathBuf::from(&from_env);
        if path.join("Cargo.toml").exists() {
            return Some(path);
        }
        println!(
            "cargo:warning=NEO_ZKVM_PROGRAM_DIR is set to '{}' but no Cargo.toml exists there",
            from_env
        );
    }

    let workspace_path = manifest_dir.join("../neo-zkvm-program");
    if workspace_path.join("Cargo.toml").exists() {
        Some(workspace_path)
    } else {
        None
    }
}

#[cfg(feature = "sp1")]
fn env_flag(name: &str) -> Option<bool> {
    std::env::var(name).ok().map(|v| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

#[cfg(feature = "sp1")]
fn has_sp1_toolchain() -> bool {
    // Cargo feature mock-elf forces dummy ELF path (same as SP1_FORCE_DUMMY).
    if std::env::var("CARGO_FEATURE_MOCK_ELF").is_ok() {
        return false;
    }
    if env_flag("SP1_FORCE_DUMMY") == Some(true) {
        return false;
    }
    if env_flag("SP1_TOOLCHAIN_AVAILABLE") == Some(true) {
        return true;
    }

    // Check for cargo-prove binary at standard SP1 install locations
    let home = std::env::var("HOME").unwrap_or_default();
    let cargo_prove = [
        format!("{home}/.sp1/bin/cargo-prove"),
        "cargo-prove".to_string(),
    ]
    .into_iter()
    .find(|p| std::path::Path::new(p).exists());

    if cargo_prove.is_none() {
        return false;
    }

    // Verify the succinct Rust toolchain is actually installed (cargo-prove
    // binary can be present without the RISC-V target having been downloaded)
    if let Ok(o) = std::process::Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        && let Ok(output) = String::from_utf8(o.stdout)
        && output.contains("succinct")
    {
        return true;
    }
    false
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let elf_dir = PathBuf::from(&out_dir).join("elf");
    let _ = std::fs::create_dir_all(&elf_dir);

    let elf_path = elf_dir.join("riscv32im-succinct-zkvm-elf");
    #[cfg(feature = "sp1")]
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    #[cfg(feature = "sp1")]
    let has_sp1 = has_sp1_toolchain();
    println!("cargo:rerun-if-env-changed=SP1_FORCE_DUMMY");
    println!("cargo:rerun-if-env-changed=SP1_TOOLCHAIN_AVAILABLE");
    println!("cargo:rerun-if-env-changed=NEO_ZKVM_PROGRAM_DIR");
    println!("cargo:rerun-if-env-changed=SP1_ALLOW_DUMMY");

    #[cfg(not(feature = "sp1"))]
    {
        if std::env::var("SP1_ALLOW_DUMMY").as_deref() != Ok("1") {
            println!(
                "cargo:warning=neo-zkvm-prover built without the 'sp1' feature; using dummy ELF. Set SP1_ALLOW_DUMMY=1 to suppress this warning."
            );
        }
        write_dummy_elf(&elf_path, elf_markers::DUMMY_ELF_NOT_FOR_PRODUCTION);
        enable_mock_elf();
    }

    #[cfg(feature = "sp1")]
    if has_sp1 {
        let is_clippy_invocation = std::env::var("RUSTC_WORKSPACE_WRAPPER")
            .map(|val| val.contains("clippy-driver"))
            .unwrap_or(false);
        let skip_program_build = std::env::var("SP1_SKIP_PROGRAM_BUILD")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        if let Some(program_dir) = resolve_program_dir(&manifest_dir) {
            if skip_program_build {
                println!("cargo:warning=SP1_SKIP_PROGRAM_BUILD=true; skipping guest build step");
            } else {
                let build_args = sp1_build::BuildArgs {
                    output_directory: Some(elf_dir.to_string_lossy().to_string()),
                    elf_name: Some("riscv32im-succinct-zkvm-elf".to_string()),
                    ..sp1_build::BuildArgs::default()
                };
                sp1_build::build_program_with_args(&program_dir.to_string_lossy(), build_args);
            }

            if !elf_path.exists() {
                let workspace_elf_path = manifest_dir.join(
                    "../../target/elf-compilation/riscv32im-succinct-zkvm-elf/release/neo-zkvm-program",
                );

                if workspace_elf_path.exists() {
                    if let Err(e) = std::fs::copy(&workspace_elf_path, &elf_path) {
                        println!(
                            "cargo:warning=Failed to copy SP1 ELF from {} to {}: {}; using dummy ELF",
                            workspace_elf_path.display(),
                            elf_path.display(),
                            e
                        );
                        write_dummy_elf(&elf_path, elf_markers::DUMMY_ELF_BUILD_FAILED);
                        enable_mock_elf();
                    }
                } else if is_clippy_invocation || skip_program_build {
                    println!(
                        "cargo:warning=SP1 ELF not present during clippy/skip build; using temporary dummy ELF"
                    );
                    write_dummy_elf(&elf_path, elf_markers::DUMMY_ELF_FOR_CLIPPY);
                    enable_mock_elf();
                } else {
                    println!(
                        "cargo:warning=SP1 build completed but ELF was not found at {}; using dummy ELF",
                        elf_path.display()
                    );
                    write_dummy_elf(&elf_path, elf_markers::DUMMY_ELF_BUILD_FAILED);
                    enable_mock_elf();
                }
            }

            println!(
                "cargo:rerun-if-changed={}",
                program_dir.join("src").display()
            );
            println!(
                "cargo:rerun-if-changed={}",
                program_dir.join("Cargo.toml").display()
            );
        } else {
            println!(
                "cargo:warning=SP1 toolchain found but neo-zkvm-program source is unavailable; using dummy ELF"
            );
            println!(
                "cargo:warning=Set NEO_ZKVM_PROGRAM_DIR=/path/to/neo-zkvm-program to enable SP1 ELF compilation"
            );
            write_dummy_elf(&elf_path, elf_markers::DUMMY_ELF_NO_PROGRAM_SOURCE);
            enable_mock_elf();
        }
    } else {
        println!("cargo:warning=SP1 toolchain not found, using dummy ELF");
        println!("cargo:warning=Install with: curl -L https://sp1.succinct.xyz | bash && sp1up");
        write_dummy_elf(&elf_path, elf_markers::DUMMY_ELF_NOT_FOR_PRODUCTION);
        enable_mock_elf();
    }
}
