use neo_vm_rs::{interop_hash, last_interpreter_ip, pop_byte_arg, StackValue, SyscallProvider};
use sha2::{Digest, Sha256};

use crate::{disassembler::Disassembler, trace_step::TraceStep};

pub(crate) struct TraceHost {
    script: Vec<u8>,
    steps: Vec<TraceStep>,
}

impl TraceHost {
    pub(crate) fn new(script: Vec<u8>) -> Self {
        Self {
            script,
            steps: Vec::new(),
        }
    }

    pub(crate) fn print_trace(&self) {
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
            // Disassembler::new is cheap (reference-only); decode_instruction
            // does the actual work. For the trace/debug path this is fine —
            // production execution doesn't use TraceHost.
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
