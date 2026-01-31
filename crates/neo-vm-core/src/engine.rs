//! Neo VM Execution Engine
//!
//! Neo VM Engine
//!
//! Core execution engine for Neo zkVM.

use crate::stack_item::StackItem;
use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
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
    #[error("Invalid operation")]
    InvalidOperation,
    #[error("Invalid script")]
    InvalidScript,
    #[error("Invalid public key format for CHECKSIG")]
    InvalidPublicKey,
    #[error("Invalid signature format for CHECKSIG")]
    InvalidSignature,
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
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

// SAFETY: ExecutionContext is designed for single-threaded use within NeoVM.
unsafe impl Send for ExecutionContext {}
unsafe impl Sync for ExecutionContext {}

/// Built-in syscall IDs (Neo N3 compatible)
pub mod syscall {
    pub const SYSTEM_RUNTIME_LOG: u32 = 0x01;
    pub const SYSTEM_RUNTIME_NOTIFY: u32 = 0x02;
    pub const SYSTEM_RUNTIME_GETTIME: u32 = 0x03;
    pub const SYSTEM_STORAGE_GET: u32 = 0x10;
    pub const SYSTEM_STORAGE_PUT: u32 = 0x11;
    pub const SYSTEM_STORAGE_DELETE: u32 = 0x12;
}

/// Gas cost lookup table for O(1) opcode cost retrieval
/// Uses u16 to support CHECKSIG's high gas cost (32768)
const GAS_COSTS: [u16; 256] = [
    // 0x00-0x0F (PUSHINT8-PUSHM1)
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x10-0x1F (PUSH0-PUSH16)
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x20-0x2F
    1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0x30-0x3F (flow control)
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    // 0x40-0x4F (RET, DEPTH, CLEAR, stack ops)
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0x50-0x5F (stack ops)
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0x60-0x6F (slot ops)
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0x70-0x7F (slot ops)
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0x80-0x8F (splice/buffer ops)
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0x90-0x9F (bitwise/invert/equality)
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, // 0xA0-0xAF (arithmetic)
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, // 0xB0-0xBF (comparison/min/max/within)
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, // 0xC0-0xCF (compound types)
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, // 0xD0-0xDF (compound types)
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xE0-0xEF (reserved)
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    // 0xF0-0xFF (crypto: SHA256, RIPEMD160, CHECKSIG)
    512, 512, 512, 32768, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

#[inline]
fn get_gas_cost(op: u8) -> u64 {
    GAS_COSTS[op as usize] as u64
}

/// Maximum script size in bytes (1MB)
pub const MAX_SCRIPT_SIZE: usize = 1024 * 1024;

/// Execution trace step for proof generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceStep {
    pub ip: usize,
    pub opcode: u8,
    pub stack_hash: [u8; 32],
    pub gas_consumed: u64,
}

/// Full execution trace
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExecutionTrace {
    pub steps: Vec<TraceStep>,
    pub initial_state_hash: [u8; 32],
    pub final_state_hash: [u8; 32],
}

pub struct NeoVM {
    pub state: VMState,
    pub eval_stack: Vec<StackItem>,
    pub invocation_stack: Vec<ExecutionContext>,
    pub gas_consumed: u64,
    pub gas_limit: u64,
    pub notifications: Vec<StackItem>,
    pub logs: Vec<String>,
    pub trace: ExecutionTrace,
    pub tracing_enabled: bool,
    // Slot support for Neo VM compatibility
    pub local_slots: Vec<StackItem>,
    pub argument_slots: Vec<StackItem>,
    pub static_slots: Vec<StackItem>,
}

impl NeoVM {
    /// Default stack capacity for pre-allocation
    const DEFAULT_STACK_CAPACITY: usize = 64;
    /// Default invocation depth capacity
    const DEFAULT_INVOCATION_CAPACITY: usize = 8;

    #[inline]
    pub fn new(gas_limit: u64) -> Self {
        Self {
            state: VMState::None,
            eval_stack: Vec::with_capacity(Self::DEFAULT_STACK_CAPACITY),
            invocation_stack: Vec::with_capacity(Self::DEFAULT_INVOCATION_CAPACITY),
            gas_consumed: 0,
            gas_limit,
            notifications: Vec::new(),
            logs: Vec::new(),
            trace: ExecutionTrace::default(),
            tracing_enabled: false,
            local_slots: Vec::with_capacity(Self::DEFAULT_STACK_CAPACITY),
            argument_slots: Vec::with_capacity(Self::DEFAULT_STACK_CAPACITY),
            static_slots: Vec::with_capacity(Self::DEFAULT_STACK_CAPACITY),
        }
    }

    /// Run the VM until halt or fault
    #[inline]
    pub fn run(&mut self) {
        while !matches!(self.state, VMState::Halt | VMState::Fault) {
            if self.execute_next().is_err() {
                self.state = VMState::Fault;
                break;
            }
        }
    }

    #[inline]
    pub fn enable_tracing(&mut self) {
        self.tracing_enabled = true;
        self.trace.initial_state_hash = self.compute_state_hash();
    }

    #[inline]
    fn compute_state_hash(&self) -> [u8; 32] {
        use sha2::Digest;
        let mut hasher = Sha256::new();
        for item in &self.eval_stack {
            hasher.update(format!("{:?}", item).as_bytes());
        }
        hasher.update(self.gas_consumed.to_le_bytes());
        hasher.finalize().into()
    }

    #[inline]
    pub fn load_script(&mut self, script: Vec<u8>) -> Result<(), VMError> {
        if script.len() > MAX_SCRIPT_SIZE {
            return Err(VMError::InvalidScript);
        }
        self.invocation_stack
            .push(ExecutionContext { script, ip: 0 });
        Ok(())
    }

    pub fn execute_next(&mut self) -> Result<(), VMError> {
        let ctx = self
            .invocation_stack
            .last_mut()
            .ok_or(VMError::StackUnderflow)?;

        if ctx.ip >= ctx.script.len() {
            self.state = VMState::Halt;
            if self.tracing_enabled {
                self.trace.final_state_hash = self.compute_state_hash();
            }
            return Ok(());
        }

        let ip = ctx.ip;
        let op = ctx.script[ctx.ip];
        ctx.ip += 1;

        // Gas metering
        let gas_cost = get_gas_cost(op);
        self.gas_consumed += gas_cost;
        if self.gas_consumed > self.gas_limit {
            self.state = VMState::Fault;
            return Err(VMError::OutOfGas);
        }

        // Record trace step
        if self.tracing_enabled {
            let step = TraceStep {
                ip,
                opcode: op,
                stack_hash: self.compute_state_hash(),
                gas_consumed: self.gas_consumed,
            };
            self.trace.steps.push(step);
        }

        if let Err(e) = self.execute_op(op) {
            self.state = VMState::Fault;
            return Err(e);
        }
        Ok(())
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
            // PUSHDATA1 - Push data with 1-byte length prefix
            0x0C => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let len = ctx.script[ctx.ip] as usize;
                ctx.ip += 1;
                if ctx.ip + len > ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let data = ctx.script[ctx.ip..ctx.ip + len].to_vec();
                ctx.ip += len;
                self.eval_stack.push(StackItem::ByteString(data));
            }
            // PUSHDATA2 - Push data with 2-byte length prefix
            0x0D => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if ctx.ip + 2 > ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let len = u16::from_le_bytes([ctx.script[ctx.ip], ctx.script[ctx.ip + 1]]) as usize;
                ctx.ip += 2;
                if ctx.ip + len > ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let data = ctx.script[ctx.ip..ctx.ip + len].to_vec();
                ctx.ip += len;
                self.eval_stack.push(StackItem::ByteString(data));
            }
            // PUSHINT8
            0x00 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if ctx.ip >= ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let val = ctx.script[ctx.ip] as i8 as i128;
                ctx.ip += 1;
                self.eval_stack.push(StackItem::Integer(val));
            }
            // PUSHINT16
            0x01 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if ctx.ip + 2 > ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let val = i16::from_le_bytes([ctx.script[ctx.ip], ctx.script[ctx.ip + 1]]) as i128;
                ctx.ip += 2;
                self.eval_stack.push(StackItem::Integer(val));
            }
            0x45 => {
                self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
            }
            0x4A => {
                let item = self
                    .eval_stack
                    .last()
                    .ok_or(VMError::StackUnderflow)?
                    .clone();
                self.eval_stack.push(item);
            }
            // ADD
            0x9E => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let result = a.checked_add(b).ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // SUB
            0x9F => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let result = a.checked_sub(b).ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // MUL
            0xA0 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let result = a.checked_mul(b).ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // DIV
            0xA1 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                let result = a.checked_div(b).ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // MOD
            0xA2 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                let result = a.checked_rem(b).ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // POW
            0xA3 => {
                let exp = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let base = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if exp < 0 {
                    return Err(VMError::InvalidOperation);
                }
                let result = base.pow(exp as u32);
                self.eval_stack.push(StackItem::Integer(result));
            }
            // SHL
            0xA8 => {
                let shift = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let value = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if !(0..=256).contains(&shift) {
                    return Err(VMError::InvalidOperation);
                }
                let result = value
                    .checked_shl(shift as u32)
                    .ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // SHR
            0xA9 => {
                let shift = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let value = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if !(0..=256).contains(&shift) {
                    return Err(VMError::InvalidOperation);
                }
                let result = value
                    .checked_shr(shift as u32)
                    .ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // MIN
            0xB9 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a.min(b)));
            }
            // MAX
            0xBA => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a.max(b)));
            }
            // WITHIN (a <= x < b)
            0xBB => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let x = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a <= x && x < b));
            }
            // SIGN
            0x99 => {
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let sign = if a > 0 {
                    1
                } else if a < 0 {
                    -1
                } else {
                    0
                };
                self.eval_stack.push(StackItem::Integer(sign));
            }
            // ABS
            0x9A => {
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let result = a.checked_abs().ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // NEGATE
            0x9B => {
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let result = a.checked_neg().ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // INC
            0x9C => {
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let result = a.checked_add(1).ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // DEC
            0x9D => {
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let result = a.checked_sub(1).ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(StackItem::Integer(result));
            }
            // LT
            0xB5 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a < b));
            }
            // LE
            0xB6 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a <= b));
            }
            // GT
            0xB7 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a > b));
            }
            // GE
            0xB8 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a >= b));
            }
            // EQUAL
            0x97 => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a == b));
            }
            // NOTEQUAL
            0x98 => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a != b));
            }
            // ISNULL
            0xD8 => {
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.eval_stack
                    .push(StackItem::Boolean(matches!(item, StackItem::Null)));
            }
            // NZ - Not zero
            0xB1 => {
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a != 0));
            }
            // NUMEQUAL
            0xB3 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a == b));
            }
            // NUMNOTEQUAL
            0xB4 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Boolean(a != b));
            }
            // INVERT (bitwise NOT)
            0x90 => {
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(!a));
            }
            // AND (bitwise)
            0x91 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a & b));
            }
            // OR (bitwise)
            0x92 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                self.eval_stack.push(StackItem::Integer(a | b));
            }
            // XOR (bitwise)
            0x93 => {
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
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
                self.eval_stack
                    .push(StackItem::Boolean(a.to_bool() && b.to_bool()));
            }
            // BOOLOR
            0xAC => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.eval_stack
                    .push(StackItem::Boolean(a.to_bool() || b.to_bool()));
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
                let n = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
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
                let n = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
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
            // NIP - Remove second-to-top item
            0x46 => {
                let len = self.eval_stack.len();
                if len < 2 {
                    return Err(VMError::StackUnderflow);
                }
                self.eval_stack.remove(len - 2);
            }
            // XDROP - Remove item at index n
            0x48 => {
                let n = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)? as usize;
                let len = self.eval_stack.len();
                if n >= len {
                    return Err(VMError::StackUnderflow);
                }
                self.eval_stack.remove(len - 1 - n);
            }
            // CLEAR - Clear the stack
            0x49 => {
                self.eval_stack.clear();
            }
            // TUCK - Copy top item and insert before second-to-top
            0x4E => {
                let len = self.eval_stack.len();
                if len < 2 {
                    return Err(VMError::StackUnderflow);
                }
                let item = self.eval_stack[len - 1].clone();
                self.eval_stack.insert(len - 2, item);
            }
            // REVERSE3 - Reverse top 3 items
            0x53 => {
                let len = self.eval_stack.len();
                if len < 3 {
                    return Err(VMError::StackUnderflow);
                }
                self.eval_stack.swap(len - 1, len - 3);
            }
            // REVERSE4 - Reverse top 4 items
            0x54 => {
                let len = self.eval_stack.len();
                if len < 4 {
                    return Err(VMError::StackUnderflow);
                }
                self.eval_stack.swap(len - 1, len - 4);
                self.eval_stack.swap(len - 2, len - 3);
            }
            // REVERSEN - Reverse top n items
            0x55 => {
                let n = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)? as usize;
                let len = self.eval_stack.len();
                if n > len {
                    return Err(VMError::StackUnderflow);
                }
                let start = len - n;
                self.eval_stack[start..].reverse();
            }
            // INITSLOT - Initialize local and argument slots
            0x57 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let local_count = ctx.script[ctx.ip] as usize;
                let arg_count = ctx.script[ctx.ip + 1] as usize;
                ctx.ip += 2;
                self.local_slots = vec![StackItem::Null; local_count];
                // Pop arguments from stack into argument slots
                self.argument_slots = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    let arg = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                    self.argument_slots.push(arg);
                }
                self.argument_slots.reverse();
            }
            // LDLOC0-LDLOC6 - Load local variable 0-6
            0x66..=0x6C => {
                let idx = (op - 0x66) as usize;
                let item = self
                    .local_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(item);
            }
            // LDLOC_S - Load local variable (short form)
            0x6D => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let idx = ctx.script[ctx.ip] as usize;
                ctx.ip += 1;
                let item = self
                    .local_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(item);
            }
            // STLOC0-STLOC6 - Store local variable 0-6
            0x6E..=0x72 => {
                let val = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let idx = (op - 0x6E) as usize;
                if idx >= self.local_slots.len() {
                    self.local_slots.resize(idx + 1, StackItem::Null);
                }
                self.local_slots[idx] = val;
            }
            // STLOC_S - Store local variable (short form)
            0x73 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let idx = ctx.script[ctx.ip] as usize;
                ctx.ip += 1;
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if idx >= self.local_slots.len() {
                    return Err(VMError::InvalidOperation);
                }
                self.local_slots[idx] = item;
            }
            // LDARG0-LDARG6 - Load argument 0-6
            0x74..=0x79 => {
                let idx = (op - 0x74) as usize;
                let item = self
                    .argument_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(item);
            }
            // LDARG - Load argument
            0x7A => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let idx = ctx.script[ctx.ip] as usize;
                ctx.ip += 1;
                let item = self
                    .argument_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.eval_stack.push(item);
            }
            // NOP
            0x21 => {}
            // ASSERT
            0x39 => {
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if !cond.to_bool() {
                    self.state = VMState::Fault;
                    return Err(VMError::InvalidOperation);
                }
            }
            // JMP (1-byte offset)
            0x22 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip = ((ctx.ip as isize - 1) + offset as isize) as usize;
            }
            // JMPIF (1-byte offset)
            0x24 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if cond.to_bool() {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // JMPIFNOT (1-byte offset)
            0x26 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if !cond.to_bool() {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // JMPEQ - Jump if equal
            0x28 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if a == b {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // JMPNE - Jump if not equal
            0x2A => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if a != b {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // JMPGT - Jump if greater than
            0x2C => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if a > b {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // JMPGE - Jump if greater or equal
            0x2E => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if a >= b {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // JMPLT - Jump if less than
            0x30 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if a < b {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // JMPLE - Jump if less or equal
            0x32 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let offset = ctx.script[ctx.ip] as i8;
                ctx.ip += 1;
                let b = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                let a = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)?;
                if a <= b {
                    ctx.ip = ((ctx.ip as isize - 2) + offset as isize) as usize;
                }
            }
            // CALL (1-byte offset)
            0x34 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
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
                let result = Ripemd160::digest(sha_result).to_vec();
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

                let result = VerifyingKey::from_sec1_bytes(&pubkey_bytes)
                    .map_err(|_| VMError::InvalidPublicKey)?;
                let signature =
                    Signature::from_slice(&sig_bytes).map_err(|_| VMError::InvalidSignature)?;
                let msg_hash = Sha256::digest(&msg_bytes);

                let verified = result.verify(&msg_hash, &signature).is_ok();
                self.eval_stack.push(StackItem::Boolean(verified));
            }
            // SYSCALL
            0x41 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
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
            // NEWARRAY0 - Create empty array
            0xC2 => {
                self.eval_stack.push(StackItem::Array(Vec::new()));
            }
            // NEWARRAY - Create array with n elements
            0xC3 => {
                let n = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)? as usize;
                let arr = vec![StackItem::Null; n];
                self.eval_stack.push(StackItem::Array(arr));
            }
            // NEWSTRUCT0 - Create empty struct
            0xC5 => {
                self.eval_stack.push(StackItem::Struct(Vec::new()));
            }
            // NEWSTRUCT - Create struct with n elements
            0xC6 => {
                let n = self
                    .eval_stack
                    .pop()
                    .and_then(|x| x.to_integer())
                    .ok_or(VMError::StackUnderflow)? as usize;
                let s = vec![StackItem::Null; n];
                self.eval_stack.push(StackItem::Struct(s));
            }
            // NEWMAP - Create empty map
            0xC8 => {
                self.eval_stack.push(StackItem::Map(Vec::new()));
            }
            // SIZE - Get size of array/map/string
            0xCA => {
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let size = match &item {
                    StackItem::Array(a) | StackItem::Struct(a) => a.len(),
                    StackItem::Map(m) => m.len(),
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b.len(),
                    _ => return Err(VMError::InvalidType),
                };
                self.eval_stack.push(StackItem::Integer(size as i128));
            }
            // PICKITEM - Get item from array/map
            0xCE => {
                let key = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let container = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let item = match (container, key) {
                    (StackItem::Array(a), StackItem::Integer(i)) => a
                        .get(i as usize)
                        .cloned()
                        .ok_or(VMError::InvalidOperation)?,
                    (StackItem::Struct(s), StackItem::Integer(i)) => s
                        .get(i as usize)
                        .cloned()
                        .ok_or(VMError::InvalidOperation)?,
                    (StackItem::Map(m), k) => m
                        .iter()
                        .find(|(mk, _)| *mk == k)
                        .map(|(_, v)| v.clone())
                        .ok_or(VMError::InvalidOperation)?,
                    _ => return Err(VMError::InvalidType),
                };
                self.eval_stack.push(item);
            }
            // SETITEM - Set item in array/map
            0xD0 => {
                let value = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let key = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let container = self.eval_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                match (container, key) {
                    (StackItem::Array(a), StackItem::Integer(i)) => {
                        let idx = i as usize;
                        if idx >= a.len() {
                            return Err(VMError::InvalidOperation);
                        }
                        a[idx] = value;
                    }
                    (StackItem::Map(m), k) => {
                        if let Some(entry) = m.iter_mut().find(|(mk, _)| *mk == k) {
                            entry.1 = value;
                        } else {
                            m.push((k, value));
                        }
                    }
                    _ => return Err(VMError::InvalidType),
                }
            }
            // APPEND - Append to array
            0xCF => {
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let container = self.eval_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                match container {
                    StackItem::Array(a) => a.push(item),
                    _ => return Err(VMError::InvalidType),
                }
            }
            // REMOVE - Remove from array/map
            0xD2 => {
                let key = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let container = self.eval_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                match (container, key) {
                    (StackItem::Array(a), StackItem::Integer(i)) => {
                        let idx = i as usize;
                        if idx >= a.len() {
                            return Err(VMError::InvalidOperation);
                        }
                        a.remove(idx);
                    }
                    (StackItem::Map(m), k) => {
                        m.retain(|(mk, _)| *mk != k);
                    }
                    _ => return Err(VMError::InvalidType),
                }
            }
            // RET
            0x40 => {
                self.invocation_stack
                    .pop()
                    .ok_or(VMError::InvalidOperation)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_operations() {
        let mut vm = NeoVM::new(1_000_000);
        let _ = vm.load_script(vec![0x11, 0x12, 0x13, 0x40]);

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert!(matches!(vm.state, VMState::Halt));
        assert_eq!(vm.eval_stack.len(), 3);
    }

    #[test]
    fn test_add_operation() {
        let mut vm = NeoVM::new(1_000_000);
        let _ = vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]);

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
    }

    #[test]
    fn test_sub_operation() {
        let mut vm = NeoVM::new(1_000_000);
        let _ = vm.load_script(vec![0x15, 0x12, 0x9F, 0x40]);

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
    }

    #[test]
    fn test_mul_operation() {
        let mut vm = NeoVM::new(1_000_000);
        let _ = vm.load_script(vec![0x13, 0x14, 0xA0, 0x40]);

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(12)));
    }

    #[test]
    fn test_comparison_lt() {
        let mut vm = NeoVM::new(1_000_000);
        let _ = vm.load_script(vec![0x12, 0x15, 0xB5, 0x40]);

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
    }
}
