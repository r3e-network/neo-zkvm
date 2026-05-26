//! Neo zkVM CLI - Complete development toolkit
//!
//! A comprehensive command-line interface for Neo zkVM development,
//! including execution, debugging, assembly, and proof generation.

use neo_vm_guest::{execute, ProofInput};
use neo_vm_rs::{interpret_with_stack_and_syscalls, MAX_SCRIPT_SIZE};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify;
use std::env;
use std::fs;

mod assembler;
mod disassembler;
mod inspector;
mod trace_host;
mod trace_step;

use assembler::Assembler;
use disassembler::Disassembler;
use inspector::Inspector;
use trace_host::TraceHost;

const VERSION: &str = "0.2.2";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "run" => cmd_run(&args[2..]),
        "prove" => cmd_prove(&args[2..]),
        "asm" => cmd_assemble(&args[2..]),
        "disasm" => cmd_disassemble(&args[2..]),
        "debug" => cmd_debug(&args[2..]),
        "inspect" => cmd_inspect(&args[2..]),
        "version" | "-v" | "--version" => {
            println!("neo-zkvm v{}", VERSION);
            Ok(())
        }
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        cmd => {
            eprintln!("Error: Unknown command '{}'\n", cmd);
            eprintln!("Run 'neo-zkvm help' for usage information.");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn print_help() {
    println!(
        r#"Neo zkVM CLI v{}

A comprehensive toolkit for Neo zkVM development.

USAGE:
    neo-zkvm <COMMAND> [OPTIONS] [ARGS]

COMMANDS:
    run <script>        Execute a script and show results
    prove <script>      Generate ZK proof for script execution
    asm <source>        Assemble source code to bytecode
    disasm <hex>        Disassemble bytecode to readable format
    debug <script>      Trace execution with shared neo-vm-rs semantics
    inspect <script>    Analyze and display script information
    version             Show version information
    help                Show this help message

SCRIPT INPUT FORMATS:
    - Hex string:       12139E40 or 0x12139E40
    - Binary file:      script.bin or script.nef
    - Assembly file:    script.neoasm (for asm command)

EXAMPLES:
    # Execute a simple addition (PUSH2 PUSH3 ADD RET)
    neo-zkvm run 12139E40

    # Assemble source code
    neo-zkvm asm "PUSH2 PUSH3 ADD RET"
    neo-zkvm asm program.neoasm

    # Disassemble bytecode
    neo-zkvm disasm 12139E40

    # Trace execution
    neo-zkvm debug 12139E40

    # Inspect script structure
    neo-zkvm inspect 12139E40

    # Generate ZK proof (default mode: sp1)
    neo-zkvm prove 12139E40

    # Generate ZK proof with explicit mode
    neo-zkvm prove 12139E40 --proof-mode groth16
    neo-zkvm prove 12139E40 -m mock

    # Allow explicit SP1 fallback to mock when setup is unavailable
    neo-zkvm prove 12139E40 -m sp1 --allow-fallback

For more information, visit: https://github.com/r3e-network/neo-zkvm"#,
        VERSION
    );
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "Missing script argument.\n\nUsage: neo-zkvm run <script>\n\nExamples:\n  \
             neo-zkvm run 12139E40\n  neo-zkvm run script.bin"
                .to_string(),
        );
    }

    let script = parse_script(&args[0])?;
    let gas_limit = parse_gas_limit(args)?;

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
    if let Some(error) = output.error {
        println!("  Error:        {}", error);
    }
    println!("=======================================");

    Ok(())
}
fn cmd_prove(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "Missing script argument.\n\nUsage: neo-zkvm prove <script> [--proof-mode <mode>|-m <mode>] [--allow-fallback]\n\nExamples:\n  \
             neo-zkvm prove 12139E40\n  neo-zkvm prove script.bin\n  neo-zkvm prove 12139E40 \
             --proof-mode groth16\n  neo-zkvm prove 12139E40 -m mock\n  neo-zkvm prove 12139E40 -m sp1 \
             --allow-fallback"
                .to_string(),
        );
    }

    let script = parse_script(&args[0])?;
    let gas_limit = parse_gas_limit(args)?;
    let proof_mode = parse_proof_mode(args)?;
    let allow_fallback = parse_allow_fallback(args);

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

    println!("═══════════════════════════════════════");
    println!("  PROOF GENERATION RESULT");
    println!("═══════════════════════════════════════");
    println!("  Requested: {:?}", proof_mode);
    println!("  Mode:     {:?}", proof.proof_mode);
    println!("  Result:   {:?}", proof.output.result);
    let verified = verify(&proof);
    println!("  Verified: {}", verified);
    println!("═══════════════════════════════════════");

    if !verified {
        return Err("Proof verification failed; aborting.".to_string());
    }

    Ok(())
}

fn cmd_assemble(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "Missing source argument.\n\nUsage: neo-zkvm asm <source>\n\nExamples:\n  \
             neo-zkvm asm \"PUSH2 PUSH3 ADD RET\"\n  neo-zkvm asm program.neoasm"
                .to_string(),
        );
    }

    let source = if args[0].ends_with(".neoasm") {
        fs::read_to_string(&args[0]).map_err(|e| format!("Failed to read file: {}", e))?
    } else {
        args[0].clone()
    };

    let mut assembler = Assembler::new();
    let bytecode = assembler.assemble(&source)?;

    println!("{}", hex::encode(&bytecode));

    // Show warnings if any
    for warning in assembler.warnings() {
        eprintln!("Warning: {}", warning);
    }

    Ok(())
}

fn cmd_disassemble(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "Missing bytecode argument.\n\nUsage: neo-zkvm disasm <hex>\n\nExamples:\n  \
             neo-zkvm disasm 12139E40\n  neo-zkvm disasm script.bin"
                .to_string(),
        );
    }

    let script = parse_script(&args[0])?;
    let disasm = Disassembler::new(&script);

    println!("{}", disasm.disassemble());

    Ok(())
}

fn cmd_debug(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "Missing script argument.\n\nUsage: neo-zkvm debug <script>\n\nExamples:\n  \
             neo-zkvm debug 12139E40\n  neo-zkvm debug script.bin"
                .to_string(),
        );
    }

    let script = parse_script(&args[0])?;
    let mut trace_host = TraceHost::new(script.clone());

    println!("Tracing script with shared neo-vm-rs semantics...\n");
    let result = interpret_with_stack_and_syscalls(&script, Vec::new(), &mut trace_host);
    trace_host.print_trace();

    match result {
        Ok(result) => {
            println!("\nExecution result:");
            println!("  State: {:?}", result.state);
            println!("  Stack: {:?}", result.stack);
            if let Some(error) = result.fault_message {
                println!("  Fault: {}", error);
            }
            Ok(())
        }
        Err(error) => Err(format!("Trace execution failed: {error}")),
    }
}

fn cmd_inspect(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "Missing script argument.\n\nUsage: neo-zkvm inspect <script>\n\nExamples:\n  \
             neo-zkvm inspect 12139E40\n  neo-zkvm inspect script.bin"
                .to_string(),
        );
    }

    let script = parse_script(&args[0])?;
    let inspector = Inspector::new(&script);

    println!("{}", inspector.analyze());

    Ok(())
}

fn parse_script(input: &str) -> Result<Vec<u8>, String> {
    if input.ends_with(".nef") || input.ends_with(".bin") {
        let metadata =
            fs::metadata(input).map_err(|e| format!("Failed to read file '{}': {}", input, e))?;
        if metadata.len() > MAX_SCRIPT_SIZE as u64 {
            return Err(format!(
                "Script file exceeds maximum size of {} bytes",
                MAX_SCRIPT_SIZE
            ));
        }
        let content =
            fs::read(input).map_err(|e| format!("Failed to read file '{}': {}", input, e))?;
        // Defense-in-depth: re-check after read in case the file was replaced
        // between metadata() and read() (TOCTOU). The metadata check catches
        // obviously-too-large files without reading them; this catches the race.
        if content.len() > MAX_SCRIPT_SIZE {
            return Err(format!(
                "Script content exceeds maximum size of {} bytes",
                MAX_SCRIPT_SIZE
            ));
        }
        Ok(content)
    } else {
        // If the input looks like a file path (contains '.' or '/') but
        // doesn't end in .nef/.bin, the user may have mistyped a filename.
        // Give a clearer error than "Invalid hex string".
        if (input.contains('.') || input.contains('/')) && !input.starts_with("0x") {
            return Err(format!(
                "'{}' does not look like hex or a known script file — use a .nef/.bin file or hex bytes (optionally 0x-prefixed)",
                input
            ));
        }
        let hex_str = input.trim_start_matches("0x");
        let decoded = hex::decode(hex_str).map_err(|e| format!("Invalid hex string: {}", e))?;
        if decoded.len() > MAX_SCRIPT_SIZE {
            return Err(format!(
                "Script exceeds maximum size of {} bytes",
                MAX_SCRIPT_SIZE
            ));
        }
        Ok(decoded)
    }
}

fn parse_gas_limit(args: &[String]) -> Result<u64, String> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--gas" || arg == "-g" {
            let value = args
                .get(i + 1)
                .ok_or_else(|| "Missing value for --gas".to_string())?;
            return value
                .parse()
                .map_err(|_| "Invalid gas limit value".to_string());
        }
    }
    Ok(1_000_000) // Default gas limit
}

fn parse_requested_proof_mode(args: &[String]) -> Result<Option<ProofMode>, String> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--proof-mode" || arg == "-m" {
            let mode = args
                .get(i + 1)
                .ok_or_else(|| "Missing value for --proof-mode".to_string())?
                .to_ascii_lowercase();

            return match mode.as_str() {
                "execute" => Ok(Some(ProofMode::Execute)),
                "mock" => Ok(Some(ProofMode::Mock)),
                "sp1" => Ok(Some(ProofMode::Sp1)),
                "plonk" => Ok(Some(ProofMode::Plonk)),
                "groth16" => Ok(Some(ProofMode::Groth16)),
                _ => Err(
                    "Invalid proof mode. Expected one of: execute, mock, sp1, plonk, groth16"
                        .to_string(),
                ),
            };
        }
    }

    Ok(None)
}

fn parse_proof_mode(args: &[String]) -> Result<ProofMode, String> {
    Ok(parse_requested_proof_mode(args)?.unwrap_or(ProofMode::Sp1))
}

fn parse_allow_fallback(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--allow-fallback")
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
    use neo_zkvm_prover::ProofMode;

    #[test]
    fn test_parse_proof_mode_defaults_to_sp1() {
        let args = vec!["12139E40".to_string()];
        let mode = parse_proof_mode(&args).unwrap();
        assert_eq!(mode, ProofMode::Sp1);
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
            let args = vec![
                "12139E40".to_string(),
                "--proof-mode".to_string(),
                mode_str.to_string(),
            ];
            let mode = parse_proof_mode(&args).unwrap();
            assert_eq!(mode, expected_mode);
        }
    }

    #[test]
    fn test_parse_proof_mode_accepts_short_alias() {
        let args = vec!["12139E40".to_string(), "-m".to_string(), "mock".to_string()];
        let mode = parse_proof_mode(&args).unwrap();
        assert_eq!(mode, ProofMode::Mock);
    }

    #[test]
    fn test_parse_proof_mode_rejects_invalid_mode() {
        let args = vec![
            "12139E40".to_string(),
            "--proof-mode".to_string(),
            "bad-mode".to_string(),
        ];
        let err = parse_proof_mode(&args).unwrap_err();
        assert!(err.contains("Invalid proof mode"));
    }

    #[test]
    fn test_parse_proof_mode_requires_value() {
        let args = vec!["12139E40".to_string(), "--proof-mode".to_string()];
        let err = parse_proof_mode(&args).unwrap_err();
        assert!(err.contains("Missing value for --proof-mode"));
    }

    #[test]
    fn test_parse_proof_mode_requires_value_short_alias() {
        let args = vec!["12139E40".to_string(), "-m".to_string()];
        let err = parse_proof_mode(&args).unwrap_err();
        assert!(err.contains("Missing value for --proof-mode"));
    }

    #[test]
    fn test_parse_requested_proof_mode_detects_explicit_mode() {
        let args = vec!["12139E40".to_string(), "-m".to_string(), "sp1".to_string()];
        let mode = parse_requested_proof_mode(&args).unwrap();
        assert_eq!(mode, Some(ProofMode::Sp1));
    }

    #[test]
    fn test_parse_allow_fallback_flag() {
        let args = vec![
            "12139E40".to_string(),
            "-m".to_string(),
            "sp1".to_string(),
            "--allow-fallback".to_string(),
        ];
        assert!(parse_allow_fallback(&args));
    }

    #[test]
    fn test_should_error_on_fallback_for_crypto_modes() {
        // Sp1 requested, Mock produced, fallback not allowed → error
        assert!(should_error_on_fallback(
            ProofMode::Sp1,
            ProofMode::Mock,
            false,
        ));
        // Sp1 requested, Mock produced, fallback allowed → no error
        assert!(!should_error_on_fallback(
            ProofMode::Sp1,
            ProofMode::Mock,
            true,
        ));
        // Mock requested, Mock produced → no error (modes match)
        assert!(!should_error_on_fallback(
            ProofMode::Mock,
            ProofMode::Mock,
            false,
        ));
    }

    #[test]
    fn test_parse_gas_limit_requires_value() {
        let args = vec!["12139E40".to_string(), "--gas".to_string()];
        let err = parse_gas_limit(&args).unwrap_err();
        assert!(err.contains("Missing value for --gas"));
    }

    #[test]
    fn test_cmd_prove_requires_release_for_crypto_mode_without_fallback() {
        if !cfg!(debug_assertions) {
            return;
        }
        let args = vec!["12139E40".to_string()];
        let err = cmd_prove(&args).unwrap_err();
        assert!(err.contains("requires a release build"));
    }
}
