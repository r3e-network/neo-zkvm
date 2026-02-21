//! Neo VM Execution Engine
//!
//! Neo VM Engine
//!
//! Core execution engine for Neo zkVM.

use crate::{
    stack_item::StackItem,
    storage::{MemoryStorage, StorageBackend, StorageContext, StorageError},
};
use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Stack overflow: max depth {0} exceeded")]
    StackOverflow(usize),
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
    #[error("Storage is read-only")]
    StorageReadOnly,
    #[error("Invalid script")]
    InvalidScript,
    #[error("Invalid public key format for CHECKSIG")]
    InvalidPublicKey,
    #[error("Invalid signature format for CHECKSIG")]
    InvalidSignature,
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    #[error("Invocation depth exceeded: max {0}")]
    InvocationDepthExceeded(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VMState {
    None,
    Halt,
    Fault,
    Break,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub script: Arc<Vec<u8>>,
    pub ip: usize,
    pub local_slots: Vec<StackItem>,
    pub argument_slots: Vec<StackItem>,
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

/// Default maximum stack depth
pub const DEFAULT_MAX_STACK_DEPTH: usize = 2048;

/// Default maximum invocation depth
pub const DEFAULT_MAX_INVOCATION_DEPTH: usize = 1024;

/// Exception handling state within a TRY block
#[derive(Debug, Clone, PartialEq)]
pub enum ExceptionState {
    InTry,
    InCatch,
    InFinally,
}

/// Exception context pushed by TRY instructions
#[derive(Debug, Clone)]
pub struct ExceptionContext {
    pub catch_offset: Option<usize>,
    pub finally_offset: Option<usize>,
    pub state: ExceptionState,
    /// Pending exception item to re-throw in ENDFINALLY
    pub pending_exception: Option<StackItem>,
}

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
    pub max_stack_depth: usize,
    pub max_invocation_depth: usize,
    pub notifications: Vec<StackItem>,
    pub logs: Vec<String>,
    pub trace: ExecutionTrace,
    pub tracing_enabled: bool,
    pub storage: MemoryStorage,
    pub storage_context: StorageContext,
    // Static slot support for Neo VM compatibility.
    // Local/argument slots are scoped per invocation frame.
    pub static_slots: Vec<StackItem>,
    // Exception handling
    pub exception_stack: Vec<ExceptionContext>,
}

impl NeoVM {
    /// Default stack capacity for pre-allocation
    const DEFAULT_STACK_CAPACITY: usize = 64;
    /// Default invocation depth capacity
    const DEFAULT_INVOCATION_CAPACITY: usize = 8;

    /// Create a new VM with default limits
    #[inline]
    pub fn new(gas_limit: u64) -> Self {
        Self::with_limits(
            gas_limit,
            DEFAULT_MAX_STACK_DEPTH,
            DEFAULT_MAX_INVOCATION_DEPTH,
        )
    }

    /// Create a new VM with custom limits
    #[inline]
    pub fn with_limits(
        gas_limit: u64,
        max_stack_depth: usize,
        max_invocation_depth: usize,
    ) -> Self {
        Self {
            state: VMState::None,
            eval_stack: Vec::with_capacity(Self::DEFAULT_STACK_CAPACITY),
            invocation_stack: Vec::with_capacity(Self::DEFAULT_INVOCATION_CAPACITY),
            gas_consumed: 0,
            gas_limit,
            max_stack_depth,
            max_invocation_depth,
            notifications: Vec::new(),
            logs: Vec::new(),
            trace: ExecutionTrace::default(),
            tracing_enabled: false,
            storage: MemoryStorage::new(),
            storage_context: StorageContext::default(),
            static_slots: Vec::with_capacity(Self::DEFAULT_STACK_CAPACITY),
            exception_stack: Vec::new(),
        }
    }

    /// Configure current contract storage context used by storage syscalls.
    #[inline]
    pub fn set_storage_context(&mut self, script_hash: [u8; 20], read_only: bool) {
        self.storage_context.script_hash = script_hash;
        self.storage_context.read_only = read_only;
    }

    /// Update only the read-only flag for storage syscalls.
    #[inline]
    pub fn set_storage_read_only(&mut self, read_only: bool) {
        self.storage_context.read_only = read_only;
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
            if let Ok(bytes) = bincode::serialize(item) {
                hasher.update(&bytes);
            }
        }
        hasher.update(self.gas_consumed.to_le_bytes());
        hasher.finalize().into()
    }

    fn read_u8(ctx: &mut ExecutionContext) -> Result<u8, VMError> {
        if ctx.ip >= ctx.script.len() {
            return Err(VMError::InvalidScript);
        }
        let byte = ctx.script[ctx.ip];
        ctx.ip += 1;
        Ok(byte)
    }

    fn read_i8(ctx: &mut ExecutionContext) -> Result<i8, VMError> {
        Ok(Self::read_u8(ctx)? as i8)
    }

    fn read_u16_le(ctx: &mut ExecutionContext) -> Result<u16, VMError> {
        if ctx.ip + 1 >= ctx.script.len() {
            return Err(VMError::InvalidScript);
        }
        let val = u16::from_le_bytes([ctx.script[ctx.ip], ctx.script[ctx.ip + 1]]);
        ctx.ip += 2;
        Ok(val)
    }

    fn read_u32_le(ctx: &mut ExecutionContext) -> Result<u32, VMError> {
        if ctx.ip + 3 >= ctx.script.len() {
            return Err(VMError::InvalidScript);
        }
        let val = u32::from_le_bytes([
            ctx.script[ctx.ip],
            ctx.script[ctx.ip + 1],
            ctx.script[ctx.ip + 2],
            ctx.script[ctx.ip + 3],
        ]);
        ctx.ip += 4;
        Ok(val)
    }

    fn relative_target(base_ip: usize, offset: i8, script_len: usize) -> Result<usize, VMError> {
        let target = base_ip as isize + offset as isize;
        if target < 0 || target as usize >= script_len {
            return Err(VMError::InvalidScript);
        }
        Ok(target as usize)
    }

    fn relative_target_long(
        base_ip: usize,
        offset: i32,
        script_len: usize,
    ) -> Result<usize, VMError> {
        let target = base_ip as isize + offset as isize;
        if target < 0 || target as usize >= script_len {
            return Err(VMError::InvalidScript);
        }
        Ok(target as usize)
    }

    fn read_i32_le(ctx: &mut ExecutionContext) -> Result<i32, VMError> {
        Ok(Self::read_u32_le(ctx)? as i32)
    }

    /// Pop one integer from a stack without dropping on type mismatch.
    fn pop_integer_from_stack(eval_stack: &mut Vec<StackItem>) -> Result<i128, VMError> {
        match eval_stack.last() {
            Some(item) => match item.to_integer() {
                Some(value) => {
                    eval_stack.pop();
                    Ok(value)
                }
                None => Err(VMError::InvalidType),
            },
            None => Err(VMError::StackUnderflow),
        }
    }

    fn pop_usize_nonneg(&mut self) -> Result<usize, VMError> {
        let value = Self::pop_integer_from_stack(&mut self.eval_stack)?;
        if value < 0 || value > usize::MAX as i128 {
            Err(VMError::InvalidOperation)
        } else {
            Ok(value as usize)
        }
    }

    /// Pop one byte-sequence stack item without dropping on type mismatch.
    fn pop_bytes_from_stack(eval_stack: &mut Vec<StackItem>) -> Result<Vec<u8>, VMError> {
        match eval_stack.last() {
            Some(StackItem::ByteString(_)) | Some(StackItem::Buffer(_)) => {
                match eval_stack.pop().expect("stack last() checked above") {
                    StackItem::ByteString(bytes) | StackItem::Buffer(bytes) => Ok(bytes),
                    _ => unreachable!("type checked above"),
                }
            }
            Some(_) => Err(VMError::InvalidType),
            None => Err(VMError::StackUnderflow),
        }
    }

    #[inline]
    fn map_storage_error(err: StorageError) -> VMError {
        match err {
            StorageError::ReadOnly => VMError::StorageReadOnly,
        }
    }

    /// Push an item to the eval stack with depth checking
    #[inline]
    fn push(&mut self, item: StackItem) -> Result<(), VMError> {
        if self.eval_stack.len() >= self.max_stack_depth {
            return Err(VMError::StackOverflow(self.max_stack_depth));
        }
        self.eval_stack.push(item);
        Ok(())
    }

    /// Check if pushing to the invocation stack would exceed the limit
    #[inline]
    fn check_invocation_depth(&self) -> Result<(), VMError> {
        if self.invocation_stack.len() >= self.max_invocation_depth {
            return Err(VMError::InvocationDepthExceeded(self.max_invocation_depth));
        }
        Ok(())
    }

    #[inline]
    pub fn load_script(&mut self, script: Vec<u8>) -> Result<(), VMError> {
        if script.len() > MAX_SCRIPT_SIZE {
            return Err(VMError::InvalidScript);
        }
        self.check_invocation_depth()?;
        self.invocation_stack.push(ExecutionContext {
            script: Arc::new(script),
            ip: 0,
            local_slots: Vec::new(),
            argument_slots: Vec::new(),
        });
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
            0x10 => self.push(StackItem::Integer(0))?,
            0x11..=0x20 => {
                let n = (op - 0x10) as i128;
                self.push(StackItem::Integer(n))?;
            }
            0x0F => self.push(StackItem::Integer(-1))?,
            0x0B => self.push(StackItem::Null)?,
            // PUSHDATA1 - Push data with 1-byte length prefix
            0x0C => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let len = Self::read_u8(ctx)? as usize;
                if ctx.ip + len > ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let data = ctx.script[ctx.ip..ctx.ip + len].to_vec();
                ctx.ip += len;
                self.push(StackItem::ByteString(data))?;
            }
            // PUSHDATA2 - Push data with 2-byte length prefix
            0x0D => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let len = Self::read_u16_le(ctx)? as usize;
                if ctx.ip + len > ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let data = ctx.script[ctx.ip..ctx.ip + len].to_vec();
                ctx.ip += len;
                self.push(StackItem::ByteString(data))?;
            }
            // PUSHINT8
            0x00 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let val = Self::read_u8(ctx)? as i8 as i128;
                self.push(StackItem::Integer(val))?;
            }
            // PUSHINT16
            0x01 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let val = i16::from_le_bytes(Self::read_u16_le(ctx)?.to_le_bytes()) as i128;
                self.push(StackItem::Integer(val))?;
            }
            // PUSHINT32
            0x02 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let val = Self::read_i32_le(ctx)? as i128;
                self.push(StackItem::Integer(val))?;
            }
            // PUSHINT64
            0x03 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if ctx.ip + 7 >= ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&ctx.script[ctx.ip..ctx.ip + 8]);
                ctx.ip += 8;
                let val = i64::from_le_bytes(bytes) as i128;
                self.push(StackItem::Integer(val))?;
            }
            // PUSHINT128
            0x04 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if ctx.ip + 15 >= ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&ctx.script[ctx.ip..ctx.ip + 16]);
                ctx.ip += 16;
                let val = i128::from_le_bytes(bytes);
                self.push(StackItem::Integer(val))?;
            }
            // PUSHINT256
            0x05 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if ctx.ip + 31 >= ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let data = &ctx.script[ctx.ip..ctx.ip + 32];
                ctx.ip += 32;
                // Check that the upper 16 bytes are a valid sign extension of the lower 16
                let mut lo = [0u8; 16];
                lo.copy_from_slice(&data[..16]);
                let lo_val = i128::from_le_bytes(lo);
                let hi = &data[16..32];
                let sign_ext = if lo_val < 0 { 0xFFu8 } else { 0x00u8 };
                if !hi.iter().all(|&b| b == sign_ext) {
                    return Err(VMError::InvalidOperation);
                }
                self.push(StackItem::Integer(lo_val))?;
            }
            // PUSHDATA4 - Push data with 4-byte length prefix
            0x0E => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let len = Self::read_u32_le(ctx)? as usize;
                if ctx.ip + len > ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                let data = ctx.script[ctx.ip..ctx.ip + len].to_vec();
                ctx.ip += len;
                self.push(StackItem::ByteString(data))?;
            }
            // PUSHA - Push address (4-byte offset -> absolute pointer)
            0x0A => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let target = base_ip as isize + offset as isize;
                if target < 0 || target as usize >= ctx.script.len() {
                    return Err(VMError::InvalidScript);
                }
                self.push(StackItem::Pointer(target as u32))?;
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
                self.push(item)?;
            }
            // ADD
            0x9E => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let result = a.checked_add(b).ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // SUB
            0x9F => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let result = a.checked_sub(b).ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // MUL
            0xA0 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let result = a.checked_mul(b).ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // DIV
            0xA1 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                let result = a.checked_div(b).ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // MOD
            0xA2 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                let result = a.checked_rem(b).ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // POW
            0xA3 => {
                let exp = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let base = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if exp < 0 || exp > u32::MAX as i128 {
                    return Err(VMError::InvalidOperation);
                }
                let result = base
                    .checked_pow(exp as u32)
                    .ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // SHL
            0xA8 => {
                let shift = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let value = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if !(0..=256).contains(&shift) {
                    return Err(VMError::InvalidOperation);
                }
                let result = value
                    .checked_shl(shift as u32)
                    .ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // SHR
            0xA9 => {
                let shift = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let value = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if !(0..=256).contains(&shift) {
                    return Err(VMError::InvalidOperation);
                }
                let result = value
                    .checked_shr(shift as u32)
                    .ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // MIN
            0xB9 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Integer(a.min(b)))?;
            }
            // MAX
            0xBA => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Integer(a.max(b)))?;
            }
            // WITHIN (a <= x < b)
            0xBB => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let x = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Boolean(a <= x && x < b))?;
            }
            // SIGN
            0x99 => {
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let sign = if a > 0 {
                    1
                } else if a < 0 {
                    -1
                } else {
                    0
                };
                self.push(StackItem::Integer(sign))?;
            }
            // ABS
            0x9A => {
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let result = a.checked_abs().ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // NEGATE
            0x9B => {
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let result = a.checked_neg().ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // INC
            0x9C => {
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let result = a.checked_add(1).ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // DEC
            0x9D => {
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let result = a.checked_sub(1).ok_or(VMError::InvalidOperation)?;
                self.push(StackItem::Integer(result))?;
            }
            // LT
            0xB5 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Boolean(a < b))?;
            }
            // LE
            0xB6 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Boolean(a <= b))?;
            }
            // GT
            0xB7 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Boolean(a > b))?;
            }
            // GE
            0xB8 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Boolean(a >= b))?;
            }
            // EQUAL
            0x97 => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.push(StackItem::Boolean(a == b))?;
            }
            // NOTEQUAL
            0x98 => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.push(StackItem::Boolean(a != b))?;
            }
            // ISNULL
            0xD8 => {
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.push(StackItem::Boolean(matches!(item, StackItem::Null)))?;
            }
            // NZ - Not zero
            0xB1 => {
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Boolean(a != 0))?;
            }
            // NUMEQUAL
            0xB3 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Boolean(a == b))?;
            }
            // NUMNOTEQUAL
            0xB4 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Boolean(a != b))?;
            }
            // INVERT (bitwise NOT)
            0x90 => {
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Integer(!a))?;
            }
            // AND (bitwise)
            0x91 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Integer(a & b))?;
            }
            // OR (bitwise)
            0x92 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Integer(a | b))?;
            }
            // XOR (bitwise)
            0x93 => {
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                self.push(StackItem::Integer(a ^ b))?;
            }
            // NOT (logical)
            0xAA => {
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.push(StackItem::Boolean(!a.to_bool()))?;
            }
            // BOOLAND
            0xAB => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.push(StackItem::Boolean(a.to_bool() && b.to_bool()))?;
            }
            // BOOLOR
            0xAC => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.push(StackItem::Boolean(a.to_bool() || b.to_bool()))?;
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
                self.push(item)?;
            }
            // PICK
            0x4D => {
                let n = self.pop_usize_nonneg()?;
                let len = self.eval_stack.len();
                if n >= len {
                    return Err(VMError::StackUnderflow);
                }
                let item = self.eval_stack[len - 1 - n].clone();
                self.push(item)?;
            }
            // ROLL
            0x52 => {
                let n = self.pop_usize_nonneg()?;
                let len = self.eval_stack.len();
                if n >= len {
                    return Err(VMError::StackUnderflow);
                }
                let item = self.eval_stack.remove(len - 1 - n);
                self.push(item)?;
            }
            // OVER
            0x4B => {
                let len = self.eval_stack.len();
                if len < 2 {
                    return Err(VMError::StackUnderflow);
                }
                let item = self.eval_stack[len - 2].clone();
                self.push(item)?;
            }
            // DEPTH
            0x43 => {
                let depth = self.eval_stack.len() as i128;
                self.push(StackItem::Integer(depth))?;
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
                let n = self.pop_usize_nonneg()?;
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
                let n = self.pop_usize_nonneg()?;
                let len = self.eval_stack.len();
                if n > len {
                    return Err(VMError::StackUnderflow);
                }
                let start = len - n;
                self.eval_stack[start..].reverse();
            }
            // INITSLOT - Initialize local and argument slots
            0x57 => {
                let (local_count, arg_count) = {
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    (Self::read_u8(ctx)? as usize, Self::read_u8(ctx)? as usize)
                };
                // Pop arguments from stack into argument slots
                let mut argument_slots = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    let arg = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                    argument_slots.push(arg);
                }
                argument_slots.reverse();
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                ctx.local_slots = vec![StackItem::Null; local_count];
                ctx.argument_slots = argument_slots;
            }
            // LDLOC0-LDLOC6 - Load local variable 0-6
            0x66..=0x6C => {
                let idx = (op - 0x66) as usize;
                let item = self
                    .invocation_stack
                    .last()
                    .ok_or(VMError::StackUnderflow)?
                    .local_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.push(item)?;
            }
            // LDLOC_S - Load local variable (short form)
            0x6D => {
                let idx = {
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    Self::read_u8(ctx)? as usize
                };
                let item = self
                    .invocation_stack
                    .last()
                    .ok_or(VMError::StackUnderflow)?
                    .local_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.push(item)?;
            }
            // STLOC0-STLOC4 - Store local variable 0-4
            0x6E..=0x72 => {
                let val = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let idx = (op - 0x6E) as usize;
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if idx >= ctx.local_slots.len() {
                    return Err(VMError::InvalidOperation);
                }
                ctx.local_slots[idx] = val;
            }
            // STLOC_S - Store local variable (short form)
            0x73 => {
                let idx = {
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    Self::read_u8(ctx)? as usize
                };
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if idx >= ctx.local_slots.len() {
                    return Err(VMError::InvalidOperation);
                }
                ctx.local_slots[idx] = item;
            }
            // LDARG0-LDARG6 - Load argument 0-6
            0x74..=0x79 => {
                let idx = (op - 0x74) as usize;
                let item = self
                    .invocation_stack
                    .last()
                    .ok_or(VMError::StackUnderflow)?
                    .argument_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.push(item)?;
            }
            // LDARG - Load argument
            0x7A => {
                let idx = {
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    Self::read_u8(ctx)? as usize
                };
                let item = self
                    .invocation_stack
                    .last()
                    .ok_or(VMError::StackUnderflow)?
                    .argument_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.push(item)?;
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
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
            }
            // JMPIF (1-byte offset)
            0x24 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if cond.to_bool() {
                    ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPIFNOT (1-byte offset)
            0x26 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if !cond.to_bool() {
                    ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPEQ - Jump if equal
            0x28 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a == b {
                    ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPNE - Jump if not equal
            0x2A => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a != b {
                    ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPGT - Jump if greater than
            0x2C => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a > b {
                    ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPGE - Jump if greater or equal
            0x2E => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a >= b {
                    ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPLT - Jump if less than
            0x30 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a < b {
                    ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPLE - Jump if less or equal
            0x32 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a <= b {
                    ctx.ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMP_L (4-byte offset)
            0x23 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
            }
            // JMPIF_L (4-byte offset)
            0x25 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if cond.to_bool() {
                    ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPIFNOT_L (4-byte offset)
            0x27 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if !cond.to_bool() {
                    ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPEQ_L (4-byte offset)
            0x29 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a == b {
                    ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPNE_L (4-byte offset)
            0x2B => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a != b {
                    ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPGT_L (4-byte offset)
            0x2D => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a > b {
                    ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPGE_L (4-byte offset)
            0x2F => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a >= b {
                    ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPLT_L (4-byte offset)
            0x31 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a < b {
                    ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                }
            }
            // JMPLE_L (4-byte offset)
            0x33 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let b = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a <= b {
                    ctx.ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                }
            }
            // CALL (1-byte offset)
            0x34 => {
                self.check_invocation_depth()?;
                let (target_ip, script) = {
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                    let offset = Self::read_i8(ctx)?;
                    let target_ip = Self::relative_target(base_ip, offset, ctx.script.len())?;
                    let script = ctx.script.clone();
                    (target_ip, script)
                };
                self.invocation_stack.push(ExecutionContext {
                    script,
                    ip: target_ip,
                    local_slots: Vec::new(),
                    argument_slots: Vec::new(),
                });
            }
            // CALL_L (4-byte offset)
            0x35 => {
                self.check_invocation_depth()?;
                let (target_ip, script) = {
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                    let offset = Self::read_i32_le(ctx)?;
                    let target_ip = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                    let script = ctx.script.clone();
                    (target_ip, script)
                };
                self.invocation_stack.push(ExecutionContext {
                    script,
                    ip: target_ip,
                    local_slots: Vec::new(),
                    argument_slots: Vec::new(),
                });
            }
            // CALLA - Call absolute address from stack
            0x36 => {
                self.check_invocation_depth()?;
                let addr = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let target_ip = match addr {
                    StackItem::Pointer(p) => p as usize,
                    _ => return Err(VMError::InvalidType),
                };
                let script = {
                    let ctx = self
                        .invocation_stack
                        .last()
                        .ok_or(VMError::StackUnderflow)?;
                    if target_ip >= ctx.script.len() {
                        return Err(VMError::InvalidScript);
                    }
                    ctx.script.clone()
                };
                self.invocation_stack.push(ExecutionContext {
                    script,
                    ip: target_ip,
                    local_slots: Vec::new(),
                    argument_slots: Vec::new(),
                });
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
                self.push(StackItem::ByteString(result))?;
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
                self.push(StackItem::ByteString(result))?;
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
                self.push(StackItem::ByteString(result))?;
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
                self.push(StackItem::Boolean(verified))?;
            }
            // SYSCALL
            0x41 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let id = Self::read_u32_le(ctx)?;
                self.execute_syscall(id)?;
            }
            // NEWARRAY0 - Create empty array
            0xC2 => {
                self.push(StackItem::Array(Vec::new()))?;
            }
            // NEWARRAY - Create array with n elements
            0xC3 => {
                let n = self.pop_usize_nonneg()?;
                let arr = vec![StackItem::Null; n];
                self.push(StackItem::Array(arr))?;
            }
            // NEWSTRUCT0 - Create empty struct
            0xC5 => {
                self.push(StackItem::Struct(Vec::new()))?;
            }
            // NEWSTRUCT - Create struct with n elements
            0xC6 => {
                let n = self.pop_usize_nonneg()?;
                let s = vec![StackItem::Null; n];
                self.push(StackItem::Struct(s))?;
            }
            // NEWMAP - Create empty map
            0xC8 => {
                self.push(StackItem::Map(Vec::new()))?;
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
                self.push(StackItem::Integer(size as i128))?;
            }
            // PICKITEM - Get item from array/map
            0xCE => {
                let key = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let container = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let item = match (container, key) {
                    (StackItem::Array(a), StackItem::Integer(i)) => {
                        if i < 0 {
                            return Err(VMError::InvalidOperation);
                        }
                        a.get(i as usize)
                            .cloned()
                            .ok_or(VMError::InvalidOperation)?
                    }
                    (StackItem::Struct(s), StackItem::Integer(i)) => {
                        if i < 0 {
                            return Err(VMError::InvalidOperation);
                        }
                        s.get(i as usize)
                            .cloned()
                            .ok_or(VMError::InvalidOperation)?
                    }
                    (StackItem::Map(m), k) => m
                        .iter()
                        .find(|(mk, _)| *mk == k)
                        .map(|(_, v)| v.clone())
                        .ok_or(VMError::InvalidOperation)?,
                    _ => return Err(VMError::InvalidType),
                };
                self.push(item)?;
            }
            // SETITEM - Set item in array/map
            0xD0 => {
                let value = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let key = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let container = self.eval_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                match (container, key) {
                    (StackItem::Array(a), StackItem::Integer(i))
                    | (StackItem::Struct(a), StackItem::Integer(i)) => {
                        if i < 0 {
                            return Err(VMError::InvalidOperation);
                        }
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
                        if i < 0 {
                            return Err(VMError::InvalidOperation);
                        }
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
            // PACK - Pop n, then pop n items, create Array
            0xC0 => {
                let n = self.pop_usize_nonneg()?;
                let len = self.eval_stack.len();
                if n > len {
                    return Err(VMError::StackUnderflow);
                }
                let items = self.eval_stack.split_off(len - n);
                self.push(StackItem::Array(items))?;
            }
            // UNPACK - Pop Array, push all items then push count
            0xC1 => {
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let items = match item {
                    StackItem::Array(a) | StackItem::Struct(a) => a,
                    _ => return Err(VMError::InvalidType),
                };
                let count = items.len() as i128;
                for it in items {
                    self.push(it)?;
                }
                self.push(StackItem::Integer(count))?;
            }
            // NEWARRAY_T - Create typed array of n Null items (type byte consumed but ignored)
            0xC4 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let _type_byte = Self::read_u8(ctx)?;
                let n = self.pop_usize_nonneg()?;
                let arr = vec![StackItem::Null; n];
                self.push(StackItem::Array(arr))?;
            }
            // PACKMAP - Pop n, then pop n key-value pairs, create Map
            0xBE => {
                let n = self.pop_usize_nonneg()?;
                if self.eval_stack.len() < n * 2 {
                    return Err(VMError::StackUnderflow);
                }
                let mut pairs = Vec::with_capacity(n);
                for _ in 0..n {
                    let v = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                    let k = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                    pairs.push((k, v));
                }
                pairs.reverse();
                self.push(StackItem::Map(pairs))?;
            }
            // PACKSTRUCT - Pop n, then pop n items, create Struct
            0xBF => {
                let n = self.pop_usize_nonneg()?;
                let len = self.eval_stack.len();
                if n > len {
                    return Err(VMError::StackUnderflow);
                }
                let items = self.eval_stack.split_off(len - n);
                self.push(StackItem::Struct(items))?;
            }
            // HASKEY - Check if key exists in container
            0xCB => {
                let key = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let container = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let result = match (container, key) {
                    (StackItem::Array(a), StackItem::Integer(i))
                    | (StackItem::Struct(a), StackItem::Integer(i)) => {
                        i >= 0 && (i as usize) < a.len()
                    }
                    (StackItem::Map(m), k) => m.iter().any(|(mk, _)| *mk == k),
                    _ => return Err(VMError::InvalidType),
                };
                self.push(StackItem::Boolean(result))?;
            }
            // KEYS - Pop Map, push Array of keys
            0xCC => {
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                match item {
                    StackItem::Map(m) => {
                        let keys: Vec<StackItem> = m.into_iter().map(|(k, _)| k).collect();
                        self.push(StackItem::Array(keys))?;
                    }
                    _ => return Err(VMError::InvalidType),
                }
            }
            // VALUES - Pop Map or Array, push Array of values
            0xCD => {
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                match item {
                    StackItem::Map(m) => {
                        let vals: Vec<StackItem> = m.into_iter().map(|(_, v)| v).collect();
                        self.push(StackItem::Array(vals))?;
                    }
                    StackItem::Array(a) => {
                        self.push(StackItem::Array(a))?;
                    }
                    StackItem::Struct(s) => {
                        self.push(StackItem::Array(s))?;
                    }
                    _ => return Err(VMError::InvalidType),
                }
            }
            // REVERSEITEMS - Reverse Array/Struct in-place on stack
            0xD1 => {
                let container = self.eval_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                match container {
                    StackItem::Array(a) | StackItem::Struct(a) => a.reverse(),
                    _ => return Err(VMError::InvalidType),
                }
            }
            // CLEARITEMS - Clear all items from container on stack
            0xD3 => {
                let container = self.eval_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                match container {
                    StackItem::Array(a) | StackItem::Struct(a) => a.clear(),
                    StackItem::Map(m) => m.clear(),
                    _ => return Err(VMError::InvalidType),
                }
            }
            // POPITEM - Pop last item from Array on stack top
            0xD4 => {
                let container = self.eval_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                let item = match container {
                    StackItem::Array(a) => a.pop().ok_or(VMError::InvalidOperation)?,
                    _ => return Err(VMError::InvalidType),
                };
                self.push(item)?;
            }
            // ISTYPE - Check if item matches type byte
            0xD9 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let type_byte = Self::read_u8(ctx)?;
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let type_id = match &item {
                    StackItem::Null => 0x00,
                    StackItem::Boolean(_) => 0x01,
                    StackItem::Integer(_) => 0x21,
                    StackItem::ByteString(_) => 0x28,
                    StackItem::Buffer(_) => 0x30,
                    StackItem::Array(_) => 0x40,
                    StackItem::Struct(_) => 0x41,
                    StackItem::Map(_) => 0x48,
                    StackItem::Pointer(_) => 0x10,
                };
                self.push(StackItem::Boolean(type_id == type_byte))?;
            }
            // CONVERT - Convert item to target type
            0xDB => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let type_byte = Self::read_u8(ctx)?;
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let converted = match type_byte {
                    0x01 => StackItem::Boolean(item.to_bool()),
                    0x21 => match item {
                        StackItem::Integer(_) => item,
                        StackItem::Boolean(b) => StackItem::Integer(b as i128),
                        StackItem::ByteString(b) | StackItem::Buffer(b) => {
                            if b.len() > 16 {
                                return Err(VMError::InvalidOperation);
                            }
                            let mut bytes = [0u8; 16];
                            bytes[..b.len()].copy_from_slice(&b);
                            // Sign-extend if high bit set
                            if !b.is_empty() && b[b.len() - 1] & 0x80 != 0 {
                                bytes[b.len()..].fill(0xFF);
                            }
                            StackItem::Integer(i128::from_le_bytes(bytes))
                        }
                        _ => return Err(VMError::InvalidType),
                    },
                    0x28 => match item {
                        StackItem::ByteString(_) => item,
                        StackItem::Integer(i) => StackItem::ByteString(i.to_le_bytes().to_vec()),
                        StackItem::Boolean(b) => StackItem::ByteString(vec![b as u8]),
                        StackItem::Buffer(b) => StackItem::ByteString(b),
                        _ => return Err(VMError::InvalidType),
                    },
                    0x30 => match item {
                        StackItem::Buffer(_) => item,
                        StackItem::ByteString(b) => StackItem::Buffer(b),
                        _ => return Err(VMError::InvalidType),
                    },
                    _ => return Err(VMError::InvalidType),
                };
                self.push(converted)?;
            }
            // NEWBUFFER - Create zeroed buffer of given size
            0x88 => {
                let size = self.pop_usize_nonneg()?;
                self.push(StackItem::Buffer(vec![0u8; size]))?;
            }
            // MEMCPY - Copy bytes between buffers
            0x89 => {
                let count = self.pop_usize_nonneg()?;
                let src_index = self.pop_usize_nonneg()?;
                let src = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let dst_index = self.pop_usize_nonneg()?;
                let src_bytes = match &src {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    _ => return Err(VMError::InvalidType),
                };
                if src_index + count > src_bytes.len() {
                    return Err(VMError::InvalidOperation);
                }
                let copied = src_bytes[src_index..src_index + count].to_vec();
                let dst = self.eval_stack.last_mut().ok_or(VMError::StackUnderflow)?;
                match dst {
                    StackItem::Buffer(b) => {
                        if dst_index + count > b.len() {
                            return Err(VMError::InvalidOperation);
                        }
                        b[dst_index..dst_index + count].copy_from_slice(&copied);
                    }
                    _ => return Err(VMError::InvalidType),
                }
            }
            // CAT - Concatenate two ByteStrings
            0x8B => {
                let b = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let mut result = match a {
                    StackItem::ByteString(v) | StackItem::Buffer(v) => v,
                    _ => return Err(VMError::InvalidType),
                };
                let b_bytes = match b {
                    StackItem::ByteString(v) | StackItem::Buffer(v) => v,
                    _ => return Err(VMError::InvalidType),
                };
                result.extend_from_slice(&b_bytes);
                self.push(StackItem::ByteString(result))?;
            }
            // SUBSTR - Extract substring
            0x8C => {
                let count = self.pop_usize_nonneg()?;
                let index = self.pop_usize_nonneg()?;
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let bytes = match item {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    _ => return Err(VMError::InvalidType),
                };
                if index + count > bytes.len() {
                    return Err(VMError::InvalidOperation);
                }
                self.push(StackItem::ByteString(bytes[index..index + count].to_vec()))?;
            }
            // LEFT - Take left N bytes
            0x8D => {
                let count = self.pop_usize_nonneg()?;
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let bytes = match item {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    _ => return Err(VMError::InvalidType),
                };
                if count > bytes.len() {
                    return Err(VMError::InvalidOperation);
                }
                self.push(StackItem::ByteString(bytes[..count].to_vec()))?;
            }
            // RIGHT - Take right N bytes
            0x8E => {
                let count = self.pop_usize_nonneg()?;
                let item = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let bytes = match item {
                    StackItem::ByteString(b) | StackItem::Buffer(b) => b,
                    _ => return Err(VMError::InvalidType),
                };
                if count > bytes.len() {
                    return Err(VMError::InvalidOperation);
                }
                let start = bytes.len() - count;
                self.push(StackItem::ByteString(bytes[start..].to_vec()))?;
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
            // === Exception Handling Opcodes ===

            // ABORT - Immediately fault
            0x38 => {
                self.state = VMState::Fault;
                return Err(VMError::InvalidOperation);
            }
            // THROW - Pop exception item, trigger exception handling
            0x3A => {
                let exception = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.handle_throw(exception)?;
            }
            // TRY - Push exception context (1-byte catch offset, 1-byte finally offset)
            0x3B => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let catch_off = Self::read_i8(ctx)?;
                let finally_off = Self::read_i8(ctx)?;
                let script_len = ctx.script.len();
                let catch_addr = if catch_off != 0 {
                    Some(Self::relative_target(base_ip, catch_off, script_len)?)
                } else {
                    None
                };
                let finally_addr = if finally_off != 0 {
                    Some(Self::relative_target(base_ip, finally_off, script_len)?)
                } else {
                    None
                };
                if catch_addr.is_none() && finally_addr.is_none() {
                    return Err(VMError::InvalidScript);
                }
                self.exception_stack.push(ExceptionContext {
                    catch_offset: catch_addr,
                    finally_offset: finally_addr,
                    state: ExceptionState::InTry,
                    pending_exception: None,
                });
            }
            // TRY_L - Push exception context (4-byte catch offset, 4-byte finally offset)
            0x3C => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let catch_off = Self::read_i32_le(ctx)?;
                let finally_off = Self::read_i32_le(ctx)?;
                let script_len = ctx.script.len();
                let catch_addr = if catch_off != 0 {
                    Some(Self::relative_target_long(base_ip, catch_off, script_len)?)
                } else {
                    None
                };
                let finally_addr = if finally_off != 0 {
                    Some(Self::relative_target_long(
                        base_ip,
                        finally_off,
                        script_len,
                    )?)
                } else {
                    None
                };
                if catch_addr.is_none() && finally_addr.is_none() {
                    return Err(VMError::InvalidScript);
                }
                self.exception_stack.push(ExceptionContext {
                    catch_offset: catch_addr,
                    finally_offset: finally_addr,
                    state: ExceptionState::InTry,
                    pending_exception: None,
                });
            }
            // ENDTRY - End try block, jump to target (1-byte offset)
            0x3D => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i8(ctx)?;
                let target = Self::relative_target(base_ip, offset, ctx.script.len())?;
                self.end_try(target)?;
            }
            // ENDTRY_L - End try block, jump to target (4-byte offset)
            0x3E => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let base_ip = ctx.ip.checked_sub(1).ok_or(VMError::InvalidScript)?;
                let offset = Self::read_i32_le(ctx)?;
                let target = Self::relative_target_long(base_ip, offset, ctx.script.len())?;
                self.end_try(target)?;
            }
            // ENDFINALLY - End finally block
            0x3F => {
                let exc_ctx = self
                    .exception_stack
                    .pop()
                    .ok_or(VMError::InvalidOperation)?;
                if exc_ctx.state != ExceptionState::InFinally {
                    return Err(VMError::InvalidOperation);
                }
                if let Some(pending) = exc_ctx.pending_exception {
                    // Re-throw the pending exception
                    self.handle_throw(pending)?;
                }
                // Otherwise just continue execution
            }
            // ABORTMSG - Pop message, then abort
            0xE0 => {
                let _msg = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                self.state = VMState::Fault;
                return Err(VMError::InvalidOperation);
            }
            // ASSERTMSG - Pop message, then assert top of stack
            0xE1 => {
                let _msg = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let cond = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if !cond.to_bool() {
                    self.state = VMState::Fault;
                    return Err(VMError::InvalidOperation);
                }
            }

            // === Static Slot Opcodes ===

            // INITSSLOT - Initialize static slots with N Null items
            0x56 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let count = Self::read_u8(ctx)? as usize;
                if count == 0 {
                    return Err(VMError::InvalidOperation);
                }
                self.static_slots = vec![StackItem::Null; count];
            }
            // LDSFLD0-LDSFLD5 - Load static field 0-5
            0x58..=0x5D => {
                let idx = (op - 0x58) as usize;
                let item = self
                    .static_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.push(item)?;
            }
            // LDSFLD - Load static field (1-byte index)
            0x5E => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let idx = Self::read_u8(ctx)? as usize;
                let item = self
                    .static_slots
                    .get(idx)
                    .cloned()
                    .ok_or(VMError::InvalidOperation)?;
                self.push(item)?;
            }
            // STSFLD0-STSFLD5 - Store static field 0-5
            0x5F..=0x64 => {
                let val = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let idx = (op - 0x5F) as usize;
                if idx >= self.static_slots.len() {
                    return Err(VMError::InvalidOperation);
                }
                self.static_slots[idx] = val;
            }
            // STSFLD - Store static field (1-byte index)
            0x65 => {
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                let idx = Self::read_u8(ctx)? as usize;
                let val = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                if idx >= self.static_slots.len() {
                    return Err(VMError::InvalidOperation);
                }
                self.static_slots[idx] = val;
            }

            // === Argument Store Opcodes ===

            // STARG0-STARG6 - Store into argument slot 0-6
            0x7B..=0x80 => {
                let val = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let idx = (op - 0x7B) as usize;
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if idx >= ctx.argument_slots.len() {
                    return Err(VMError::InvalidOperation);
                }
                ctx.argument_slots[idx] = val;
            }
            // STARG - Store into argument slot (1-byte index)
            0x81 => {
                let idx = {
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    Self::read_u8(ctx)? as usize
                };
                let val = self.eval_stack.pop().ok_or(VMError::StackUnderflow)?;
                let ctx = self
                    .invocation_stack
                    .last_mut()
                    .ok_or(VMError::StackUnderflow)?;
                if idx >= ctx.argument_slots.len() {
                    return Err(VMError::InvalidOperation);
                }
                ctx.argument_slots[idx] = val;
            }

            // === Arithmetic Opcodes ===

            // SQRT - Integer square root
            0xA4 => {
                let a = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if a < 0 {
                    return Err(VMError::InvalidOperation);
                }
                let result = (a as f64).sqrt() as i128;
                self.push(StackItem::Integer(result))?;
            }
            // MODMUL - (x * y) % modulus
            0xA5 => {
                let modulus = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let y = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let x = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if modulus == 0 {
                    return Err(VMError::DivisionByZero);
                }
                // Use checked_mul with fallback to prevent overflow
                let result = match x.checked_mul(y) {
                    Some(product) => product
                        .checked_rem(modulus)
                        .ok_or(VMError::InvalidOperation)?,
                    None => {
                        // Overflow: reduce operands modulo modulus first, then multiply
                        let xm = x.checked_rem(modulus).ok_or(VMError::InvalidOperation)?;
                        let ym = y.checked_rem(modulus).ok_or(VMError::InvalidOperation)?;
                        let product = xm.checked_mul(ym).ok_or(VMError::InvalidOperation)?;
                        product
                            .checked_rem(modulus)
                            .ok_or(VMError::InvalidOperation)?
                    }
                };
                self.push(StackItem::Integer(result))?;
            }
            // MODPOW - Modular exponentiation: base^exp % modulus
            0xA6 => {
                let modulus = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let exp = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                let base = Self::pop_integer_from_stack(&mut self.eval_stack)?;
                if modulus == 0 {
                    return Err(VMError::DivisionByZero);
                }
                if exp < 0 {
                    return Err(VMError::InvalidOperation);
                }
                // Special case: modulus == 1 always yields 0
                if modulus == 1 || modulus == -1 {
                    self.push(StackItem::Integer(0))?;
                } else {
                    let result = Self::mod_pow(base, exp, modulus)?;
                    self.push(StackItem::Integer(result))?;
                }
            }

            _ => return Err(VMError::InvalidOpcode(op)),
        }
        Ok(())
    }

    /// Handle a thrown exception by jumping to catch/finally or faulting
    fn handle_throw(&mut self, exception: StackItem) -> Result<(), VMError> {
        while let Some(mut exc_ctx) = self.exception_stack.pop() {
            // If we have a catch block and we're in the try state, jump to catch
            if exc_ctx.state == ExceptionState::InTry {
                if let Some(catch_addr) = exc_ctx.catch_offset {
                    exc_ctx.state = ExceptionState::InCatch;
                    self.exception_stack.push(exc_ctx);
                    // Push exception item onto eval stack for the catch block
                    self.push(exception)?;
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    ctx.ip = catch_addr;
                    return Ok(());
                }
                // No catch block, try finally
                if let Some(finally_addr) = exc_ctx.finally_offset {
                    exc_ctx.state = ExceptionState::InFinally;
                    exc_ctx.pending_exception = Some(exception);
                    self.exception_stack.push(exc_ctx);
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    ctx.ip = finally_addr;
                    return Ok(());
                }
            }
            // If we're in catch state, go to finally if available
            if exc_ctx.state == ExceptionState::InCatch {
                if let Some(finally_addr) = exc_ctx.finally_offset {
                    exc_ctx.state = ExceptionState::InFinally;
                    exc_ctx.pending_exception = Some(exception);
                    self.exception_stack.push(exc_ctx);
                    let ctx = self
                        .invocation_stack
                        .last_mut()
                        .ok_or(VMError::StackUnderflow)?;
                    ctx.ip = finally_addr;
                    return Ok(());
                }
            }
            // Otherwise pop this context and try the next one
        }
        // No exception handler found, fault
        self.state = VMState::Fault;
        Err(VMError::InvalidOperation)
    }

    /// End a try or catch block, transitioning to finally or popping the context
    fn end_try(&mut self, end_target: usize) -> Result<(), VMError> {
        let exc_ctx = self
            .exception_stack
            .last_mut()
            .ok_or(VMError::InvalidOperation)?;
        if exc_ctx.state == ExceptionState::InFinally {
            return Err(VMError::InvalidOperation);
        }
        if let Some(finally_addr) = exc_ctx.finally_offset {
            exc_ctx.state = ExceptionState::InFinally;
            let ctx = self
                .invocation_stack
                .last_mut()
                .ok_or(VMError::StackUnderflow)?;
            ctx.ip = finally_addr;
        } else {
            // No finally block, just pop the exception context and jump to end
            self.exception_stack.pop();
            let ctx = self
                .invocation_stack
                .last_mut()
                .ok_or(VMError::StackUnderflow)?;
            ctx.ip = end_target;
        }
        Ok(())
    }

    /// Square-and-multiply modular exponentiation: base^exp % modulus
    fn mod_pow(mut base: i128, mut exp: i128, modulus: i128) -> Result<i128, VMError> {
        let mut result: i128 = 1;
        base = base.checked_rem(modulus).ok_or(VMError::InvalidOperation)?;
        if base < 0 {
            base = base
                .checked_add(modulus.abs())
                .ok_or(VMError::InvalidOperation)?;
        }
        while exp > 0 {
            if exp & 1 == 1 {
                result = Self::mod_mul_safe(result, base, modulus)?;
            }
            exp >>= 1;
            if exp > 0 {
                base = Self::mod_mul_safe(base, base, modulus)?;
            }
        }
        Ok(result)
    }

    /// Safe modular multiplication that handles potential i128 overflow
    fn mod_mul_safe(a: i128, b: i128, modulus: i128) -> Result<i128, VMError> {
        match a.checked_mul(b) {
            Some(product) => product
                .checked_rem(modulus)
                .ok_or(VMError::InvalidOperation),
            None => {
                // Fallback: reduce first, then multiply
                let am = a.checked_rem(modulus).ok_or(VMError::InvalidOperation)?;
                let bm = b.checked_rem(modulus).ok_or(VMError::InvalidOperation)?;
                let product = am.checked_mul(bm).ok_or(VMError::InvalidOperation)?;
                product
                    .checked_rem(modulus)
                    .ok_or(VMError::InvalidOperation)
            }
        }
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
                self.push(StackItem::Integer(0))?;
                Ok(())
            }
            syscall::SYSTEM_STORAGE_GET => {
                let key = Self::pop_bytes_from_stack(&mut self.eval_stack)?;
                match self.storage.get(&self.storage_context, &key) {
                    Some(value) => self.push(StackItem::ByteString(value))?,
                    None => self.push(StackItem::Null)?,
                }
                Ok(())
            }
            syscall::SYSTEM_STORAGE_PUT => {
                let value = Self::pop_bytes_from_stack(&mut self.eval_stack)?;
                let key = Self::pop_bytes_from_stack(&mut self.eval_stack)?;
                self.storage
                    .put(&self.storage_context, &key, &value)
                    .map_err(Self::map_storage_error)?;
                Ok(())
            }
            syscall::SYSTEM_STORAGE_DELETE => {
                let key = Self::pop_bytes_from_stack(&mut self.eval_stack)?;
                self.storage
                    .delete(&self.storage_context, &key)
                    .map_err(Self::map_storage_error)?;
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

    #[test]
    fn test_pow_negative_exponent_returns_error() {
        let mut vm = NeoVM::new(1_000_000);
        // PUSH2 (base=2), PUSHINT8 0xC8 (=-56 as i8, exp<0), POW, RET
        let _ = vm.load_script(vec![0x12, 0x00, 0xC8, 0xA3, 0x40]);
        vm.run();
        assert!(matches!(vm.state, VMState::Fault));
    }

    #[test]
    fn test_booland_stack_depth() {
        let mut vm = NeoVM::with_limits(1_000_000, 2, 1024);
        let _ = vm.load_script(vec![0x11, 0x11, 0xAB, 0x40]);
        vm.run();
        assert!(matches!(vm.state, VMState::Halt));
    }

    #[test]
    fn test_division_by_zero() {
        let mut vm = NeoVM::new(1_000_000);
        // PUSH5, PUSH0, DIV, RET
        let _ = vm.load_script(vec![0x15, 0x10, 0xA1, 0x40]);
        vm.run();
        assert!(matches!(vm.state, VMState::Fault));
    }

    #[test]
    fn test_stack_overflow() {
        let mut vm = NeoVM::with_limits(1_000_000, 3, 1024);
        // Push 4 items with depth limit 3
        let _ = vm.load_script(vec![0x11, 0x12, 0x13, 0x14, 0x40]);
        vm.run();
        assert!(matches!(vm.state, VMState::Fault));
    }

    #[test]
    fn test_sha256_opcode() {
        let mut vm = NeoVM::new(1_000_000);
        let mut script = vec![0x0C, 5];
        script.extend_from_slice(b"hello");
        script.push(0xF0);
        script.push(0x40);
        let _ = vm.load_script(script);
        vm.run();
        assert!(matches!(vm.state, VMState::Halt));
        assert_eq!(vm.eval_stack.len(), 1);
        if let Some(StackItem::ByteString(hash)) = vm.eval_stack.last() {
            assert_eq!(hash.len(), 32);
        } else {
            panic!("Expected ByteString");
        }
    }

    #[test]
    fn test_jmp_forward() {
        let mut vm = NeoVM::new(1_000_000);
        // [0]=PUSH1, [1]=JMP, [2]=offset(3), [3]=PUSH2(skipped), [4]=PUSH3, [5]=RET
        let _ = vm.load_script(vec![0x11, 0x22, 0x03, 0x12, 0x13, 0x40]);
        vm.run();
        assert!(matches!(vm.state, VMState::Halt));
        assert_eq!(vm.eval_stack.len(), 2);
        assert_eq!(vm.eval_stack[0], StackItem::Integer(1));
        assert_eq!(vm.eval_stack[1], StackItem::Integer(3));
    }

    #[test]
    fn test_newarray_and_pickitem() {
        let mut vm = NeoVM::new(1_000_000);
        // PUSH3, NEWARRAY, PUSH0, PICKITEM, RET
        let _ = vm.load_script(vec![0x13, 0xC3, 0x10, 0xCE, 0x40]);
        vm.run();
        assert!(matches!(vm.state, VMState::Halt));
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Null));
    }

    #[test]
    fn test_syscall_log() {
        let mut vm = NeoVM::new(1_000_000);
        let mut script = vec![0x0C, 4];
        script.extend_from_slice(b"test");
        script.extend_from_slice(&[0x41, 0x01, 0x00, 0x00, 0x00]);
        script.push(0x40);
        let _ = vm.load_script(script);
        vm.run();
        assert!(matches!(vm.state, VMState::Halt));
        assert_eq!(vm.logs, vec!["test".to_string()]);
    }
}
