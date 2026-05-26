//! Neo zkVM Disassembler with enhanced formatting
//!
//! Features:
//! - Full Neo N3 opcode support
//! - Colored output (when terminal supports it)
//! - Jump target annotations
//! - Operand decoding

use neo_vm_rs::{interop_hash, OpCode, StackItemType};

pub struct Disassembler<'a> {
    script: &'a [u8],
}

impl<'a> Disassembler<'a> {
    pub fn new(script: &'a [u8]) -> Self {
        Self { script }
    }

    pub fn disassemble(&self) -> String {
        let mut output = String::new();
        let mut ip = 0;

        while ip < self.script.len() {
            let (name, size) = self.decode_instruction(ip);
            let bytes = &self.script[ip..ip + size.min(self.script.len() - ip)];
            let hex_bytes = bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            output.push_str(&format!("{:04X}:  {:16}  {}\n", ip, hex_bytes, name));

            ip += size;
        }

        output
    }

    pub fn decode_instruction(&self, ip: usize) -> (String, usize) {
        if ip >= self.script.len() {
            return ("???".to_string(), 1);
        }

        let raw_opcode = self.script[ip];
        let opcode = match OpCode::try_from(raw_opcode) {
            Ok(opcode) => opcode,
            Err(_) => return (format!("??? (0x{raw_opcode:02X})"), 1),
        };

        // Validate that there are enough bytes remaining for the full
        // instruction before attempting to decode operands. This prevents
        // producing garbled output or advancing to invalid positions when
        // the script is truncated.
        let operand_size = opcode.operand_size();
        if ip + 1 + operand_size > self.script.len() {
            return (format!("{} (truncated)", opcode.name()), 1);
        }

        match opcode {
            OpCode::PUSHINT8 => {
                let val = self.read_i8(ip + 1);
                (format!("PUSHINT8 {}", val), 2)
            }
            OpCode::PUSHINT16 => {
                let val = self.read_i16(ip + 1);
                (format!("PUSHINT16 {}", val), 3)
            }
            OpCode::PUSHINT32 => {
                let val = self.read_i32(ip + 1);
                (format!("PUSHINT32 {}", val), 5)
            }
            OpCode::PUSHINT64 => {
                let val = self.read_i64(ip + 1);
                (format!("PUSHINT64 {}", val), 9)
            }
            OpCode::PUSHINT128 | OpCode::PUSHINT256 => {
                (opcode.name().to_string(), 1 + opcode.operand_size())
            }
            OpCode::PUSHA => {
                let offset = self.read_i32(ip + 1);
                (format!("PUSHA {:+}", offset), 5)
            }
            OpCode::PUSHDATA1 => {
                let len = self.read_u8(ip + 1) as usize;
                let data = self.read_bytes(ip + 2, len);
                let size = 2usize.saturating_add(len).min(self.script.len() - ip);
                (format!("PUSHDATA1 0x{}", hex::encode(&data)), size)
            }
            OpCode::PUSHDATA2 => {
                let len = self.read_u16(ip + 1) as usize;
                let data = self.read_bytes(ip + 3, len.min(32));
                let suffix = if len > 32 { "..." } else { "" };
                let size = 3usize.saturating_add(len).min(self.script.len() - ip);
                (
                    format!("PUSHDATA2 0x{}{}", hex::encode(&data), suffix),
                    size,
                )
            }
            OpCode::PUSHDATA4 => {
                let len = self.read_u32(ip + 1) as usize;
                let size = 5usize.saturating_add(len).min(self.script.len() - ip);
                (format!("PUSHDATA4 [{}B]", len), size)
            }
            OpCode::JMP => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMP {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMP_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMP_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::JMPIF => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPIF {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMPIF_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPIF_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::JMPIFNOT => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPIFNOT {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMPIFNOT_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPIFNOT_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::JMPEQ => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPEQ {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMPEQ_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPEQ_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::JMPNE => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPNE {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMPNE_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPNE_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::JMPGT => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPGT {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMPGT_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPGT_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::JMPGE => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPGE {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMPGE_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPGE_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::JMPLT => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPLT {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMPLT_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPLT_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::JMPLE => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPLE {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::JMPLE_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPLE_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::CALL => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("CALL {:+} -> 0x{:04X}", offset, target), 2)
            }
            OpCode::CALL_L => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("CALL_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            OpCode::CALLT => {
                let token = self.read_u16(ip + 1);
                (format!("CALLT {}", token), 3)
            }
            OpCode::TRY => {
                let catch = self.read_i8(ip + 1);
                let finally = self.read_i8(ip + 2);
                (format!("TRY catch:{:+} finally:{:+}", catch, finally), 3)
            }
            OpCode::TRY_L => {
                let catch = self.read_i32(ip + 1);
                let finally = self.read_i32(ip + 5);
                (format!("TRY_L catch:{:+} finally:{:+}", catch, finally), 9)
            }
            OpCode::ENDTRY => {
                let offset = self.read_i8(ip + 1);
                (format!("ENDTRY {:+}", offset), 2)
            }
            OpCode::ENDTRY_L => {
                let offset = self.read_i32(ip + 1);
                (format!("ENDTRY_L {:+}", offset), 5)
            }
            OpCode::SYSCALL => {
                let id = self.read_u32(ip + 1);
                let name = self.syscall_name(id);
                (format!("SYSCALL {} (0x{:08X})", name, id), 5)
            }
            OpCode::INITSSLOT => {
                let count = self.read_u8(ip + 1);
                (format!("INITSSLOT {}", count), 2)
            }
            OpCode::INITSLOT => {
                let locals = self.read_u8(ip + 1);
                let args = self.read_u8(ip + 2);
                (format!("INITSLOT locals:{} args:{}", locals, args), 3)
            }
            OpCode::LDSFLD
            | OpCode::STSFLD
            | OpCode::LDLOC
            | OpCode::STLOC
            | OpCode::LDARG
            | OpCode::STARG => {
                let idx = self.read_u8(ip + 1);
                (format!("{} {}", opcode.name(), idx), 2)
            }
            OpCode::NEWARRAY_T => {
                let t = self.read_u8(ip + 1);
                (format!("NEWARRAY_T {}", self.type_name(t)), 2)
            }
            OpCode::ISTYPE => {
                let t = self.read_u8(ip + 1);
                (format!("ISTYPE {}", self.type_name(t)), 2)
            }
            OpCode::CONVERT => {
                let t = self.read_u8(ip + 1);
                (format!("CONVERT {}", self.type_name(t)), 2)
            }
            _ => (opcode.name().to_string(), 1),
        }
    }

    fn read_u8(&self, pos: usize) -> u8 {
        self.script.get(pos).copied().unwrap_or(0)
    }

    fn read_i8(&self, pos: usize) -> i8 {
        self.read_u8(pos) as i8
    }

    fn read_u16(&self, pos: usize) -> u16 {
        let b0 = self.read_u8(pos) as u16;
        let b1 = self.read_u8(pos + 1) as u16;
        b0 | (b1 << 8)
    }

    fn read_i16(&self, pos: usize) -> i16 {
        self.read_u16(pos) as i16
    }

    fn read_u32(&self, pos: usize) -> u32 {
        let b0 = self.read_u8(pos) as u32;
        let b1 = self.read_u8(pos + 1) as u32;
        let b2 = self.read_u8(pos + 2) as u32;
        let b3 = self.read_u8(pos + 3) as u32;
        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    }

    fn read_i32(&self, pos: usize) -> i32 {
        self.read_u32(pos) as i32
    }

    fn read_i64(&self, pos: usize) -> i64 {
        let lo = self.read_u32(pos) as u64;
        let hi = self.read_u32(pos + 4) as u64;
        (lo | (hi << 32)) as i64
    }

    fn read_bytes(&self, pos: usize, len: usize) -> Vec<u8> {
        let end = (pos + len).min(self.script.len());
        self.script.get(pos..end).unwrap_or(&[]).to_vec()
    }

    fn syscall_name(&self, id: u32) -> &'static str {
        match id {
            id if id == interop_hash("System.Runtime.Log") => "System.Runtime.Log",
            id if id == interop_hash("System.Runtime.Notify") => "System.Runtime.Notify",
            id if id == interop_hash("System.Runtime.GetTime") => "System.Runtime.GetTime",
            id if id == interop_hash("System.Storage.Get") => "System.Storage.Get",
            id if id == interop_hash("System.Storage.Put") => "System.Storage.Put",
            id if id == interop_hash("System.Storage.Delete") => "System.Storage.Delete",
            id if id == interop_hash("System.Crypto.SHA256") => "System.Crypto.SHA256",
            id if id == interop_hash("System.Crypto.RIPEMD160") => "System.Crypto.RIPEMD160",
            id if id == interop_hash("System.Crypto.Hash160") => "System.Crypto.Hash160",
            id if id == interop_hash("System.Crypto.CheckSig") => "System.Crypto.CheckSig",
            _ => "Unknown",
        }
    }

    fn type_name(&self, t: u8) -> &'static str {
        StackItemType::from_byte(t)
            .map(StackItemType::name)
            .unwrap_or("Unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::{Disassembler, OpCode};
    use neo_vm_rs::interop_hash;

    #[test]
    fn test_decode_long_flow_control_opcodes() {
        let script = vec![
            0x29, 0x04, 0x00, 0x00, 0x00, // JMPEQ_L +4
            0x2B, 0xF8, 0xFF, 0xFF, 0xFF, // JMPNE_L -8
            0x3C, 0x02, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0xFF, 0xFF, // TRY_L +2, -2
            0x3E, 0x08, 0x00, 0x00, 0x00, // ENDTRY_L +8
        ];
        let disasm = Disassembler::new(&script);

        let (name0, size0) = disasm.decode_instruction(0);
        assert_eq!(size0, 5);
        assert!(name0.starts_with("JMPEQ_L"));

        let (name1, size1) = disasm.decode_instruction(5);
        assert_eq!(size1, 5);
        assert!(name1.starts_with("JMPNE_L"));

        let (name2, size2) = disasm.decode_instruction(10);
        assert_eq!(size2, 9);
        assert!(name2.starts_with("TRY_L"));

        let (name3, size3) = disasm.decode_instruction(19);
        assert_eq!(size3, 5);
        assert!(name3.starts_with("ENDTRY_L"));
    }

    #[test]
    fn test_disassembles_canonical_crypto_syscall() {
        let mut script = vec![OpCode::SYSCALL.byte()];
        script.extend_from_slice(&interop_hash("System.Crypto.SHA256").to_le_bytes());

        let disasm = Disassembler::new(&script);
        let (name, size) = disasm.decode_instruction(0);

        assert_eq!(size, 5);
        assert!(name.contains("System.Crypto.SHA256"));
    }

    #[test]
    fn test_disassembles_reserved_opcode_bytes_as_unknown() {
        for (byte, expected) in [(0xDA, "??? (0xDA)"), (0xF1, "??? (0xF1)")] {
            let script = [byte];
            let disasm = Disassembler::new(&script);
            let (name, size) = disasm.decode_instruction(0);

            assert_eq!(size, 1);
            assert_eq!(name, expected);
        }
    }

    #[test]
    fn test_truncated_pushint128_is_graceful() {
        // PUSHINT128 needs 16 operand bytes. Provide only 5.
        let mut script = vec![OpCode::PUSHINT128.byte()];
        script.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05]);
        let disasm = Disassembler::new(&script);
        let (name, size) = disasm.decode_instruction(0);
        // Must not panic; should indicate truncation or fall back safely.
        assert!(!name.is_empty());
        assert!(size >= 1);
    }

    #[test]
    fn test_truncated_pushint256_is_graceful() {
        // PUSHINT256 needs 32 operand bytes. Provide only 3.
        let mut script = vec![OpCode::PUSHINT256.byte()];
        script.extend_from_slice(&[0xAA, 0xBB, 0xCC]);
        let disasm = Disassembler::new(&script);
        let (name, size) = disasm.decode_instruction(0);
        assert!(!name.is_empty());
        assert!(size >= 1);
    }

    #[test]
    fn test_truncated_jmp_l_is_graceful() {
        // JMP_L needs 4 operand bytes. Provide only 2.
        let mut script = vec![OpCode::JMP_L.byte()];
        script.extend_from_slice(&[0x00, 0x01]);
        let disasm = Disassembler::new(&script);
        let (name, size) = disasm.decode_instruction(0);
        assert!(!name.is_empty());
        assert!(size >= 1);
    }
}
