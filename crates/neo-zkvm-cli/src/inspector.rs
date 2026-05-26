use std::collections::HashMap;

use neo_vm_rs::OpCode;

use crate::disassembler::Disassembler;

pub struct Inspector<'a> {
    script: &'a [u8],
}

impl<'a> Inspector<'a> {
    pub fn new(script: &'a [u8]) -> Self {
        Self { script }
    }

    pub fn analyze(&self) -> String {
        let mut output = String::new();

        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str("  SCRIPT ANALYSIS\n");
        output.push_str("═══════════════════════════════════════════════════════════════\n\n");

        output.push_str(&format!("  Size:         {} bytes\n", self.script.len()));
        output.push_str(&format!("  Hash (hex):   {}\n", hex::encode(self.script)));

        let stats = self.collect_opcode_stats();
        output.push_str("\n───────────────────────────────────────────────────────────────\n");
        output.push_str("  OPCODE STATISTICS\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");

        let mut sorted_stats: Vec<_> = stats.iter().collect();
        sorted_stats.sort_by(|a, b| b.1.cmp(a.1));

        for (name, count) in sorted_stats.iter().take(10) {
            output.push_str(&format!("    {:12} {:3}\n", name, count));
        }

        let jumps = self.find_jump_targets();
        if !jumps.is_empty() {
            output.push_str("\n───────────────────────────────────────────────────────────────\n");
            output.push_str("  JUMP TARGETS\n");
            output.push_str("───────────────────────────────────────────────────────────────\n");
            for target in &jumps {
                output.push_str(&format!("    0x{:04X}\n", target));
            }
        }

        let estimated_gas = self.estimate_gas();
        output.push_str("\n───────────────────────────────────────────────────────────────\n");
        output.push_str("  GAS ESTIMATION\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("    Minimum:    {}\n", estimated_gas.0));
        output.push_str(&format!("    Maximum:    {}\n", estimated_gas.1));

        output.push_str("\n───────────────────────────────────────────────────────────────\n");
        output.push_str("  DISASSEMBLY\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        let disasm = Disassembler::new(self.script);
        output.push_str(&disasm.disassemble());

        output.push_str("\n═══════════════════════════════════════════════════════════════\n");

        output
    }

    fn collect_opcode_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        let disasm = Disassembler::new(self.script);
        let mut ip = 0;

        while ip < self.script.len() {
            let (name, size) = disasm.decode_instruction(ip);
            let opcode_name = name.split_whitespace().next().unwrap_or(&name).to_string();
            *stats.entry(opcode_name).or_insert(0) += 1;
            ip += size;
        }

        stats
    }

    fn find_jump_targets(&self) -> Vec<usize> {
        let mut targets = Vec::new();
        let mut ip = 0;

        while ip < self.script.len() {
            let opcode_ip = ip;
            let raw_opcode = self.script[ip];
            let Ok(opcode) = OpCode::try_from(raw_opcode) else {
                ip += 1;
                continue;
            };
            ip += 1;

            match opcode {
                OpCode::JMP
                | OpCode::JMPIF
                | OpCode::JMPIFNOT
                | OpCode::JMPEQ
                | OpCode::JMPNE
                | OpCode::JMPGT
                | OpCode::JMPGE
                | OpCode::JMPLT
                | OpCode::JMPLE
                | OpCode::CALL => {
                    if ip < self.script.len() {
                        let offset = self.script[ip] as i8;
                        let target = (opcode_ip as isize + offset as isize) as usize;
                        if !targets.contains(&target) {
                            targets.push(target);
                        }
                    }
                    ip = self.advance_past_operands(opcode, ip);
                }
                OpCode::JMP_L
                | OpCode::JMPIF_L
                | OpCode::JMPIFNOT_L
                | OpCode::JMPEQ_L
                | OpCode::JMPNE_L
                | OpCode::JMPGT_L
                | OpCode::JMPGE_L
                | OpCode::JMPLT_L
                | OpCode::JMPLE_L
                | OpCode::CALL_L => {
                    if ip + 3 < self.script.len() {
                        let offset = i32::from_le_bytes([
                            self.script[ip],
                            self.script[ip + 1],
                            self.script[ip + 2],
                            self.script[ip + 3],
                        ]);
                        let target = (opcode_ip as isize + offset as isize) as usize;
                        if !targets.contains(&target) {
                            targets.push(target);
                        }
                    }
                    ip = self.advance_past_operands(opcode, ip);
                }
                _ => {
                    ip = self.advance_past_operands(opcode, ip);
                }
            }
        }

        targets.sort();
        targets
    }

    fn estimate_gas(&self) -> (u64, u64) {
        let mut min_gas = 0u64;
        let mut max_gas = 0u64;
        let mut ip = 0;

        while ip < self.script.len() {
            let op = self.script[ip];
            let cost = OpCode::try_from(op).map_or(1, estimated_opcode_cost);
            min_gas += cost;
            max_gas += cost;
            ip += 1;
            if let Ok(opcode) = OpCode::try_from(op) {
                ip = self.advance_past_operands(opcode, ip);
            }
        }

        // 10x safety factor: the static opcode-count estimate can't account
        // for dynamic costs (syscalls, storage, nested calls). Multiplying by
        // 10 gives headroom; callers should still monitor actual consumption.
        max_gas *= 10;

        (min_gas, max_gas)
    }

    fn advance_past_operands(&self, opcode: OpCode, ip: usize) -> usize {
        match opcode {
            OpCode::PUSHDATA1 if ip < self.script.len() => ip
                .saturating_add(1 + self.script[ip] as usize)
                .min(self.script.len()),
            OpCode::PUSHDATA2 if ip + 1 < self.script.len() => {
                let len = u16::from_le_bytes([self.script[ip], self.script[ip + 1]]) as usize;
                ip.saturating_add(2 + len).min(self.script.len())
            }
            OpCode::PUSHDATA4 if ip + 3 < self.script.len() => {
                let len = u32::from_le_bytes([
                    self.script[ip],
                    self.script[ip + 1],
                    self.script[ip + 2],
                    self.script[ip + 3],
                ]) as usize;
                ip.saturating_add(4)
                    .saturating_add(len)
                    .min(self.script.len())
            }
            _ => ip
                .saturating_add(opcode.operand_size())
                .min(self.script.len()),
        }
    }
}

pub fn estimated_opcode_cost(opcode: OpCode) -> u64 {
    match opcode {
        OpCode::PUSHINT8
        | OpCode::PUSHINT16
        | OpCode::PUSHINT32
        | OpCode::PUSHINT64
        | OpCode::PUSHINT128
        | OpCode::PUSHINT256
        | OpCode::PUSHA => 4,
        OpCode::PUSHNULL | OpCode::PUSHDATA1 | OpCode::PUSHDATA2 | OpCode::PUSHDATA4 => 8,
        OpCode::PUSHM1
        | OpCode::PUSH0
        | OpCode::PUSH1
        | OpCode::PUSH2
        | OpCode::PUSH3
        | OpCode::PUSH4
        | OpCode::PUSH5
        | OpCode::PUSH6
        | OpCode::PUSH7
        | OpCode::PUSH8
        | OpCode::PUSH9
        | OpCode::PUSH10
        | OpCode::PUSH11
        | OpCode::PUSH12
        | OpCode::PUSH13
        | OpCode::PUSH14
        | OpCode::PUSH15
        | OpCode::PUSH16 => 1,
        OpCode::NOP => 1,
        OpCode::JMP
        | OpCode::JMP_L
        | OpCode::JMPIF
        | OpCode::JMPIF_L
        | OpCode::JMPIFNOT
        | OpCode::JMPIFNOT_L
        | OpCode::JMPEQ
        | OpCode::JMPEQ_L
        | OpCode::JMPNE
        | OpCode::JMPNE_L
        | OpCode::JMPGT
        | OpCode::JMPGT_L
        | OpCode::JMPGE
        | OpCode::JMPGE_L
        | OpCode::JMPLT
        | OpCode::JMPLT_L
        | OpCode::JMPLE
        | OpCode::JMPLE_L => 2,
        OpCode::CALL | OpCode::CALL_L | OpCode::CALLA | OpCode::CALLT => 4,
        OpCode::ABORT | OpCode::ASSERT | OpCode::THROW => 1,
        OpCode::RET | OpCode::SYSCALL => 1,
        OpCode::DEPTH | OpCode::DROP | OpCode::NIP => 1,
        OpCode::XDROP | OpCode::CLEAR => 2,
        OpCode::DUP | OpCode::OVER | OpCode::PICK | OpCode::TUCK => 1,
        OpCode::SWAP | OpCode::ROT | OpCode::ROLL => 2,
        OpCode::REVERSE3 | OpCode::REVERSE4 | OpCode::REVERSEN => 4,
        OpCode::INITSSLOT | OpCode::INITSLOT => 4,
        OpCode::LDSFLD0
        | OpCode::LDSFLD1
        | OpCode::LDSFLD2
        | OpCode::LDSFLD3
        | OpCode::LDSFLD4
        | OpCode::LDSFLD5
        | OpCode::LDSFLD6 => 4,
        OpCode::STSFLD0
        | OpCode::STSFLD1
        | OpCode::STSFLD2
        | OpCode::STSFLD3
        | OpCode::STSFLD4
        | OpCode::STSFLD5
        | OpCode::STSFLD6 => 4,
        OpCode::LDLOC0
        | OpCode::LDLOC1
        | OpCode::LDLOC2
        | OpCode::LDLOC3
        | OpCode::LDLOC4
        | OpCode::LDLOC5
        | OpCode::LDLOC6 => 4,
        OpCode::STLOC0
        | OpCode::STLOC1
        | OpCode::STLOC2
        | OpCode::STLOC3
        | OpCode::STLOC4
        | OpCode::STLOC5
        | OpCode::STLOC6 => 4,
        OpCode::LDARG0
        | OpCode::LDARG1
        | OpCode::LDARG2
        | OpCode::LDARG3
        | OpCode::LDARG4
        | OpCode::LDARG5
        | OpCode::LDARG6 => 4,
        OpCode::STARG0
        | OpCode::STARG1
        | OpCode::STARG2
        | OpCode::STARG3
        | OpCode::STARG4
        | OpCode::STARG5
        | OpCode::STARG6 => 4,
        OpCode::NEWBUFFER | OpCode::MEMCPY | OpCode::CAT | OpCode::SUBSTR => 32,
        OpCode::LEFT | OpCode::RIGHT => 8,
        OpCode::INVERT | OpCode::AND | OpCode::OR | OpCode::XOR => 1,
        OpCode::EQUAL | OpCode::NOTEQUAL => 1,
        OpCode::SIGN | OpCode::ABS | OpCode::NEGATE | OpCode::INC | OpCode::DEC => 1,
        OpCode::ADD | OpCode::SUB | OpCode::MUL | OpCode::DIV | OpCode::MOD => 1,
        OpCode::SHL | OpCode::SHR => 1,
        OpCode::NOT | OpCode::BOOLAND | OpCode::BOOLOR => 1,
        OpCode::NZ => 1,
        OpCode::NUMEQUAL
        | OpCode::NUMNOTEQUAL
        | OpCode::LT
        | OpCode::LE
        | OpCode::GT
        | OpCode::GE => 1,
        OpCode::MIN | OpCode::MAX | OpCode::WITHIN => 4,
        OpCode::PACK | OpCode::UNPACK => 16,
        OpCode::NEWARRAY0 | OpCode::NEWARRAY | OpCode::NEWARRAY_T => 16,
        OpCode::NEWSTRUCT0 | OpCode::NEWSTRUCT => 16,
        OpCode::NEWMAP => 16,
        OpCode::SIZE => 1,
        OpCode::HASKEY => 16,
        OpCode::KEYS | OpCode::VALUES => 32,
        OpCode::PICKITEM => 16,
        OpCode::APPEND => 32,
        OpCode::SETITEM => 32,
        OpCode::REVERSEITEMS => 32,
        OpCode::REMOVE => 8,
        OpCode::CLEARITEMS => 4,
        OpCode::POPITEM => 1,
        OpCode::ISNULL | OpCode::ISTYPE => 1,
        OpCode::CONVERT => 32,
        _ => 2,
    }
}
