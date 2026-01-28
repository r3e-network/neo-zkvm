//! Neo VM Execution Engine

use crate::opcode::OpCode;
use crate::stack_item::StackItem;
use num_bigint::BigInt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),
    #[error("Out of gas")]
    OutOfGas,
    #[error("Invalid jump")]
    InvalidJump,
    #[error("Assertion failed")]
    AssertionFailed,
}

/// VM execution state
#[derive(Debug, Clone)]
pub enum VMState {
    None,
    Halt,
    Fault,
    Break,
}

/// Execution context (call frame)
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub script: Vec<u8>,
    pub ip: usize,
    pub local_vars: Vec<StackItem>,
    pub arguments: Vec<StackItem>,
    pub static_fields: Vec<StackItem>,
}

/// Neo Virtual Machine
pub struct NeoVM {
    pub state: VMState,
    pub eval_stack: Vec<StackItem>,
    pub invocation_stack: Vec<ExecutionContext>,
    pub gas_consumed: u64,
    pub gas_limit: u64,
}

impl NeoVM {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            state: VMState::None,
            eval_stack: Vec::new(),
            invocation_stack: Vec::new(),
            gas_consumed: 0,
            gas_limit,
        }
    }

    pub fn load_script(&mut self, script: Vec<u8>) {
        let ctx = ExecutionContext {
            script,
            ip: 0,
            local_vars: Vec::new(),
            arguments: Vec::new(),
            static_fields: Vec::new(),
        };
        self.invocation_stack.push(ctx);
    }

    /// Execute one instruction
    pub fn execute_next(&mut self) -> Result<(), VMError> {
        let ctx = self.invocation_stack.last_mut()
            .ok_or(VMError::StackUnderflow)?;
        
        if ctx.ip >= ctx.script.len() {
            self.state = VMState::Halt;
            return Ok(());
        }

        let opcode = ctx.script[ctx.ip];
        ctx.ip += 1;
        self.execute_opcode(opcode)
    }

    fn pop(&mut self) -> Result<StackItem, VMError> {
        self.eval_stack.pop().ok_or(VMError::StackUnderflow)
    }

    fn push(&mut self, item: StackItem) {
        self.eval_stack.push(item);
    }

    fn execute_opcode(&mut self, op: u8) -> Result<(), VMError> {
        match op {
            0x10 => self.push(StackItem::Integer(BigInt::from(0))),
            0x11..=0x20 => {
                let n = (op - 0x10) as i32;
                self.push(StackItem::Integer(BigInt::from(n)));
            }
            0x0F => self.push(StackItem::Integer(BigInt::from(-1))),
            0x0B => self.push(StackItem::Null),
            _ => return self.execute_opcode_ext(op),
        }
        Ok(())
    }

    fn execute_opcode_ext(&mut self, op: u8) -> Result<(), VMError> {
        match op {
            // Stack ops
            0x45 => { self.pop()?; } // DROP
            0x4A => { // DUP
                let item = self.eval_stack.last()
                    .ok_or(VMError::StackUnderflow)?.clone();
                self.push(item);
            }
            // Arithmetic
            0x9E => { // ADD
                let b = self.pop()?.to_integer().unwrap();
                let a = self.pop()?.to_integer().unwrap();
                self.push(StackItem::Integer(a + b));
            }
            0x9F => { // SUB
                let b = self.pop()?.to_integer().unwrap();
                let a = self.pop()?.to_integer().unwrap();
                self.push(StackItem::Integer(a - b));
            }
            0xA0 => { // MUL
                let b = self.pop()?.to_integer().unwrap();
                let a = self.pop()?.to_integer().unwrap();
                self.push(StackItem::Integer(a * b));
            }
            0x40 => { // RET
                self.invocation_stack.pop();
                if self.invocation_stack.is_empty() {
                    self.state = VMState::Halt;
                }
            }
            _ => return Err(VMError::InvalidOpcode(op)),
        }
        Ok(())
    }
}
