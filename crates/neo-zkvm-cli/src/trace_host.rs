use neo_vm_guest::try_crypto_syscall;
use neo_vm_rs::{StackValue, SyscallProvider, last_interpreter_ip};

use crate::{disassembler::Disassembler, trace_step::TraceStep};

pub(crate) struct TraceHost {
    script: Vec<u8>,
    steps: Vec<TraceStep>,
    gas_limit: u64,
    steps_executed: u64,
}

impl TraceHost {
    pub(crate) fn new(script: Vec<u8>, gas_limit: u64) -> Self {
        Self {
            script,
            steps: Vec::new(),
            gas_limit,
            steps_executed: 0,
        }
    }

    pub(crate) fn steps_executed(&self) -> u64 {
        self.steps_executed
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
        println!(
            "  ({} steps, gas limit {})",
            self.steps_executed, self.gas_limit
        );
    }
}

impl SyscallProvider for TraceHost {
    fn on_instruction(&mut self, opcode: u8) -> Result<(), String> {
        self.steps_executed = self.steps_executed.saturating_add(1);
        if self.steps_executed > self.gas_limit {
            return Err("Out of gas".to_string());
        }

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
        // Reuse the same deterministic crypto adapters as the proof guest so
        // debug traces match prove/run semantics for hash syscalls.
        if try_crypto_syscall(api, stack)? {
            return Ok(());
        }

        Err(format!(
            "unsupported trace syscall 0x{api:08x}; only deterministic crypto adapters \
             (SHA256, RIPEMD160, Hash160, Hash256) are available"
        ))
    }
}
