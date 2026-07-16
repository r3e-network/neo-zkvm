//! Neo zkVM CLI — development toolkit for execution, assembly, and proving.

use clap::{Parser, Subcommand, ValueEnum};
use neo_vm_guest::{ProofInput, bincode_serialize, deserialize_neoproof, execute};
use neo_vm_rs::{MAX_SCRIPT_SIZE, interpret_with_stack_and_syscalls};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::{verify_detailed_for_mode, verify_for_mode};
use std::fs;
use std::path::{Path, PathBuf};

mod assembler;
mod disassembler;
mod inspector;
mod trace_host;
mod trace_step;

use assembler::Assembler;
use disassembler::Disassembler;
use inspector::Inspector;
use trace_host::TraceHost;

const DEFAULT_GAS: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliProofMode {
    Execute,
    Mock,
    Sp1,
    Plonk,
    Groth16,
}

impl From<CliProofMode> for ProofMode {
    fn from(mode: CliProofMode) -> Self {
        match mode {
            CliProofMode::Execute => ProofMode::Execute,
            CliProofMode::Mock => ProofMode::Mock,
            CliProofMode::Sp1 => ProofMode::Sp1,
            CliProofMode::Plonk => ProofMode::Plonk,
            CliProofMode::Groth16 => ProofMode::Groth16,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "neo-zkvm",
    version,
    about = "Neo zkVM development toolkit",
    long_about = "Execute, assemble, disassemble, debug, inspect, and prove Neo VM scripts with shared neo-vm-rs semantics."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Execute a script and show results
    Run {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
        /// Gas / step budget
        #[arg(short = 'g', long = "gas", default_value_t = DEFAULT_GAS)]
        gas: u64,
    },
    /// Generate a ZK proof for script execution
    Prove {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
        /// Gas / step budget
        #[arg(short = 'g', long = "gas", default_value_t = DEFAULT_GAS)]
        gas: u64,
        /// Proof mode (default: sp1)
        #[arg(short = 'm', long = "proof-mode", value_enum, default_value_t = CliProofMode::Sp1)]
        proof_mode: CliProofMode,
        /// Allow mock fallback when SP1 is unavailable
        #[arg(long = "allow-fallback")]
        allow_fallback: bool,
        /// Write serialized NeoProof bytes to this path
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Assemble source code to bytecode
    Asm {
        /// Inline assembly or a .neoasm file path
        source: String,
    },
    /// Disassemble bytecode to readable form
    Disasm {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
    },
    /// Trace execution with shared neo-vm-rs semantics
    Debug {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
        /// Gas / step budget
        #[arg(short = 'g', long = "gas", default_value_t = DEFAULT_GAS)]
        gas: u64,
    },
    /// Analyze and display script information
    Inspect {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
    },
    /// Verify a serialized NeoProof file (from `prove -o`)
    Verify {
        /// Path to a bincode-serialized NeoProof
        proof: PathBuf,
        /// Required proof mode — must match the proof (pinned; never attacker-chosen alone)
        #[arg(short = 'm', long = "proof-mode", value_enum)]
        proof_mode: CliProofMode,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = dispatch(cli.command) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn dispatch(command: Commands) -> Result<(), String> {
    match command {
        Commands::Run { script, gas } => cmd_run(&script, gas),
        Commands::Prove {
            script,
            gas,
            proof_mode,
            allow_fallback,
            output,
        } => cmd_prove(&script, gas, proof_mode.into(), allow_fallback, output),
        Commands::Asm { source } => cmd_assemble(&source),
        Commands::Disasm { script } => cmd_disassemble(&script),
        Commands::Debug { script, gas } => cmd_debug(&script, gas),
        Commands::Inspect { script } => cmd_inspect(&script),
        Commands::Verify { proof, proof_mode } => cmd_verify(&proof, proof_mode.into()),
    }
}

fn cmd_run(script_input: &str, gas_limit: u64) -> Result<(), String> {
    let script = parse_script(script_input)?;

    println!("Executing script with shared neo-vm-rs semantics...\n");

    let output = execute(ProofInput {
        script,
        arguments: vec![],
        gas_limit,
    });

    println!("=======================================");
    println!("  EXECUTION RESULT");
    println!("=======================================");
    println!(
        "  State:        {}",
        if output.state == 0 { "Halt" } else { "Fault" }
    );
    println!("  Gas consumed: {}", output.gas_consumed);
    println!("  Result:       {:?}", output.result);
    if let Some(ref error) = output.error {
        println!("  Error:        {error}");
    }
    println!("=======================================");

    if output.state != 0 {
        return Err(output
            .error
            .unwrap_or_else(|| "Script execution faulted".to_string()));
    }

    Ok(())
}

fn cmd_prove(
    script_input: &str,
    gas_limit: u64,
    proof_mode: ProofMode,
    allow_fallback: bool,
    output_path: Option<PathBuf>,
) -> Result<(), String> {
    if let Some(ref path) = output_path {
        validate_output_path(path)?;
    }

    let script = parse_script(script_input)?;

    if !allow_fallback
        && matches!(
            proof_mode,
            ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16
        )
        && cfg!(debug_assertions)
    {
        return Err(
            "Requested cryptographic proof mode requires a release build. Re-run with `cargo run --release --bin neo-zkvm -- prove ...` or pass --allow-fallback.".to_string(),
        );
    }

    println!("Generating ZK proof...\n");

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit,
    };

    let prover = NeoProver::new(ProverConfig {
        proof_mode,
        allow_mock_fallback: allow_fallback,
        ..ProverConfig::default()
    });
    let proof = if allow_fallback {
        prover.prove(input)
    } else {
        prover.prove_strict(input)?
    };

    if should_error_on_fallback(proof_mode, proof.proof_mode, allow_fallback) {
        return Err(format!(
            "Requested proof mode {:?} but prover produced {:?}. Re-run with --allow-fallback to accept mock fallback, or fix SP1 setup.",
            proof_mode, proof.proof_mode
        ));
    }

    // Pin verification to the *actual* mode produced (after any allowed
    // fallback). Never use bare `verify()`, which accepts attacker-chosen Mock.
    let verified = verify_for_mode(&proof, proof.proof_mode);

    println!("=======================================");
    println!("  PROOF GENERATION RESULT");
    println!("=======================================");
    println!("  Requested: {proof_mode:?}");
    println!("  Mode:      {:?}", proof.proof_mode);
    println!("  Result:    {:?}", proof.output.result);
    println!("  Gas:       {}", proof.output.gas_consumed);
    println!("  Verified:  {verified}");
    println!("=======================================");

    if !verified {
        return Err("Proof verification failed; aborting.".to_string());
    }

    if let Some(path) = output_path {
        let bytes =
            bincode_serialize(&proof).map_err(|e| format!("Failed to serialize NeoProof: {e}"))?;
        fs::write(&path, &bytes)
            .map_err(|e| format!("Failed to write proof to '{}': {e}", path.display()))?;
        println!("Wrote {} proof bytes to {}", bytes.len(), path.display());
    }

    Ok(())
}

fn cmd_assemble(source_arg: &str) -> Result<(), String> {
    let source = if source_arg.ends_with(".neoasm") {
        fs::read_to_string(source_arg).map_err(|e| format!("Failed to read file: {e}"))?
    } else {
        source_arg.to_string()
    };

    let mut assembler = Assembler::new();
    let bytecode = assembler.assemble(&source)?;

    println!("{}", hex::encode(&bytecode));

    for warning in assembler.warnings() {
        eprintln!("Warning: {warning}");
    }

    Ok(())
}

fn cmd_disassemble(script_input: &str) -> Result<(), String> {
    let script = parse_script(script_input)?;
    let disasm = Disassembler::new(&script);
    println!("{}", disasm.disassemble());
    Ok(())
}

fn cmd_debug(script_input: &str, gas_limit: u64) -> Result<(), String> {
    let script = parse_script(script_input)?;
    let mut trace_host = TraceHost::new(script.clone(), gas_limit);

    println!("Tracing script with shared neo-vm-rs semantics...\n");
    let result = interpret_with_stack_and_syscalls(&script, Vec::new(), &mut trace_host);
    trace_host.print_trace();

    match result {
        Ok(result) => {
            println!("\nExecution result:");
            println!("  State: {:?}", result.state);
            println!("  Stack: {:?}", result.stack);
            println!("  Steps: {}", trace_host.steps_executed());
            if let Some(error) = &result.fault_message {
                println!("  Fault: {error}");
            }
            // Match `run`: non-zero exit on Fault for scripting automation.
            if !matches!(result.state, neo_vm_rs::VmState::Halt) {
                return Err(result
                    .fault_message
                    .unwrap_or_else(|| format!("Trace execution faulted: {:?}", result.state)));
            }
            Ok(())
        }
        Err(error) => Err(format!("Trace execution failed: {error}")),
    }
}

fn cmd_verify(proof_path: &Path, expected_mode: ProofMode) -> Result<(), String> {
    let bytes = fs::read(proof_path)
        .map_err(|e| format!("Failed to read proof '{}': {e}", proof_path.display()))?;

    // Bound proof file size before deserialization (DoS guard).
    const MAX_PROOF_FILE_BYTES: usize = 2 * 1024 * 1024;
    if bytes.len() > MAX_PROOF_FILE_BYTES {
        return Err(format!(
            "Proof file exceeds maximum size ({} > {MAX_PROOF_FILE_BYTES} bytes)",
            bytes.len()
        ));
    }

    let proof =
        deserialize_neoproof(&bytes).map_err(|e| format!("Failed to deserialize NeoProof: {e}"))?;

    let detailed = verify_detailed_for_mode(&proof, expected_mode);
    let simple = verify_for_mode(&proof, expected_mode);

    println!("=======================================");
    println!("  PROOF VERIFICATION");
    println!("=======================================");
    println!("  File:     {}", proof_path.display());
    println!("  Expected: {expected_mode:?}");
    println!("  Actual:   {:?}", proof.proof_mode);
    println!("  Type:     {:?}", detailed.proof_type);
    println!("  Valid:    {simple}");
    if let Some(err) = &detailed.error {
        println!("  Error:    {err}");
    }
    println!("  Gas:      {}", proof.output.gas_consumed);
    println!("  Result:   {:?}", proof.output.result);
    println!("=======================================");

    if !simple {
        return Err(detailed
            .error
            .unwrap_or_else(|| "Proof verification failed".to_string()));
    }
    Ok(())
}

fn cmd_inspect(script_input: &str) -> Result<(), String> {
    let script = parse_script(script_input)?;
    let inspector = Inspector::new(&script);
    println!("{}", inspector.analyze());
    Ok(())
}

fn parse_script(input: &str) -> Result<Vec<u8>, String> {
    if input.ends_with(".nef") || input.ends_with(".bin") {
        let metadata =
            fs::metadata(input).map_err(|e| format!("Failed to read file '{input}': {e}"))?;
        if metadata.len() > MAX_SCRIPT_SIZE as u64 {
            return Err(format!(
                "Script file exceeds maximum size of {MAX_SCRIPT_SIZE} bytes"
            ));
        }
        let content = fs::read(input).map_err(|e| format!("Failed to read file '{input}': {e}"))?;
        // Defense-in-depth: re-check after read in case the file was replaced
        // between metadata() and read() (TOCTOU).
        if content.len() > MAX_SCRIPT_SIZE {
            return Err(format!(
                "Script content exceeds maximum size of {MAX_SCRIPT_SIZE} bytes"
            ));
        }
        Ok(content)
    } else {
        if (input.contains('.') || input.contains('/')) && !input.starts_with("0x") {
            return Err(format!(
                "'{input}' does not look like hex or a known script file — use a .nef/.bin file or hex bytes (optionally 0x-prefixed)"
            ));
        }
        let hex_str = input.trim_start_matches("0x");
        let decoded = hex::decode(hex_str).map_err(|e| format!("Invalid hex string: {e}"))?;
        if decoded.len() > MAX_SCRIPT_SIZE {
            return Err(format!(
                "Script exceeds maximum size of {MAX_SCRIPT_SIZE} bytes"
            ));
        }
        Ok(decoded)
    }
}

fn validate_output_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("Output path must not be empty".to_string());
    }
    if path.is_dir() {
        return Err(format!(
            "Output path '{}' is a directory; provide a file path",
            path.display()
        ));
    }
    Ok(())
}

fn should_error_on_fallback(
    requested_mode: ProofMode,
    actual_mode: ProofMode,
    allow_fallback: bool,
) -> bool {
    !allow_fallback
        && matches!(
            requested_mode,
            ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16
        )
        && actual_mode != requested_mode
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_cli(args: &[&str]) -> Result<Cli, String> {
        Cli::try_parse_from(std::iter::once("neo-zkvm").chain(args.iter().copied()))
            .map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_proof_mode_defaults_to_sp1() {
        let cli = parse_cli(&["prove", "12139E40"]).unwrap();
        match cli.command {
            Commands::Prove { proof_mode, .. } => {
                assert_eq!(ProofMode::from(proof_mode), ProofMode::Sp1);
            }
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_parse_proof_mode_accepts_all_modes() {
        let cases = [
            ("execute", ProofMode::Execute),
            ("mock", ProofMode::Mock),
            ("sp1", ProofMode::Sp1),
            ("plonk", ProofMode::Plonk),
            ("groth16", ProofMode::Groth16),
        ];

        for (mode_str, expected_mode) in cases {
            let cli = parse_cli(&["prove", "12139E40", "--proof-mode", mode_str]).unwrap();
            match cli.command {
                Commands::Prove { proof_mode, .. } => {
                    assert_eq!(ProofMode::from(proof_mode), expected_mode);
                }
                _ => panic!("expected prove"),
            }
        }
    }

    #[test]
    fn test_parse_proof_mode_accepts_short_alias() {
        let cli = parse_cli(&["prove", "12139E40", "-m", "mock"]).unwrap();
        match cli.command {
            Commands::Prove { proof_mode, .. } => {
                assert_eq!(ProofMode::from(proof_mode), ProofMode::Mock);
            }
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_parse_proof_mode_rejects_invalid_mode() {
        let err = parse_cli(&["prove", "12139E40", "--proof-mode", "bad-mode"]).unwrap_err();
        assert!(
            err.to_ascii_lowercase().contains("invalid")
                || err.to_ascii_lowercase().contains("possible values"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_proof_mode_requires_value() {
        let err = parse_cli(&["prove", "12139E40", "--proof-mode"]).unwrap_err();
        assert!(
            err.contains("requires a value")
                || err.contains("a value is required")
                || err.contains("proof-mode"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_proof_mode_requires_value_short_alias() {
        let err = parse_cli(&["prove", "12139E40", "-m"]).unwrap_err();
        assert!(
            err.contains("requires a value")
                || err.contains("a value is required")
                || err.contains("proof-mode")
                || err.contains("-m"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_requested_proof_mode_detects_explicit_mode() {
        let cli = parse_cli(&["prove", "12139E40", "-m", "sp1"]).unwrap();
        match cli.command {
            Commands::Prove { proof_mode, .. } => {
                assert_eq!(ProofMode::from(proof_mode), ProofMode::Sp1);
            }
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_parse_allow_fallback_flag() {
        let cli = parse_cli(&["prove", "12139E40", "-m", "sp1", "--allow-fallback"]).unwrap();
        match cli.command {
            Commands::Prove { allow_fallback, .. } => assert!(allow_fallback),
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_should_error_on_fallback_for_crypto_modes() {
        assert!(should_error_on_fallback(
            ProofMode::Sp1,
            ProofMode::Mock,
            false,
        ));
        assert!(!should_error_on_fallback(
            ProofMode::Sp1,
            ProofMode::Mock,
            true,
        ));
        assert!(!should_error_on_fallback(
            ProofMode::Mock,
            ProofMode::Mock,
            false,
        ));
    }

    #[test]
    fn test_parse_gas_limit_requires_value() {
        let err = parse_cli(&["run", "12139E40", "--gas"]).unwrap_err();
        assert!(
            err.contains("requires a value")
                || err.contains("a value is required")
                || err.contains("gas"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_cmd_prove_requires_release_for_crypto_mode_without_fallback() {
        if !cfg!(debug_assertions) {
            return;
        }
        let err = cmd_prove("12139E40", DEFAULT_GAS, ProofMode::Sp1, false, None).unwrap_err();
        assert!(err.contains("requires a release build"));
    }

    #[test]
    fn test_parse_output_path_requires_value() {
        let err = parse_cli(&["prove", "12139E40", "-m", "mock", "--output"]).unwrap_err();
        assert!(
            err.contains("requires a value")
                || err.contains("a value is required")
                || err.contains("output"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_output_path_accepts_file() {
        let cli = parse_cli(&["prove", "12139E40", "-m", "mock", "-o", "proof.bin"]).unwrap();
        match cli.command {
            Commands::Prove { output, .. } => {
                assert_eq!(
                    output.as_deref().map(Path::as_os_str),
                    Some("proof.bin".as_ref())
                );
            }
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_cmd_run_fails_on_fault() {
        // PUSH5 PUSH0 DIV RET (division by zero)
        let err = cmd_run("1510A140", DEFAULT_GAS).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_cmd_debug_fails_on_fault() {
        let err = cmd_debug("1510A140", DEFAULT_GAS).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_cmd_prove_strict_fails_on_fault() {
        let err = cmd_prove("1510A140", DEFAULT_GAS, ProofMode::Mock, false, None).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_cli_run_parses_gas() {
        let cli = parse_cli(&["run", "12139E40", "-g", "42"]).unwrap();
        match cli.command {
            Commands::Run { gas, .. } => assert_eq!(gas, 42),
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn test_cli_verify_requires_mode() {
        let err = parse_cli(&["verify", "proof.bin"]).unwrap_err();
        assert!(
            err.contains("proof-mode") || err.contains("required"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_cli_verify_parses() {
        let cli = parse_cli(&["verify", "proof.bin", "-m", "mock"]).unwrap();
        match cli.command {
            Commands::Verify { proof, proof_mode } => {
                assert_eq!(proof.as_os_str(), "proof.bin");
                assert_eq!(ProofMode::from(proof_mode), ProofMode::Mock);
            }
            _ => panic!("expected verify"),
        }
    }

    #[test]
    fn test_prove_and_verify_file_roundtrip() {
        let dir =
            std::env::temp_dir().join(format!("neo-zkvm-proof-roundtrip-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("proof.bin");
        let _ = fs::remove_file(&path);

        cmd_prove(
            "12139E40",
            DEFAULT_GAS,
            ProofMode::Mock,
            false,
            Some(path.clone()),
        )
        .expect("prove should succeed");

        cmd_verify(&path, ProofMode::Mock).expect("verify should accept matching mode");
        let err = cmd_verify(&path, ProofMode::Sp1).unwrap_err();
        assert!(err.contains("Proof mode mismatch") || err.contains("mismatch"));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
