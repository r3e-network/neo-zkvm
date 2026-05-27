//! Build script for SP1 integration
//!
//! Uses `sp1-build` to compile the guest program and generate the ELF binary.
//! Falls back to a dummy ELF when the SP1 program source is not available
//! (e.g. when building from a published crate outside this workspace).

use std::path::{Path, PathBuf};

#[path = "src/elf_markers.rs"]
mod elf_markers;

fn write_dummy_elf(elf_path: &Path, marker: &[u8]) {
    if !elf_path.exists() {
        let _ = std::fs::write(elf_path, marker);
    }
}

fn enable_mock_elf() {
    // Use a non-colliding cfg name to avoid bypassing Cargo's feature
    // resolution. Source code that supports mock ELF mode should gate
    // on `#[cfg(any(feature = "mock-elf", mock_elf_available))]` so the
    // feature can be enabled via Cargo.toml or detected from the build
    // script's runtime environment.
    println!("cargo:rustc-cfg=mock_elf_available");
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
    if env_flag("SP1_FORCE_DUMMY") == Some(true) {
        return false;
    }
    if env_flag("SP1_TOOLCHAIN_AVAILABLE") == Some(true) {
        return true;
    }

    let sp1up_version = std::process::Command::new("sp1up")
        .arg("--version")
        .output();
    match sp1up_version {
        Ok(o) if o.status.success() => return true,
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            println!("cargo:warning=sp1up --version exited with status {}: {}", o.status, stderr.trim());
        }
        Err(e) => {
            println!("cargo:warning=sp1up --version failed: {e}");
        }
    }

    std::process::Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.contains("succinct"))
        .unwrap_or(false)
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
            println!("cargo:warning=neo-zkvm-prover built without the 'sp1' feature; using dummy ELF. Set SP1_ALLOW_DUMMY=1 to suppress this warning.");
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
