//! Neo VM Execution Engine

use crate::stack_item::StackItem;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),
    #[error("Out of gas")]
    OutOfGas,
}

#[derive(Debug, Clone)]
pub enum VMState {
    None,
    Halt,
    Fault,
    Break,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub script: Vec<u8>,
    pub ip: usize,
}

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
        self.invocation_stack.push(ExecutionContext { script, ip: 0 });
    }

    pub fn execute_next(&mut self) -> Result<(), VMError> {
        let ctx = self.invocation_stack.last_mut()
            .ok_or(VMError::StackUnderflow)?;
        
        if ctx.ip >= ctx.script.len() {
            self.state = VMState::Halt;
            return Ok(());
        }

        let op = ctx.script[ctx.ip];
        ctx.ip += 1;
        self.execute_op(op)
    }

    fn execute_op(&mut self, op: u8) -> Result<(), VMError> {
        match op {
            0x10 => self.eval_stack.push(StackItem::Integer(0)),
            0x11..=0x20 => {
                let n = (op - 0x10) as i128;
                self.eval_stack.push(StackItem::Integer(n));
            }
            0x0F => self.eval_stack.push(StackItem::Integer(-1)),
            0x0B => self.eval_stack.push(StackItem::Null),
            0x45 => { self.eval_stack.pop(); }
            0x4A => {
                let item = self.eval_stack.last()
                    .ok_or(VMError::StackUnderflow)?.clone();
                self.eval_stack.push(item);
            }
            0x9E => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a + b));
            }
            0x40 => {
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
