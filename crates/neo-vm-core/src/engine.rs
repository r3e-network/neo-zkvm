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
    #[error("Division by zero")]
    DivisionByZero,
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
            // ADD
            0x9E => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a + b));
            }
            // SUB
            0x9F => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a - b));
            }
            // MUL
            0xA0 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a * b));
            }
            // DIV
            0xA1 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.eval_stack.push(StackItem::Integer(a / b));
            }
            // MOD
            0xA2 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.eval_stack.push(StackItem::Integer(a % b));
            }
            // LT
            0xB5 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a < b));
            }
            // LE
            0xB6 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a <= b));
            }
            // GT
            0xB7 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a > b));
            }
            // GE
            0xB8 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a >= b));
            }
            // EQUAL
            0x97 => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a == b));
            }
            // AND (bitwise)
            0x91 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a & b));
            }
            // OR (bitwise)
            0x92 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a | b));
            }
            // XOR (bitwise)
            0x93 => {
                let b = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a ^ b));
            }
            // NOT (logical)
            0xAA => {
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(!a.to_bool()));
            }
            // BOOLAND
            0xAB => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a.to_bool() && b.to_bool()));
            }
            // BOOLOR
            0xAC => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a.to_bool() || b.to_bool()));
            }
            // SWAP
            0x50 => {
                let len = self.eval_stack.len();
                if len < 2 {
                    return Err(VMError::StackUnderflow);
                }
                self.eval_stack.swap(len - 1, len - 2);
            }
            // ROT
            0x51 => {
                let len = self.eval_stack.len();
                if len < 3 {
                    return Err(VMError::StackUnderflow);
                }
                let item = self.eval_stack.remove(len - 3);
                self.eval_stack.push(item);
            }
            // PICK
            0x4D => {
                let n = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)? as usize;
                let len = self.eval_stack.len();
                if n >= len {
                    return Err(VMError::StackUnderflow);
                }
                let item = self.eval_stack[len - 1 - n].clone();
                self.eval_stack.push(item);
            }
            // ROLL
            0x52 => {
                let n = self.eval_stack.pop().and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)? as usize;
                let len = self.eval_stack.len();
                if n >= len {
                    return Err(VMError::StackUnderflow);
                }
                let item = self.eval_stack.remove(len - 1 - n);
                self.eval_stack.push(item);
            }
            // OVER
            0x4B => {
                let len = self.eval_stack.len();
                if len < 2 {
                    return Err(VMError::StackUnderflow);
                }
                let item = self.eval_stack[len - 2].clone();
                self.eval_stack.push(item);
            }
            // DEPTH
            0x43 => {
                let depth = self.eval_stack.len() as i128;
                self.eval_stack.push(StackItem::Integer(depth));
            }
            // RET
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
