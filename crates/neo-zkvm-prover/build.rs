//! Build script for SP1 integration
//!
//! Uses sp1-build to compile the guest program and generate the ELF binary.
//! Falls back to empty ELF if SP1 toolchain is not available.

fn main() {
    // Get output directory
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let elf_dir = std::path::PathBuf::from(&out_dir).join("elf");
    std::fs::create_dir_all(&elf_dir).ok();

    let elf_path = elf_dir.join("riscv32im-succinct-zkvm-elf");

    // Check if SP1 toolchain is available
    let has_sp1 = std::process::Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.contains("succinct"))
        .unwrap_or(false);

    if has_sp1 {
        // Build the guest program with SP1
        sp1_build::build_program(&format!(
            "{}/../neo-zkvm-program",
            env!("CARGO_MANIFEST_DIR")
        ));

        println!("cargo:rerun-if-changed=../neo-zkvm-program/src");
    } else {
        println!("cargo:warning=SP1 toolchain not found, using dummy ELF");
        println!("cargo:warning=Install with: curl -L https://sp1.succinct.xyz | bash && sp1up");

        // Create a dummy ELF file so include_bytes! doesn't fail
        if !elf_path.exists() {
            std::fs::write(&elf_path, b"DUMMY_ELF_NOT_FOR_PRODUCTION").ok();
        }

        // Tell cargo we're using mock mode
        println!("cargo:rustc-cfg=feature=\"mock-elf\"");
    }
}
