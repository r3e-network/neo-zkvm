//! Neo VM Execution Engine

use crate::stack_item::StackItem;
use sha2::{Sha256, Digest};
use ripemd::Ripemd160;
use k256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
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
    #[error("Invalid type")]
    InvalidType,
    #[error("Unknown syscall: {0}")]
    UnknownSyscall(u32),
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

/// Built-in syscall IDs
pub mod syscall {
    pub const SYSTEM_RUNTIME_LOG: u32 = 0x01;
    pub const SYSTEM_RUNTIME_NOTIFY: u32 = 0x02;
    pub const SYSTEM_RUNTIME_GETTIME: u32 = 0x03;
    pub const SYSTEM_STORAGE_GET: u32 = 0x10;
    pub const SYSTEM_STORAGE_PUT: u32 = 0x11;
    pub const SYSTEM_STORAGE_DELETE: u32 = 0x12;
}

pub struct NeoVM {
    pub state: VMState,
    pub eval_stack: Vec<StackItem>,
    pub invocation_stack: Vec<ExecutionContext>,
    pub gas_consumed: u64,
    pub gas_limit: u64,
    pub notifications: Vec<StackItem>,
    pub logs: Vec<String>,
}

impl NeoVM {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            state: VMState::None,
            eval_stack: Vec::new(),
            invocation_stack: Vec::new(),
            gas_consumed: 0,
            gas_limit,
            notifications: Vec::new(),
            logs: Vec::new(),
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
            // NOP
            0x21 => {}
            // JMP (1-byte offset)
            0x22 => {
                let ctx = self.invocation_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip = ((ctx.ip as isize - 1) + offset as isize) as usize;
            }
            // JMPIF (1-byte offset)
            0x24 => {
                let ctx = self.invocation_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if cond.to_bool() {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // JMPIFNOT (1-byte offset)
            0x26 => {
                let ctx = self.invocation_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if !cond.to_bool() {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // CALL (1-byte offset)
            0x34 => {
                let ctx = self.invocation_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                let return_ip = ctx.ip + 1;
                let target_ip = ((ctx.ip as isize - 1) + offset as isize) as usize;
                let script = ctx.script.clone();
                self.invocation_stack.push(ExecutionContext { 
                    script, 
                    ip: target_ip,
                });
                // Store return address (simplified)
                self.eval_stack.push(StackItem::Pointer(return_ip as u32));
            }
            // SHA256
            0xF0 => {
                let data = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let bytes = match data {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    StackItem::Integer(i) => i.to_le_bytes().to_vec(),
                    _ => return Err(VMError::InvalidType),
                };
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let result = hasher.finalize().to_vec();
                self.eval_stack.push(StackItem::ByteString(result));
            }
            // RIPEMD160
            0xF1 => {
                let data = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let bytes = match data {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    StackItem::Integer(i) => i.to_le_bytes().to_vec(),
                    _ => return Err(VMError::InvalidType),
                };
                let mut hasher = Ripemd160::new();
                hasher.update(&bytes);
                let result = hasher.finalize().to_vec();
                self.eval_stack.push(StackItem::ByteString(result));
            }
            // SHA256 + RIPEMD160 (Hash160)
            0xF2 => {
                let data = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let bytes = match data {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    StackItem::Integer(i) => i.to_le_bytes().to_vec(),
                    _ => return Err(VMError::InvalidType),
                };
                let sha_result = Sha256::digest(&bytes);
                let result = Ripemd160::digest(&sha_result).to_vec();
                self.eval_stack.push(StackItem::ByteString(result));
            }
            // CHECKSIG (ECDSA secp256k1)
            0xF3 => {
                let pubkey = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let sig = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let msg = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                
                let pubkey_bytes = match pubkey {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    _ => return Err(VMError::InvalidType),
                };
                let sig_bytes = match sig {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    _ => return Err(VMError::InvalidType),
                };
                let msg_bytes = match msg {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    _ => return Err(VMError::InvalidType),
                };
                
                let result = (|| {
                    let vk = VerifyingKey::from_sec1_bytes(&pubkey_bytes).ok()?;
                    let signature = Signature::from_slice(&sig_bytes).ok()?;
                    let msg_hash = Sha256::digest(&msg_bytes);
                    vk.verify(&msg_hash, &signature).ok()
                })();
                
                self.eval_stack.push(StackItem::Boolean(result.is_some()));
            }
            // SYSCALL
            0x41 => {
                let ctx = self.invocation_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                // Read 4-byte syscall ID
                let id = u32::from_le_bytes([
                    ctx.script[ctx.ip],
                    ctx.script[ctx.ip + 1],
                    ctx.script[ctx.ip + 2],
                    ctx.script[ctx.ip + 3],
                ]);
                ctx.ip += 4;
                self.execute_syscall(id)?;
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

    fn execute_syscall(&mut self, id: u32) -> Result<(), VMError> {
        match id {
            syscall::SYSTEM_RUNTIME_LOG => {
                let msg = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if let StackItem::ByteString(b) = msg {
                    if let Ok(s) = String::from_utf8(b) {
                        self.logs.push(s);
                    }
                }
                Ok(())
            }
            syscall::SYSTEM_RUNTIME_NOTIFY => {
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.notifications.push(item);
                Ok(())
            }
            syscall::SYSTEM_RUNTIME_GETTIME => {
                // Return a mock timestamp for zkVM
                self.eval_stack.push(StackItem::Integer(0));
                Ok(())
            }
            _ => Err(VMError::UnknownSyscall(id)),
        }
    }
}
