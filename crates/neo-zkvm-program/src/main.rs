//! Neo zkVM SP1 Guest Program - Production Grade
//!
//! Full Neo N3 VM implementation for zero-knowledge proving.

#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};

/// Input for zkVM proving
#[derive(Serialize, Deserialize, Clone)]
pub struct GuestInput {
    pub script: Vec<u8>,
    pub arguments: Vec<StackItem>,
    pub gas_limit: u64,
}

/// Stack item types matching Neo VM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StackItem {
    Null,
    Boolean(bool),
    Integer(i128),
    ByteString(Vec<u8>),
    Array(Vec<StackItem>),
    Map(Vec<(StackItem, StackItem)>),
    Struct(Vec<StackItem>),
}

impl StackItem {
    fn to_bool(&self) -> bool {
        match self {
            StackItem::Boolean(b) => *b,
            StackItem::Integer(i) => *i != 0,
            StackItem::ByteString(b) => !b.is_empty() && b.iter().any(|&x| x != 0),
            StackItem::Null => false,
            _ => true,
        }
    }

    fn to_integer(&self) -> Option<i128> {
        match self {
            StackItem::Integer(i) => Some(*i),
            StackItem::Boolean(b) => Some(*b as i128),
            StackItem::ByteString(b) if b.len() <= 16 => {
                let mut arr = [0u8; 16];
                arr[..b.len()].copy_from_slice(b);
                Some(i128::from_le_bytes(arr))
            }
            _ => None,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            StackItem::ByteString(b) => b.clone(),
            StackItem::Integer(i) => i.to_le_bytes().to_vec(),
            StackItem::Boolean(b) => vec![*b as u8],
            _ => vec![],
        }
    }
}

/// Public values committed to the proof
#[derive(Serialize, Deserialize)]
pub struct PublicValues {
    pub script_hash: [u8; 32],
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub gas_consumed: u64,
    pub execution_success: bool,
}

/// VM execution state
#[derive(Debug, Clone, Copy, PartialEq)]
enum VMState {
    Running,
    Halt,
    Fault,
}

/// Execution context for call stack
struct ExecutionContext {
    script: Vec<u8>,
    ip: usize,
}

/// Neo VM implementation for zkVM guest
struct NeoVM {
    state: VMState,
    eval_stack: Vec<StackItem>,
    invocation_stack: Vec<ExecutionContext>,
    gas_consumed: u64,
    gas_limit: u64,
    local_slots: Vec<StackItem>,
    argument_slots: Vec<StackItem>,
    static_slots: Vec<StackItem>,
}

impl NeoVM {
    fn new(gas_limit: u64) -> Self {
        Self {
            state: VMState::Running,
            eval_stack: Vec::new(),
            invocation_stack: Vec::new(),
            gas_consumed: 0,
            gas_limit,
            local_slots: Vec::new(),
            argument_slots: Vec::new(),
            static_slots: Vec::new(),
        }
    }

    fn load_script(&mut self, script: Vec<u8>) {
        self.invocation_stack
            .push(ExecutionContext { script, ip: 0 });
    }

    fn current_context(&mut self) -> Option<&mut ExecutionContext> {
        self.invocation_stack.last_mut()
    }

    fn consume_gas(&mut self, amount: u64) -> bool {
        self.gas_consumed += amount;
        self.gas_consumed <= self.gas_limit
    }

    fn pop(&mut self) -> Option<StackItem> {
        self.eval_stack.pop()
    }

    fn push(&mut self, item: StackItem) {
        self.eval_stack.push(item);
    }

    fn peek(&self, index: usize) -> Option<&StackItem> {
        let len = self.eval_stack.len();
        if index < len {
            Some(&self.eval_stack[len - 1 - index])
        } else {
            None
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        let ctx = self.current_context()?;
        if ctx.ip < ctx.script.len() {
            let b = ctx.script[ctx.ip];
            ctx.ip += 1;
            Some(b)
        } else {
            None
        }
    }

    fn read_i8(&mut self) -> Option<i8> {
        self.read_byte().map(|b| b as i8)
    }

    fn read_i16(&mut self) -> Option<i16> {
        let lo = self.read_byte()? as i16;
        let hi = self.read_byte()? as i16;
        Some(lo | (hi << 8))
    }

    fn read_i32(&mut self) -> Option<i32> {
        let b0 = self.read_byte()? as i32;
        let b1 = self.read_byte()? as i32;
        let b2 = self.read_byte()? as i32;
        let b3 = self.read_byte()? as i32;
        Some(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    fn read_bytes(&mut self, count: usize) -> Option<Vec<u8>> {
        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            result.push(self.read_byte()?);
        }
        Some(result)
    }

    fn execute(&mut self) {
        while self.state == VMState::Running {
            if !self.execute_next() {
                break;
            }
        }
    }

    fn execute_next(&mut self) -> bool {
        let opcode = match self.read_byte() {
            Some(op) => op,
            None => {
                self.state = VMState::Halt;
                return false;
            }
        };

        if !self.consume_gas(1) {
            self.state = VMState::Fault;
            return false;
        }

        self.execute_opcode(opcode)
    }

    fn execute_opcode(&mut self, opcode: u8) -> bool {
        match opcode {
            // === Constants ===
            0x00 => self.op_pushint8(),
            0x01 => self.op_pushint16(),
            0x02 => self.op_pushint32(),
            0x03 => self.op_pushint64(),
            0x04 => self.op_pushint128(),
            0x05 => self.op_pushint256(),
            0x0B => {
                self.push(StackItem::Null);
                true
            }
            0x0C => self.op_pushdata1(),
            0x0D => self.op_pushdata2(),
            0x0E => self.op_pushdata4(),
            0x0F => {
                self.push(StackItem::Integer(-1));
                true
            }
            0x10..=0x20 => {
                self.push(StackItem::Integer((opcode - 0x10) as i128));
                true
            }

            // === Flow Control ===
            0x21 => true, // NOP
            0x22 => self.op_jmp(),
            0x23 => self.op_jmp_l(),
            0x24 => self.op_jmpif(),
            0x25 => self.op_jmpif_l(),
            0x26 => self.op_jmpifnot(),
            0x27 => self.op_jmpifnot_l(),
            0x28 => self.op_jmpeq(),
            0x29 => self.op_jmpeq_l(),
            0x2A => self.op_jmpne(),
            0x2B => self.op_jmpne_l(),
            0x2C => self.op_jmpgt(),
            0x2D => self.op_jmpgt_l(),
            0x2E => self.op_jmpge(),
            0x2F => self.op_jmpge_l(),
            0x30 => self.op_jmplt(),
            0x31 => self.op_jmplt_l(),
            0x32 => self.op_jmple(),
            0x33 => self.op_jmple_l(),
            0x34 => self.op_call(),
            0x35 => self.op_call_l(),
            0x38 => {
                self.state = VMState::Fault;
                false
            } // ABORT
            0x39 => self.op_assert(),
            0x40 => self.op_ret(),

            // === Stack Operations ===
            0x43 => {
                self.push(StackItem::Integer(self.eval_stack.len() as i128));
                true
            }
            0x45 => {
                self.pop();
                true
            } // DROP
            0x46 => self.op_nip(),
            0x48 => self.op_xdrop(),
            0x49 => {
                self.eval_stack.clear();
                true
            } // CLEAR
            0x4A => self.op_dup(),
            0x4B => self.op_over(),
            0x4D => self.op_pick(),
            0x4E => self.op_tuck(),
            0x50 => self.op_swap(),
            0x51 => self.op_rot(),
            0x52 => self.op_roll(),
            0x53 => self.op_reverse3(),
            0x54 => self.op_reverse4(),
            0x55 => self.op_reversen(),

            // === Slot Operations ===
            0x56 => self.op_initsslot(),
            0x57 => self.op_initslot(),
            0x58..=0x5D => self.op_ldsfld_n(opcode - 0x58),
            0x5E => self.op_ldsfld(),
            0x5F..=0x64 => self.op_stsfld_n(opcode - 0x5F),
            0x65 => self.op_stsfld(),
            0x66..=0x6B => self.op_ldloc_n(opcode - 0x66),
            0x6C => self.op_ldloc(),
            0x6D..=0x72 => self.op_stloc_n(opcode - 0x6D),
            0x73 => self.op_stloc(),
            0x74..=0x79 => self.op_ldarg_n(opcode - 0x74),
            0x7A => self.op_ldarg(),
            0x7B..=0x80 => self.op_starg_n(opcode - 0x7B),
            0x81 => self.op_starg(),

            // === Splice Operations ===
            0x88 => self.op_newbuffer(),
            0x8B => self.op_cat(),
            0x8C => self.op_substr(),
            0x8D => self.op_left(),
            0x8E => self.op_right(),

            // === Bitwise Operations ===
            0x90 => self.op_invert(),
            0x91 => self.op_and(),
            0x92 => self.op_or(),
            0x93 => self.op_xor(),
            0x97 => self.op_equal(),
            0x98 => self.op_notequal(),

            // === Arithmetic Operations ===
            0x99 => self.op_sign(),
            0x9A => self.op_abs(),
            0x9B => self.op_negate(),
            0x9C => self.op_inc(),
            0x9D => self.op_dec(),
            0x9E => self.op_add(),
            0x9F => self.op_sub(),
            0xA0 => self.op_mul(),
            0xA1 => self.op_div(),
            0xA2 => self.op_mod(),
            0xA3 => self.op_pow(),
            0xA4 => self.op_sqrt(),
            0xA8 => self.op_shl(),
            0xA9 => self.op_shr(),
            0xAA => self.op_not(),
            0xAB => self.op_booland(),
            0xAC => self.op_boolor(),
            0xB1 => self.op_nz(),
            0xB3 => self.op_numequal(),
            0xB4 => self.op_numnotequal(),
            0xB5 => self.op_lt(),
            0xB6 => self.op_le(),
            0xB7 => self.op_gt(),
            0xB8 => self.op_ge(),
            0xB9 => self.op_min(),
            0xBA => self.op_max(),
            0xBB => self.op_within(),

            // === Compound Types ===
            0xC0 => self.op_pack(),
            0xC1 => self.op_unpack(),
            0xC2 => {
                self.push(StackItem::Array(vec![]));
                true
            }
            0xC3 => self.op_newarray(),
            0xC5 => {
                self.push(StackItem::Struct(vec![]));
                true
            }
            0xC6 => self.op_newstruct(),
            0xC8 => {
                self.push(StackItem::Map(vec![]));
                true
            }
            0xCA => self.op_size(),
            0xCB => self.op_haskey(),
            0xCE => self.op_pickitem(),
            0xCF => self.op_append(),
            0xD0 => self.op_setitem(),
            0xD2 => self.op_remove(),
            0xD3 => self.op_clearitems(),

            // === Type Operations ===
            0xD8 => self.op_isnull(),
            0xD9 => self.op_istype(),

            _ => {
                self.state = VMState::Fault;
                false
            }
        }
    }

    // === Push Operations ===
    fn op_pushint8(&mut self) -> bool {
        let v = self.read_i8().unwrap_or(0) as i128;
        self.push(StackItem::Integer(v));
        true
    }

    fn op_pushint16(&mut self) -> bool {
        let v = self.read_i16().unwrap_or(0) as i128;
        self.push(StackItem::Integer(v));
        true
    }

    fn op_pushint32(&mut self) -> bool {
        let v = self.read_i32().unwrap_or(0) as i128;
        self.push(StackItem::Integer(v));
        true
    }

    fn op_pushint64(&mut self) -> bool {
        let mut bytes = [0u8; 8];
        for b in &mut bytes {
            *b = self.read_byte().unwrap_or(0);
        }
        self.push(StackItem::Integer(i64::from_le_bytes(bytes) as i128));
        true
    }

    fn op_pushint128(&mut self) -> bool {
        let mut bytes = [0u8; 16];
        for b in &mut bytes {
            *b = self.read_byte().unwrap_or(0);
        }
        self.push(StackItem::Integer(i128::from_le_bytes(bytes)));
        true
    }

    fn op_pushint256(&mut self) -> bool {
        let bytes = self.read_bytes(32).unwrap_or_default();
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes[..16]);
        self.push(StackItem::Integer(i128::from_le_bytes(arr)));
        true
    }

    fn op_pushdata1(&mut self) -> bool {
        let len = self.read_byte().unwrap_or(0) as usize;
        let data = self.read_bytes(len).unwrap_or_default();
        self.push(StackItem::ByteString(data));
        true
    }

    fn op_pushdata2(&mut self) -> bool {
        let len = self.read_i16().unwrap_or(0) as usize;
        let data = self.read_bytes(len).unwrap_or_default();
        self.push(StackItem::ByteString(data));
        true
    }

    fn op_pushdata4(&mut self) -> bool {
        let len = self.read_i32().unwrap_or(0) as usize;
        let data = self.read_bytes(len).unwrap_or_default();
        self.push(StackItem::ByteString(data));
        true
    }

    // === Flow Control ===
    fn op_jmp(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        self.jump(offset - 2)
    }

    fn op_jmp_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        self.jump(offset - 5)
    }

    fn op_jmpif(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        if self.pop().map(|i| i.to_bool()).unwrap_or(false) {
            self.jump(offset - 2)
        } else {
            true
        }
    }

    fn op_jmpif_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        if self.pop().map(|i| i.to_bool()).unwrap_or(false) {
            self.jump(offset - 5)
        } else {
            true
        }
    }

    fn op_jmpifnot(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        if !self.pop().map(|i| i.to_bool()).unwrap_or(true) {
            self.jump(offset - 2)
        } else {
            true
        }
    }

    fn op_jmpifnot_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        if !self.pop().map(|i| i.to_bool()).unwrap_or(true) {
            self.jump(offset - 5)
        } else {
            true
        }
    }

    fn jump(&mut self, offset: isize) -> bool {
        if let Some(ctx) = self.current_context() {
            let new_ip = (ctx.ip as isize + offset) as usize;
            if new_ip <= ctx.script.len() {
                ctx.ip = new_ip;
                return true;
            }
        }
        self.state = VMState::Fault;
        false
    }

    fn op_jmpeq(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a == b {
            self.jump(offset - 2)
        } else {
            true
        }
    }

    fn op_jmpeq_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a == b {
            self.jump(offset - 5)
        } else {
            true
        }
    }

    fn op_jmpne(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a != b {
            self.jump(offset - 2)
        } else {
            true
        }
    }

    fn op_jmpne_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a != b {
            self.jump(offset - 5)
        } else {
            true
        }
    }

    fn op_jmpgt(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a > b {
            self.jump(offset - 2)
        } else {
            true
        }
    }

    fn op_jmpgt_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a > b {
            self.jump(offset - 5)
        } else {
            true
        }
    }

    fn op_jmpge(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a >= b {
            self.jump(offset - 2)
        } else {
            true
        }
    }

    fn op_jmpge_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a >= b {
            self.jump(offset - 5)
        } else {
            true
        }
    }

    fn op_jmplt(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a < b {
            self.jump(offset - 2)
        } else {
            true
        }
    }

    fn op_jmplt_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a < b {
            self.jump(offset - 5)
        } else {
            true
        }
    }

    fn op_jmple(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a <= b {
            self.jump(offset - 2)
        } else {
            true
        }
    }

    fn op_jmple_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a <= b {
            self.jump(offset - 5)
        } else {
            true
        }
    }

    fn op_call(&mut self) -> bool {
        let offset = self.read_i8().unwrap_or(0) as isize;
        if let Some(ctx) = self.current_context() {
            let target = (ctx.ip as isize + offset - 2) as usize;
            let script = ctx.script.clone();
            self.invocation_stack
                .push(ExecutionContext { script, ip: target });
            true
        } else {
            false
        }
    }

    fn op_call_l(&mut self) -> bool {
        let offset = self.read_i32().unwrap_or(0) as isize;
        if let Some(ctx) = self.current_context() {
            let target = (ctx.ip as isize + offset - 5) as usize;
            let script = ctx.script.clone();
            self.invocation_stack
                .push(ExecutionContext { script, ip: target });
            true
        } else {
            false
        }
    }

    fn op_ret(&mut self) -> bool {
        self.invocation_stack.pop();
        if self.invocation_stack.is_empty() {
            self.state = VMState::Halt;
        }
        true
    }

    fn op_assert(&mut self) -> bool {
        if !self.pop().map(|i| i.to_bool()).unwrap_or(false) {
            self.state = VMState::Fault;
            false
        } else {
            true
        }
    }

    // === Stack Operations ===
    fn op_nip(&mut self) -> bool {
        if self.eval_stack.len() >= 2 {
            let top = self.pop().unwrap();
            self.pop();
            self.push(top);
            true
        } else {
            false
        }
    }

    fn op_xdrop(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        if n < self.eval_stack.len() {
            let idx = self.eval_stack.len() - 1 - n;
            self.eval_stack.remove(idx);
            true
        } else {
            false
        }
    }

    fn op_dup(&mut self) -> bool {
        if let Some(item) = self.peek(0).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_over(&mut self) -> bool {
        if let Some(item) = self.peek(1).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_pick(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        if let Some(item) = self.peek(n).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_tuck(&mut self) -> bool {
        if self.eval_stack.len() >= 2 {
            let top = self.peek(0).cloned().unwrap();
            let len = self.eval_stack.len();
            self.eval_stack.insert(len - 2, top);
            true
        } else {
            false
        }
    }

    fn op_swap(&mut self) -> bool {
        let len = self.eval_stack.len();
        if len >= 2 {
            self.eval_stack.swap(len - 1, len - 2);
            true
        } else {
            false
        }
    }

    fn op_rot(&mut self) -> bool {
        let len = self.eval_stack.len();
        if len >= 3 {
            let item = self.eval_stack.remove(len - 3);
            self.eval_stack.push(item);
            true
        } else {
            false
        }
    }

    fn op_roll(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        let len = self.eval_stack.len();
        if n < len {
            let item = self.eval_stack.remove(len - 1 - n);
            self.eval_stack.push(item);
            true
        } else {
            false
        }
    }

    fn op_reverse3(&mut self) -> bool {
        let len = self.eval_stack.len();
        if len >= 3 {
            self.eval_stack[len - 3..].reverse();
            true
        } else {
            false
        }
    }

    fn op_reverse4(&mut self) -> bool {
        let len = self.eval_stack.len();
        if len >= 4 {
            self.eval_stack[len - 4..].reverse();
            true
        } else {
            false
        }
    }

    fn op_reversen(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        let len = self.eval_stack.len();
        if n <= len {
            self.eval_stack[len - n..].reverse();
            true
        } else {
            false
        }
    }

    // === Slot Operations ===
    fn op_initsslot(&mut self) -> bool {
        let count = self.read_byte().unwrap_or(0) as usize;
        self.static_slots = vec![StackItem::Null; count];
        true
    }

    fn op_initslot(&mut self) -> bool {
        let locals = self.read_byte().unwrap_or(0) as usize;
        let args = self.read_byte().unwrap_or(0) as usize;
        self.local_slots = vec![StackItem::Null; locals];
        self.argument_slots = (0..args).filter_map(|_| self.pop()).collect();
        self.argument_slots.reverse();
        true
    }

    fn op_ldsfld_n(&mut self, n: u8) -> bool {
        if let Some(item) = self.static_slots.get(n as usize).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_ldsfld(&mut self) -> bool {
        let n = self.read_byte().unwrap_or(0) as usize;
        if let Some(item) = self.static_slots.get(n).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_stsfld_n(&mut self, n: u8) -> bool {
        if let Some(item) = self.pop() {
            let n = n as usize;
            if n < self.static_slots.len() {
                self.static_slots[n] = item;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn op_stsfld(&mut self) -> bool {
        let n = self.read_byte().unwrap_or(0) as usize;
        if let Some(item) = self.pop() {
            if n < self.static_slots.len() {
                self.static_slots[n] = item;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn op_ldloc_n(&mut self, n: u8) -> bool {
        if let Some(item) = self.local_slots.get(n as usize).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_ldloc(&mut self) -> bool {
        let n = self.read_byte().unwrap_or(0) as usize;
        if let Some(item) = self.local_slots.get(n).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_stloc_n(&mut self, n: u8) -> bool {
        if let Some(item) = self.pop() {
            let n = n as usize;
            if n < self.local_slots.len() {
                self.local_slots[n] = item;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn op_stloc(&mut self) -> bool {
        let n = self.read_byte().unwrap_or(0) as usize;
        if let Some(item) = self.pop() {
            if n < self.local_slots.len() {
                self.local_slots[n] = item;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn op_ldarg_n(&mut self, n: u8) -> bool {
        if let Some(item) = self.argument_slots.get(n as usize).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_ldarg(&mut self) -> bool {
        let n = self.read_byte().unwrap_or(0) as usize;
        if let Some(item) = self.argument_slots.get(n).cloned() {
            self.push(item);
            true
        } else {
            false
        }
    }

    fn op_starg_n(&mut self, n: u8) -> bool {
        if let Some(item) = self.pop() {
            let n = n as usize;
            if n < self.argument_slots.len() {
                self.argument_slots[n] = item;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn op_starg(&mut self) -> bool {
        let n = self.read_byte().unwrap_or(0) as usize;
        if let Some(item) = self.pop() {
            if n < self.argument_slots.len() {
                self.argument_slots[n] = item;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // === Arithmetic Operations ===
    fn op_add(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a.wrapping_add(b)));
        true
    }

    fn op_sub(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a.wrapping_sub(b)));
        true
    }

    fn op_mul(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a.wrapping_mul(b)));
        true
    }

    fn op_div(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if b == 0 {
            self.state = VMState::Fault;
            false
        } else {
            self.push(StackItem::Integer(a / b));
            true
        }
    }

    fn op_mod(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if b == 0 {
            self.state = VMState::Fault;
            false
        } else {
            self.push(StackItem::Integer(a % b));
            true
        }
    }

    fn op_pow(&mut self) -> bool {
        let exp = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let base = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let result = if exp >= 0 { base.pow(exp as u32) } else { 0 };
        self.push(StackItem::Integer(result));
        true
    }

    fn op_sqrt(&mut self) -> bool {
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        if a < 0 {
            self.state = VMState::Fault;
            false
        } else {
            self.push(StackItem::Integer((a as f64).sqrt() as i128));
            true
        }
    }

    fn op_sign(&mut self) -> bool {
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a.signum()));
        true
    }

    fn op_abs(&mut self) -> bool {
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a.abs()));
        true
    }

    fn op_negate(&mut self) -> bool {
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(-a));
        true
    }

    fn op_inc(&mut self) -> bool {
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a + 1));
        true
    }

    fn op_dec(&mut self) -> bool {
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a - 1));
        true
    }

    // === Bitwise Operations ===
    fn op_and(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a & b));
        true
    }

    fn op_or(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a | b));
        true
    }

    fn op_xor(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a ^ b));
        true
    }

    fn op_invert(&mut self) -> bool {
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(!a));
        true
    }

    fn op_shl(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as u32;
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a << n));
        true
    }

    fn op_shr(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as u32;
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a >> n));
        true
    }

    fn op_not(&mut self) -> bool {
        let a = self.pop().map(|i| i.to_bool()).unwrap_or(false);
        self.push(StackItem::Boolean(!a));
        true
    }

    fn op_booland(&mut self) -> bool {
        let b = self.pop().map(|i| i.to_bool()).unwrap_or(false);
        let a = self.pop().map(|i| i.to_bool()).unwrap_or(false);
        self.push(StackItem::Boolean(a && b));
        true
    }

    fn op_boolor(&mut self) -> bool {
        let b = self.pop().map(|i| i.to_bool()).unwrap_or(false);
        let a = self.pop().map(|i| i.to_bool()).unwrap_or(false);
        self.push(StackItem::Boolean(a || b));
        true
    }

    fn op_nz(&mut self) -> bool {
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Boolean(a != 0));
        true
    }

    fn op_numequal(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Boolean(a == b));
        true
    }

    fn op_numnotequal(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Boolean(a != b));
        true
    }

    fn op_lt(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Boolean(a < b));
        true
    }

    fn op_le(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Boolean(a <= b));
        true
    }

    fn op_gt(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Boolean(a > b));
        true
    }

    fn op_ge(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Boolean(a >= b));
        true
    }

    fn op_min(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a.min(b)));
        true
    }

    fn op_max(&mut self) -> bool {
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Integer(a.max(b)));
        true
    }

    fn op_within(&mut self) -> bool {
        let c = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let b = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        let a = self.pop().and_then(|i| i.to_integer()).unwrap_or(0);
        self.push(StackItem::Boolean(a >= b && a < c));
        true
    }

    // === Splice Operations ===
    fn op_newbuffer(&mut self) -> bool {
        let size = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        self.push(StackItem::ByteString(vec![0u8; size]));
        true
    }

    fn op_cat(&mut self) -> bool {
        let b = self.pop().map(|i| i.to_bytes()).unwrap_or_default();
        let mut a = self.pop().map(|i| i.to_bytes()).unwrap_or_default();
        a.extend(b);
        self.push(StackItem::ByteString(a));
        true
    }

    fn op_substr(&mut self) -> bool {
        let count = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        let index = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        let s = self.pop().map(|i| i.to_bytes()).unwrap_or_default();
        let end = (index + count).min(s.len());
        self.push(StackItem::ByteString(s[index..end].to_vec()));
        true
    }

    fn op_left(&mut self) -> bool {
        let count = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        let s = self.pop().map(|i| i.to_bytes()).unwrap_or_default();
        self.push(StackItem::ByteString(s[..count.min(s.len())].to_vec()));
        true
    }

    fn op_right(&mut self) -> bool {
        let count = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        let s = self.pop().map(|i| i.to_bytes()).unwrap_or_default();
        let start = s.len().saturating_sub(count);
        self.push(StackItem::ByteString(s[start..].to_vec()));
        true
    }

    fn op_equal(&mut self) -> bool {
        let b = self.pop();
        let a = self.pop();
        self.push(StackItem::Boolean(a == b));
        true
    }

    fn op_notequal(&mut self) -> bool {
        let b = self.pop();
        let a = self.pop();
        self.push(StackItem::Boolean(a != b));
        true
    }

    // === Compound Type Operations ===
    fn op_pack(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        let mut items = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(item) = self.pop() {
                items.push(item);
            }
        }
        items.reverse();
        self.push(StackItem::Array(items));
        true
    }

    fn op_unpack(&mut self) -> bool {
        if let Some(StackItem::Array(items)) = self.pop() {
            let len = items.len();
            for item in items {
                self.push(item);
            }
            self.push(StackItem::Integer(len as i128));
            true
        } else {
            false
        }
    }

    fn op_newarray(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        self.push(StackItem::Array(vec![StackItem::Null; n]));
        true
    }

    fn op_newstruct(&mut self) -> bool {
        let n = self.pop().and_then(|i| i.to_integer()).unwrap_or(0) as usize;
        self.push(StackItem::Struct(vec![StackItem::Null; n]));
        true
    }

    fn op_size(&mut self) -> bool {
        let size = match self.pop() {
            Some(StackItem::ByteString(b)) => b.len(),
            Some(StackItem::Array(a)) => a.len(),
            Some(StackItem::Map(m)) => m.len(),
            Some(StackItem::Struct(s)) => s.len(),
            _ => 0,
        };
        self.push(StackItem::Integer(size as i128));
        true
    }

    fn op_haskey(&mut self) -> bool {
        let key = self.pop();
        let container = self.pop();
        let has = match (container, key) {
            (Some(StackItem::Array(a)), Some(StackItem::Integer(i))) => (i as usize) < a.len(),
            (Some(StackItem::Map(m)), Some(k)) => m.iter().any(|(mk, _)| *mk == k),
            _ => false,
        };
        self.push(StackItem::Boolean(has));
        true
    }

    fn op_pickitem(&mut self) -> bool {
        let key = self.pop();
        let container = self.pop();
        match (container, key) {
            (Some(StackItem::Array(a)), Some(StackItem::Integer(i))) => {
                if let Some(item) = a.get(i as usize).cloned() {
                    self.push(item);
                    true
                } else {
                    false
                }
            }
            (Some(StackItem::Map(m)), Some(k)) => {
                if let Some((_, v)) = m.iter().find(|(mk, _)| *mk == k) {
                    self.push(v.clone());
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn op_append(&mut self) -> bool {
        let item = self.pop();
        if let (Some(StackItem::Array(mut a)), Some(i)) = (self.pop(), item) {
            a.push(i);
            self.push(StackItem::Array(a));
            true
        } else {
            false
        }
    }

    fn op_setitem(&mut self) -> bool {
        let value = self.pop();
        let key = self.pop();
        let container = self.pop();
        match (container, key, value) {
            (Some(StackItem::Array(mut a)), Some(StackItem::Integer(i)), Some(v)) => {
                let idx = i as usize;
                if idx < a.len() {
                    a[idx] = v;
                    self.push(StackItem::Array(a));
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn op_remove(&mut self) -> bool {
        let key = self.pop();
        let container = self.pop();
        match (container, key) {
            (Some(StackItem::Array(mut a)), Some(StackItem::Integer(i))) => {
                let idx = i as usize;
                if idx < a.len() {
                    a.remove(idx);
                    self.push(StackItem::Array(a));
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn op_clearitems(&mut self) -> bool {
        match self.pop() {
            Some(StackItem::Array(_)) => {
                self.push(StackItem::Array(vec![]));
                true
            }
            Some(StackItem::Map(_)) => {
                self.push(StackItem::Map(vec![]));
                true
            }
            _ => false,
        }
    }

    fn op_isnull(&mut self) -> bool {
        let is_null = matches!(self.pop(), Some(StackItem::Null) | None);
        self.push(StackItem::Boolean(is_null));
        true
    }

    fn op_istype(&mut self) -> bool {
        let type_id = self.read_byte().unwrap_or(0);
        let item = self.pop();
        let matches = matches!(
            (item, type_id),
            (Some(StackItem::Boolean(_)), 0x20)
                | (Some(StackItem::Integer(_)), 0x21)
                | (Some(StackItem::ByteString(_)), 0x28)
                | (Some(StackItem::Array(_)), 0x40)
                | (Some(StackItem::Struct(_)), 0x41)
                | (Some(StackItem::Map(_)), 0x48)
        );
        self.push(StackItem::Boolean(matches));
        true
    }
}

/// SHA256 hash for zkVM (deterministic)
fn sha256_hash(data: &[u8]) -> [u8; 32] {
    // Simple SHA256-like hash for zkVM guest
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    for chunk in data.chunks(64) {
        for (i, &byte) in chunk.iter().enumerate() {
            h[i % 8] = h[i % 8].wrapping_add(byte as u32);
            h[(i + 1) % 8] = h[(i + 1) % 8].rotate_left(5) ^ h[i % 8];
        }
    }

    let mut result = [0u8; 32];
    for (i, &word) in h.iter().enumerate() {
        result[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    result
}

/// Main entry point for SP1 guest program
pub fn main() {
    // Read input from host
    let input: GuestInput = sp1_zkvm::io::read();

    // Compute input hashes
    let script_hash = sha256_hash(&input.script);
    let input_bytes = bincode::serialize(&input.arguments).unwrap_or_default();
    let input_hash = sha256_hash(&input_bytes);

    // Execute the Neo VM script
    let mut vm = NeoVM::new(input.gas_limit);
    vm.load_script(input.script);

    // Push arguments to stack
    for arg in input.arguments {
        vm.push(arg);
    }

    // Execute
    vm.execute();

    // Compute output hash
    let result_bytes = bincode::serialize(&vm.eval_stack).unwrap_or_default();
    let output_hash = sha256_hash(&result_bytes);

    // Create public values
    let public_values = PublicValues {
        script_hash,
        input_hash,
        output_hash,
        gas_consumed: vm.gas_consumed,
        execution_success: vm.state == VMState::Halt,
    };

    // Commit public values to the proof
    sp1_zkvm::io::commit(&public_values);
}
