//! Neo zkVM CLI - Complete development toolkit

use neo_vm_core::{NeoVM, VMState};
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProverConfig};
use neo_zkvm_verifier::verify;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "run" => cmd_run(&args[2..]),
        "prove" => cmd_prove(&args[2..]),
        "asm" => cmd_assemble(&args[2..]),
        "disasm" => cmd_disassemble(&args[2..]),
        "version" | "-v" => println!("neo-zkvm v0.1.0"),
        "help" | "-h" => print_help(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!("Neo zkVM CLI v0.1.0");
    println!();
    println!("USAGE:");
    println!("  neo-zkvm <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  run <script>     Execute a script (hex or file)");
    println!("  prove <script>   Generate ZK proof for script");
    println!("  asm <file>       Assemble source to bytecode");
    println!("  disasm <hex>     Disassemble bytecode");
    println!("  version          Show version");
    println!("  help             Show this help");
}

fn cmd_run(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: neo-zkvm run <hex_script>");
        return;
    }

    let script = parse_script(&args[0]);
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script);

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        if let Err(e) = vm.execute_next() {
            eprintln!("Error: {}", e);
            return;
        }
    }

    println!("State: {:?}", vm.state);
    println!("Gas: {}", vm.gas_consumed);
    println!("Stack: {:?}", vm.eval_stack);
}

fn cmd_prove(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: neo-zkvm prove <hex_script>");
        return;
    }

    let script = parse_script(&args[0]);
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig::default());
    let proof = prover.prove(input);

    println!("Result: {:?}", proof.output.result);
    println!("Verified: {}", verify(&proof));
}

fn cmd_assemble(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: neo-zkvm asm <source>");
        return;
    }

    let source = if args[0].ends_with(".neoasm") {
        fs::read_to_string(&args[0]).unwrap_or_else(|_| args[0].clone())
    } else {
        args[0].clone()
    };

    let bytecode = assemble(&source);
    println!("{}", hex::encode(&bytecode));
}

fn cmd_disassemble(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: neo-zkvm disasm <hex>");
        return;
    }

    let script = parse_script(&args[0]);
    disassemble(&script);
}

fn parse_script(input: &str) -> Vec<u8> {
    if input.ends_with(".nef") || input.ends_with(".bin") {
        fs::read(input).expect("Failed to read file")
    } else {
        hex::decode(input.trim_start_matches("0x")).expect("Invalid hex")
    }
}

fn assemble(source: &str) -> Vec<u8> {
    let mut bytecode = Vec::new();

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0].to_uppercase().as_str() {
            "PUSH0" => bytecode.push(0x10),
            "PUSH1" => bytecode.push(0x11),
            "PUSH2" => bytecode.push(0x12),
            "PUSH3" => bytecode.push(0x13),
            "PUSH4" => bytecode.push(0x14),
            "PUSH5" => bytecode.push(0x15),
            "PUSHNULL" => bytecode.push(0x0B),
            "ADD" => bytecode.push(0x9E),
            "SUB" => bytecode.push(0x9F),
            "MUL" => bytecode.push(0xA0),
            "DIV" => bytecode.push(0xA1),
            "MOD" => bytecode.push(0xA2),
            "DUP" => bytecode.push(0x4A),
            "DROP" => bytecode.push(0x45),
            "SWAP" => bytecode.push(0x50),
            "RET" => bytecode.push(0x40),
            "NOP" => bytecode.push(0x21),
            _ => eprintln!("Unknown: {}", parts[0]),
        }
    }

    bytecode
}

fn disassemble(script: &[u8]) {
    let mut ip = 0;
    while ip < script.len() {
        let op = script[ip];
        let name = opcode_name(op);
        println!("{:04X}: {:02X}  {}", ip, op, name);
        ip += 1;
    }
}

fn opcode_name(op: u8) -> &'static str {
    match op {
        0x10 => "PUSH0",
        0x11..=0x20 => "PUSH1-16",
        0x0B => "PUSHNULL",
        0x0F => "PUSHM1",
        0x21 => "NOP",
        0x40 => "RET",
        0x45 => "DROP",
        0x4A => "DUP",
        0x50 => "SWAP",
        0x9E => "ADD",
        0x9F => "SUB",
        0xA0 => "MUL",
        0xA1 => "DIV",
        0xA2 => "MOD",
        _ => "UNKNOWN",
    }
}
