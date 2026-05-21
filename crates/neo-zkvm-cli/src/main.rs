//! Neo zkVM CLI - Complete development toolkit
//!
//! A comprehensive command-line interface for Neo zkVM development,
//! including execution, debugging, assembly, and proof generation.

use neo_vm_guest::{execute, ProofInput};
use neo_vm_rs::{
    interop_hash, interpret_with_stack_and_syscalls, last_interpreter_ip, OpCode, StackValue,
    SyscallProvider, MAX_SCRIPT_SIZE,
};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;

mod assembler;
mod disassembler;

use assembler::Assembler;
use disassembler::Disassembler;

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
        if content.len() > MAX_SCRIPT_SIZE {
            return Err(format!(
                "Script content exceeds maximum size of {} bytes",
                MAX_SCRIPT_SIZE
            ));
        }
        Ok(content)
    } else {
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

// ============================================================================
// Shared execution trace
// ============================================================================

struct TraceStep {
    ip: usize,
    opcode: u8,
    instruction: String,
}

struct TraceHost {
    script: Vec<u8>,
    steps: Vec<TraceStep>,
}

impl TraceHost {
    fn new(script: Vec<u8>) -> Self {
        Self {
            script,
            steps: Vec::new(),
        }
    }

    fn print_trace(&self) {
        println!("Executed instructions:");
        if self.steps.is_empty() {
            println!("  <none>");
            return;
        }

        for step in &self.steps {
            println!(
                "  0x{:04X}: {:02X}  {}",
                step.ip, step.opcode, step.instruction
            );
        }
    }
}

impl SyscallProvider for TraceHost {
    fn on_instruction(&mut self, opcode: u8) -> Result<(), String> {
        let ip = last_interpreter_ip() as usize;
        let instruction = if ip < self.script.len() {
            Disassembler::new(&self.script).decode_instruction(ip).0
        } else {
            format!("??? (0x{opcode:02X})")
        };

        self.steps.push(TraceStep {
            ip,
            opcode,
            instruction,
        });
        Ok(())
    }

    fn syscall(&mut self, api: u32, _ip: usize, stack: &mut Vec<StackValue>) -> Result<(), String> {
        if api == interop_hash("System.Crypto.SHA256") {
            let bytes = pop_byte_arg(stack, "System.Crypto.SHA256")?;
            stack.push(StackValue::ByteString(Sha256::digest(&bytes).to_vec()));
            return Ok(());
        }

        Err(format!("unsupported trace syscall 0x{api:08x}"))
    }
}

fn pop_byte_arg(stack: &mut Vec<StackValue>, syscall: &str) -> Result<Vec<u8>, String> {
    match stack.pop() {
        Some(StackValue::ByteString(bytes)) | Some(StackValue::Buffer(bytes)) => Ok(bytes),
        Some(other) => Err(format!(
            "{syscall} expects ByteString or Buffer, got {other:?}"
        )),
        None => Err(format!("{syscall} expects one stack argument")),
    }
}
// ============================================================================
// Inspector
// ============================================================================

struct Inspector<'a> {
    script: &'a [u8],
}

impl<'a> Inspector<'a> {
    fn new(script: &'a [u8]) -> Self {
        Self { script }
    }

    fn analyze(&self) -> String {
        let mut output = String::new();

        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str("  SCRIPT ANALYSIS\n");
        output.push_str("═══════════════════════════════════════════════════════════════\n\n");

        // Basic info
        output.push_str(&format!("  Size:         {} bytes\n", self.script.len()));
        output.push_str(&format!("  Hash (hex):   {}\n", hex::encode(self.script)));

        // Opcode statistics
        let stats = self.collect_opcode_stats();
        output.push_str("\n───────────────────────────────────────────────────────────────\n");
        output.push_str("  OPCODE STATISTICS\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");

        let mut sorted_stats: Vec<_> = stats.iter().collect();
        sorted_stats.sort_by(|a, b| b.1.cmp(a.1));

        for (name, count) in sorted_stats.iter().take(10) {
            output.push_str(&format!("    {:12} {:3}\n", name, count));
        }

        // Control flow analysis
        let jumps = self.find_jump_targets();
        if !jumps.is_empty() {
            output.push_str("\n───────────────────────────────────────────────────────────────\n");
            output.push_str("  JUMP TARGETS\n");
            output.push_str("───────────────────────────────────────────────────────────────\n");
            for target in &jumps {
                output.push_str(&format!("    0x{:04X}\n", target));
            }
        }

        // Gas estimation
        let estimated_gas = self.estimate_gas();
        output.push_str("\n───────────────────────────────────────────────────────────────\n");
        output.push_str("  GAS ESTIMATION\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("    Minimum:    {}\n", estimated_gas.0));
        output.push_str(&format!("    Maximum:    {}\n", estimated_gas.1));

        // Disassembly
        output.push_str("\n───────────────────────────────────────────────────────────────\n");
        output.push_str("  DISASSEMBLY\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        let disasm = Disassembler::new(self.script);
        output.push_str(&disasm.disassemble());

        output.push_str("\n═══════════════════════════════════════════════════════════════\n");

        output
    }

    fn collect_opcode_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        let disasm = Disassembler::new(self.script);
        let mut ip = 0;

        while ip < self.script.len() {
            let (name, size) = disasm.decode_instruction(ip);
            let opcode_name = name.split_whitespace().next().unwrap_or(&name).to_string();
            *stats.entry(opcode_name).or_insert(0) += 1;
            ip += size;
        }

        stats
    }

    fn find_jump_targets(&self) -> Vec<usize> {
        let mut targets = Vec::new();
        let mut ip = 0;

        while ip < self.script.len() {
            let op = self.script[ip];
            match op {
                0x22 | 0x24 | 0x26 | 0x28 | 0x2A | 0x2C | 0x2E | 0x30 | 0x32 | 0x34 => {
                    // 1-byte offset jumps
                    if ip + 1 < self.script.len() {
                        let offset = self.script[ip + 1] as i8;
                        let target = (ip as isize + offset as isize) as usize;
                        if !targets.contains(&target) {
                            targets.push(target);
                        }
                    }
                    ip += 2;
                }
                0x23 | 0x25 | 0x27 | 0x29 | 0x2B | 0x2D | 0x2F | 0x31 | 0x33 | 0x35 => {
                    // 4-byte offset jumps
                    if ip + 4 < self.script.len() {
                        let offset = i32::from_le_bytes([
                            self.script[ip + 1],
                            self.script[ip + 2],
                            self.script[ip + 3],
                            self.script[ip + 4],
                        ]);
                        let target = (ip as isize + offset as isize) as usize;
                        if !targets.contains(&target) {
                            targets.push(target);
                        }
                    }
                    ip += 5;
                }
                _ => {
                    ip += 1;
                    // Skip operand bytes using OpCode metadata
                    if let Ok(opcode) = OpCode::try_from(op) {
                        let size = opcode.operand_size();
                        // PUSHDATA: operand_size is the length-prefix size; read actual data length
                        match opcode {
                            OpCode::PUSHDATA1 if ip < self.script.len() => {
                                ip = ip
                                    .saturating_add(1 + self.script[ip] as usize)
                                    .min(self.script.len());
                            }
                            OpCode::PUSHDATA2 if ip + 1 < self.script.len() => {
                                let len = u16::from_le_bytes([self.script[ip], self.script[ip + 1]])
                                    as usize;
                                ip = ip.saturating_add(2 + len).min(self.script.len());
                            }
                            OpCode::PUSHDATA4 if ip + 3 < self.script.len() => {
                                let len = u32::from_le_bytes([
                                    self.script[ip],
                                    self.script[ip + 1],
                                    self.script[ip + 2],
                                    self.script[ip + 3],
                                ]) as usize;
                                ip = ip
                                    .saturating_add(4)
                                    .saturating_add(len)
                                    .min(self.script.len());
                            }
                            _ => ip += size,
                        }
                    }
                }
            }
        }

        targets.sort();
        targets
    }

    fn estimate_gas(&self) -> (u64, u64) {
        let mut min_gas = 0u64;
        let mut max_gas = 0u64;
        let mut ip = 0;

        while ip < self.script.len() {
            let op = self.script[ip];
            let cost = match op {
                0x00..=0x20 => 1, // push constants
                0x21..=0x40 => 2, // flow control, NOP, RET
                0x41 => 512,      // SYSCALL, host cost depends on target API
                0x42..=0x9F => 2, // stack, slot, splice, buffer, bitwise, INC/DEC, ADD/SUB
                0xA0..=0xDF => 8, // arithmetic (MUL+), comparison, compound types
                0xE0..=0xEF => 2, // ABORTMSG, ASSERTMSG, reserved
                _ => 1,
            };
            min_gas += cost;
            max_gas += cost;
            ip += 1;
            // Skip operand bytes
            if let Ok(opcode) = OpCode::try_from(op) {
                match opcode {
                    OpCode::PUSHDATA1 if ip < self.script.len() => {
                        ip = ip
                            .saturating_add(1 + self.script[ip] as usize)
                            .min(self.script.len());
                    }
                    OpCode::PUSHDATA2 if ip + 1 < self.script.len() => {
                        let len =
                            u16::from_le_bytes([self.script[ip], self.script[ip + 1]]) as usize;
                        ip = ip.saturating_add(2 + len).min(self.script.len());
                    }
                    OpCode::PUSHDATA4 if ip + 3 < self.script.len() => {
                        let len = u32::from_le_bytes([
                            self.script[ip],
                            self.script[ip + 1],
                            self.script[ip + 2],
                            self.script[ip + 3],
                        ]) as usize;
                        ip = ip
                            .saturating_add(4)
                            .saturating_add(len)
                            .min(self.script.len());
                    }
                    _ => ip += opcode.operand_size(),
                }
            }
        }

        // Account for potential loops (rough estimate)
        max_gas *= 10;

        (min_gas, max_gas)
    }
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
