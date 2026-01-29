//! Neo zkVM CLI - Complete development toolkit
//!
//! A comprehensive command-line interface for Neo zkVM development,
//! including execution, debugging, assembly, and proof generation.

use neo_vm_core::{NeoVM, VMState};
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProverConfig};
use neo_zkvm_verifier::verify;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

mod assembler;
mod disassembler;

use assembler::Assembler;
use disassembler::Disassembler;

const VERSION: &str = "0.2.0";

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
    debug <script>      Interactive step-by-step debugger
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

    # Debug interactively
    neo-zkvm debug 12139E40

    # Inspect script structure
    neo-zkvm inspect 12139E40

    # Generate ZK proof
    neo-zkvm prove 12139E40

For more information, visit: https://github.com/neonlabsorg/neo-zkvm"#,
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

    let mut vm = NeoVM::new(gas_limit);
    vm.load_script(script);

    println!("Executing script...\n");

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        if let Err(e) = vm.execute_next() {
            return Err(format!("Execution failed: {}", e));
        }
    }

    println!("═══════════════════════════════════════");
    println!("  EXECUTION RESULT");
    println!("═══════════════════════════════════════");
    println!("  State:        {:?}", vm.state);
    println!("  Gas consumed: {}", vm.gas_consumed);
    println!("  Stack depth:  {}", vm.eval_stack.len());
    println!("───────────────────────────────────────");

    if !vm.eval_stack.is_empty() {
        println!("  Stack (top → bottom):");
        for (i, item) in vm.eval_stack.iter().rev().enumerate() {
            println!("    [{}] {:?}", i, item);
        }
    } else {
        println!("  Stack: (empty)");
    }

    if !vm.logs.is_empty() {
        println!("───────────────────────────────────────");
        println!("  Logs:");
        for log in &vm.logs {
            println!("    {}", log);
        }
    }

    println!("═══════════════════════════════════════");

    Ok(())
}

fn cmd_prove(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "Missing script argument.\n\nUsage: neo-zkvm prove <script>\n\nExamples:\n  \
             neo-zkvm prove 12139E40\n  neo-zkvm prove script.bin"
                .to_string(),
        );
    }

    let script = parse_script(&args[0])?;
    let gas_limit = parse_gas_limit(args)?;

    println!("Generating ZK proof...\n");

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit,
    };

    let prover = NeoProver::new(ProverConfig::default());
    let proof = prover.prove(input);

    println!("═══════════════════════════════════════");
    println!("  PROOF GENERATION RESULT");
    println!("═══════════════════════════════════════");
    println!("  Result:   {:?}", proof.output.result);
    println!("  Verified: {}", verify(&proof));
    println!("═══════════════════════════════════════");

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
    let gas_limit = parse_gas_limit(args)?;

    let mut debugger = Debugger::new(script, gas_limit);
    debugger.run()?;

    Ok(())
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
        fs::read(input).map_err(|e| format!("Failed to read file '{}': {}", input, e))
    } else {
        let hex_str = input.trim_start_matches("0x");
        hex::decode(hex_str).map_err(|e| format!("Invalid hex string: {}", e))
    }
}

fn parse_gas_limit(args: &[String]) -> Result<u64, String> {
    for (i, arg) in args.iter().enumerate() {
        if (arg == "--gas" || arg == "-g") && i + 1 < args.len() {
            return args[i + 1]
                .parse()
                .map_err(|_| "Invalid gas limit value".to_string());
        }
    }
    Ok(1_000_000) // Default gas limit
}

// ============================================================================
// Debugger
// ============================================================================

struct Debugger {
    vm: NeoVM,
    script: Vec<u8>,
    breakpoints: Vec<usize>,
    history: Vec<String>,
}

impl Debugger {
    fn new(script: Vec<u8>, gas_limit: u64) -> Self {
        let mut vm = NeoVM::new(gas_limit);
        vm.load_script(script.clone());
        Self {
            vm,
            script,
            breakpoints: Vec::new(),
            history: Vec::new(),
        }
    }

    fn run(&mut self) -> Result<(), String> {
        println!("Neo zkVM Debugger v{}", VERSION);
        println!("Type 'help' for available commands.\n");

        self.print_current_state();

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            print!("(neodbg) ");
            stdout.flush().unwrap();

            let mut line = String::new();
            if stdin.lock().read_line(&mut line).is_err() {
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                // Repeat last command
                if let Some(last) = self.history.last().cloned() {
                    self.execute_command(&last)?;
                }
                continue;
            }

            self.history.push(line.to_string());

            if self.execute_command(line)? {
                break;
            }
        }

        Ok(())
    }

    fn execute_command(&mut self, cmd: &str) -> Result<bool, String> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        match parts[0] {
            "help" | "h" => self.cmd_help(),
            "step" | "s" | "n" => self.cmd_step(),
            "continue" | "c" => self.cmd_continue(),
            "run" | "r" => self.cmd_run_to_end(),
            "break" | "b" => self.cmd_breakpoint(&parts[1..]),
            "delete" | "d" => self.cmd_delete_breakpoint(&parts[1..]),
            "info" | "i" => self.cmd_info(&parts[1..]),
            "print" | "p" => self.cmd_print(&parts[1..]),
            "stack" => self.cmd_stack(),
            "disasm" => self.cmd_disasm(),
            "reset" => self.cmd_reset(),
            "quit" | "q" | "exit" => return Ok(true),
            _ => {
                println!("Unknown command: '{}'. Type 'help' for available commands.", parts[0]);
            }
        }

        Ok(false)
    }

    fn cmd_help(&self) {
        println!(
            r#"
Available commands:
  step, s, n          Execute next instruction
  continue, c         Continue until breakpoint or halt
  run, r              Run to completion
  break <addr>, b     Set breakpoint at address (hex)
  delete <addr>, d    Delete breakpoint
  info breakpoints    List all breakpoints
  info registers      Show VM state
  print <n>, p        Print stack item at index n
  stack               Show full stack
  disasm              Disassemble current script
  reset               Reset VM to initial state
  quit, q, exit       Exit debugger
"#
        );
    }

    fn cmd_step(&mut self) {
        if matches!(self.vm.state, VMState::Halt | VMState::Fault) {
            println!("Program has terminated. Use 'reset' to restart.");
            return;
        }

        if let Err(e) = self.vm.execute_next() {
            println!("Error: {}", e);
        }

        self.print_current_state();
    }

    fn cmd_continue(&mut self) {
        while !matches!(self.vm.state, VMState::Halt | VMState::Fault) {
            let ip = self.get_current_ip();
            if self.breakpoints.contains(&ip) && !self.history.last().map(|s| s.starts_with("continue")).unwrap_or(false) {
                println!("Breakpoint hit at 0x{:04X}", ip);
                break;
            }

            if let Err(e) = self.vm.execute_next() {
                println!("Error: {}", e);
                break;
            }

            // Check breakpoint after execution
            let new_ip = self.get_current_ip();
            if self.breakpoints.contains(&new_ip) {
                println!("Breakpoint hit at 0x{:04X}", new_ip);
                self.print_current_state();
                return;
            }
        }

        self.print_current_state();
    }

    fn cmd_run_to_end(&mut self) {
        while !matches!(self.vm.state, VMState::Halt | VMState::Fault) {
            if let Err(e) = self.vm.execute_next() {
                println!("Error: {}", e);
                break;
            }
        }

        self.print_current_state();
    }

    fn cmd_breakpoint(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: break <address>");
            return;
        }

        let addr_str = args[0].trim_start_matches("0x");
        match usize::from_str_radix(addr_str, 16) {
            Ok(addr) => {
                if !self.breakpoints.contains(&addr) {
                    self.breakpoints.push(addr);
                    println!("Breakpoint set at 0x{:04X}", addr);
                } else {
                    println!("Breakpoint already exists at 0x{:04X}", addr);
                }
            }
            Err(_) => println!("Invalid address: {}", args[0]),
        }
    }

    fn cmd_delete_breakpoint(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: delete <address>");
            return;
        }

        let addr_str = args[0].trim_start_matches("0x");
        match usize::from_str_radix(addr_str, 16) {
            Ok(addr) => {
                if let Some(pos) = self.breakpoints.iter().position(|&x| x == addr) {
                    self.breakpoints.remove(pos);
                    println!("Breakpoint removed at 0x{:04X}", addr);
                } else {
                    println!("No breakpoint at 0x{:04X}", addr);
                }
            }
            Err(_) => println!("Invalid address: {}", args[0]),
        }
    }

    fn cmd_info(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: info <breakpoints|registers>");
            return;
        }

        match args[0] {
            "breakpoints" | "b" => {
                if self.breakpoints.is_empty() {
                    println!("No breakpoints set.");
                } else {
                    println!("Breakpoints:");
                    for (i, bp) in self.breakpoints.iter().enumerate() {
                        println!("  {}: 0x{:04X}", i + 1, bp);
                    }
                }
            }
            "registers" | "r" => {
                println!("VM State:");
                println!("  State:        {:?}", self.vm.state);
                println!("  IP:           0x{:04X}", self.get_current_ip());
                println!("  Gas consumed: {}", self.vm.gas_consumed);
                println!("  Gas limit:    {}", self.vm.gas_limit);
                println!("  Stack depth:  {}", self.vm.eval_stack.len());
            }
            _ => println!("Unknown info type: {}", args[0]),
        }
    }

    fn cmd_print(&self, args: &[&str]) {
        if args.is_empty() {
            if let Some(top) = self.vm.eval_stack.last() {
                println!("Top: {:?}", top);
            } else {
                println!("Stack is empty.");
            }
            return;
        }

        match args[0].parse::<usize>() {
            Ok(idx) => {
                let len = self.vm.eval_stack.len();
                if idx < len {
                    println!("[{}]: {:?}", idx, self.vm.eval_stack[len - 1 - idx]);
                } else {
                    println!("Index out of range (stack depth: {})", len);
                }
            }
            Err(_) => println!("Invalid index: {}", args[0]),
        }
    }

    fn cmd_stack(&self) {
        if self.vm.eval_stack.is_empty() {
            println!("Stack is empty.");
        } else {
            println!("Stack (top → bottom):");
            for (i, item) in self.vm.eval_stack.iter().rev().enumerate() {
                println!("  [{}] {:?}", i, item);
            }
        }
    }

    fn cmd_disasm(&self) {
        let disasm = Disassembler::new(&self.script);
        println!("{}", disasm.disassemble());
    }

    fn cmd_reset(&mut self) {
        self.vm = NeoVM::new(self.vm.gas_limit);
        self.vm.load_script(self.script.clone());
        println!("VM reset to initial state.");
        self.print_current_state();
    }

    fn get_current_ip(&self) -> usize {
        self.vm
            .invocation_stack
            .last()
            .map(|ctx| ctx.ip)
            .unwrap_or(0)
    }

    fn print_current_state(&self) {
        if matches!(self.vm.state, VMState::Halt) {
            println!("Program halted. Gas consumed: {}", self.vm.gas_consumed);
            return;
        }

        if matches!(self.vm.state, VMState::Fault) {
            println!("Program faulted!");
            return;
        }

        let ip = self.get_current_ip();
        if ip < self.script.len() {
            let op = self.script[ip];
            let disasm = Disassembler::new(&self.script);
            let (name, _) = disasm.decode_instruction(ip);
            println!(
                "→ 0x{:04X}: {:02X}  {}    [gas: {}]",
                ip, op, name, self.vm.gas_consumed
            );
        }
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
            *stats.entry(name).or_insert(0) += 1;
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
                _ => ip += 1,
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
                0x0B..=0x20 => 1,
                0x43..=0x55 => 2,
                0x90..=0xBB => 8,
                0x21..=0x40 => 2,
                0xF0..=0xF2 => 512,
                0xF3 => 32768,
                0x41 => 16,
                _ => 1,
            };
            min_gas += cost;
            max_gas += cost;
            ip += 1;
        }

        // Account for potential loops (rough estimate)
        max_gas *= 10;

        (min_gas, max_gas)
    }
}
