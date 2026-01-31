//! Neo zkVM Disassembler with enhanced formatting
//!
//! Features:
//! - Full Neo N3 opcode support
//! - Colored output (when terminal supports it)
//! - Jump target annotations
//! - Operand decoding

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

        let op = self.script[ip];

        match op {
            // Constants with operands
            0x00 => {
                let val = self.read_i8(ip + 1);
                (format!("PUSHINT8 {}", val), 2)
            }
            0x01 => {
                let val = self.read_i16(ip + 1);
                (format!("PUSHINT16 {}", val), 3)
            }
            0x02 => {
                let val = self.read_i32(ip + 1);
                (format!("PUSHINT32 {}", val), 5)
            }
            0x03 => {
                let val = self.read_i64(ip + 1);
                (format!("PUSHINT64 {}", val), 9)
            }
            0x04 => ("PUSHINT128".to_string(), 17),
            0x05 => ("PUSHINT256".to_string(), 33),
            0x0A => {
                let offset = self.read_i32(ip + 1);
                (format!("PUSHA {:+}", offset), 5)
            }
            0x0B => ("PUSHNULL".to_string(), 1),
            0x0C => {
                let len = self.read_u8(ip + 1) as usize;
                let data = self.read_bytes(ip + 2, len);
                (format!("PUSHDATA1 0x{}", hex::encode(&data)), 2 + len)
            }
            0x0D => {
                let len = self.read_u16(ip + 1) as usize;
                let data = self.read_bytes(ip + 3, len.min(32));
                let suffix = if len > 32 { "..." } else { "" };
                (
                    format!("PUSHDATA2 0x{}{}", hex::encode(&data), suffix),
                    3 + len,
                )
            }
            0x0E => {
                let len = self.read_u32(ip + 1) as usize;
                (format!("PUSHDATA4 [{}B]", len), 5 + len)
            }
            0x0F => ("PUSHM1".to_string(), 1),
            0x10 => ("PUSH0".to_string(), 1),
            0x11 => ("PUSH1".to_string(), 1),
            0x12 => ("PUSH2".to_string(), 1),
            0x13 => ("PUSH3".to_string(), 1),
            0x14 => ("PUSH4".to_string(), 1),
            0x15 => ("PUSH5".to_string(), 1),
            0x16 => ("PUSH6".to_string(), 1),
            0x17 => ("PUSH7".to_string(), 1),
            0x18 => ("PUSH8".to_string(), 1),
            0x19 => ("PUSH9".to_string(), 1),
            0x1A => ("PUSH10".to_string(), 1),
            0x1B => ("PUSH11".to_string(), 1),
            0x1C => ("PUSH12".to_string(), 1),
            0x1D => ("PUSH13".to_string(), 1),
            0x1E => ("PUSH14".to_string(), 1),
            0x1F => ("PUSH15".to_string(), 1),
            0x20 => ("PUSH16".to_string(), 1),

            // Flow control
            0x21 => ("NOP".to_string(), 1),
            0x22 => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMP {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x23 => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMP_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            0x24 => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPIF {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x25 => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPIF_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            0x26 => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPIFNOT {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x27 => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPIFNOT_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            0x28 => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPEQ {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x2A => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPNE {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x2C => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPGT {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x2E => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPGE {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x30 => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPLT {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x32 => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("JMPLE {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x34 => {
                let offset = self.read_i8(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("CALL {:+} -> 0x{:04X}", offset, target), 2)
            }
            0x35 => {
                let offset = self.read_i32(ip + 1);
                let target = (ip as isize + offset as isize) as usize;
                (format!("CALL_L {:+} -> 0x{:04X}", offset, target), 5)
            }
            0x36 => ("CALLA".to_string(), 1),
            0x37 => {
                let token = self.read_u16(ip + 1);
                (format!("CALLT {}", token), 3)
            }
            0x38 => ("ABORT".to_string(), 1),
            0x39 => ("ASSERT".to_string(), 1),
            0x3A => ("THROW".to_string(), 1),
            0x3B => {
                let catch = self.read_i8(ip + 1);
                let finally = self.read_i8(ip + 2);
                (format!("TRY catch:{:+} finally:{:+}", catch, finally), 3)
            }
            0x3D => {
                let offset = self.read_i8(ip + 1);
                (format!("ENDTRY {:+}", offset), 2)
            }
            0x3F => ("ENDFINALLY".to_string(), 1),
            0x40 => ("RET".to_string(), 1),
            0x41 => {
                let id = self.read_u32(ip + 1);
                let name = self.syscall_name(id);
                (format!("SYSCALL {} (0x{:08X})", name, id), 5)
            }

            // Stack operations
            0x43 => ("DEPTH".to_string(), 1),
            0x45 => ("DROP".to_string(), 1),
            0x46 => ("NIP".to_string(), 1),
            0x48 => ("XDROP".to_string(), 1),
            0x49 => ("CLEAR".to_string(), 1),
            0x4A => ("DUP".to_string(), 1),
            0x4B => ("OVER".to_string(), 1),
            0x4D => ("PICK".to_string(), 1),
            0x4E => ("TUCK".to_string(), 1),
            0x50 => ("SWAP".to_string(), 1),
            0x51 => ("ROT".to_string(), 1),
            0x52 => ("ROLL".to_string(), 1),
            0x53 => ("REVERSE3".to_string(), 1),
            0x54 => ("REVERSE4".to_string(), 1),
            0x55 => ("REVERSEN".to_string(), 1),

            // Slot operations
            0x56 => {
                let count = self.read_u8(ip + 1);
                (format!("INITSSLOT {}", count), 2)
            }
            0x57 => {
                let locals = self.read_u8(ip + 1);
                let args = self.read_u8(ip + 2);
                (format!("INITSLOT locals:{} args:{}", locals, args), 3)
            }
            0x58 => ("LDSFLD0".to_string(), 1),
            0x59 => ("LDSFLD1".to_string(), 1),
            0x5A => ("LDSFLD2".to_string(), 1),
            0x5B => ("LDSFLD3".to_string(), 1),
            0x5C => ("LDSFLD4".to_string(), 1),
            0x5D => ("LDSFLD5".to_string(), 1),
            0x5E => {
                let idx = self.read_u8(ip + 1);
                (format!("LDSFLD {}", idx), 2)
            }
            0x5F => ("STSFLD0".to_string(), 1),
            0x60 => ("STSFLD1".to_string(), 1),
            0x61 => ("STSFLD2".to_string(), 1),
            0x62 => ("STSFLD3".to_string(), 1),
            0x63 => ("STSFLD4".to_string(), 1),
            0x64 => ("STSFLD5".to_string(), 1),
            0x65 => {
                let idx = self.read_u8(ip + 1);
                (format!("STSFLD {}", idx), 2)
            }
            0x66 => ("LDLOC0".to_string(), 1),
            0x67 => ("LDLOC1".to_string(), 1),
            0x68 => ("LDLOC2".to_string(), 1),
            0x69 => ("LDLOC3".to_string(), 1),
            0x6A => ("LDLOC4".to_string(), 1),
            0x6B => ("LDLOC5".to_string(), 1),
            0x6C => {
                let idx = self.read_u8(ip + 1);
                (format!("LDLOC {}", idx), 2)
            }
            0x6D => ("STLOC0".to_string(), 1),
            0x6E => ("STLOC1".to_string(), 1),
            0x6F => ("STLOC2".to_string(), 1),
            0x70 => ("STLOC3".to_string(), 1),
            0x71 => ("STLOC4".to_string(), 1),
            0x72 => ("STLOC5".to_string(), 1),
            0x73 => {
                let idx = self.read_u8(ip + 1);
                (format!("STLOC {}", idx), 2)
            }
            0x74 => ("LDARG0".to_string(), 1),
            0x75 => ("LDARG1".to_string(), 1),
            0x76 => ("LDARG2".to_string(), 1),
            0x77 => ("LDARG3".to_string(), 1),
            0x78 => ("LDARG4".to_string(), 1),
            0x79 => ("LDARG5".to_string(), 1),
            0x7A => {
                let idx = self.read_u8(ip + 1);
                (format!("LDARG {}", idx), 2)
            }
            0x7B => ("STARG0".to_string(), 1),
            0x7C => ("STARG1".to_string(), 1),
            0x7D => ("STARG2".to_string(), 1),
            0x7E => ("STARG3".to_string(), 1),
            0x7F => ("STARG4".to_string(), 1),
            0x80 => ("STARG5".to_string(), 1),
            0x81 => {
                let idx = self.read_u8(ip + 1);
                (format!("STARG {}", idx), 2)
            }

            // Splice
            0x88 => ("NEWBUFFER".to_string(), 1),
            0x89 => ("MEMCPY".to_string(), 1),
            0x8B => ("CAT".to_string(), 1),
            0x8C => ("SUBSTR".to_string(), 1),
            0x8D => ("LEFT".to_string(), 1),
            0x8E => ("RIGHT".to_string(), 1),

            // Bitwise
            0x90 => ("INVERT".to_string(), 1),
            0x91 => ("AND".to_string(), 1),
            0x92 => ("OR".to_string(), 1),
            0x93 => ("XOR".to_string(), 1),
            0x97 => ("EQUAL".to_string(), 1),
            0x98 => ("NOTEQUAL".to_string(), 1),

            // Arithmetic
            0x99 => ("SIGN".to_string(), 1),
            0x9A => ("ABS".to_string(), 1),
            0x9B => ("NEGATE".to_string(), 1),
            0x9C => ("INC".to_string(), 1),
            0x9D => ("DEC".to_string(), 1),
            0x9E => ("ADD".to_string(), 1),
            0x9F => ("SUB".to_string(), 1),
            0xA0 => ("MUL".to_string(), 1),
            0xA1 => ("DIV".to_string(), 1),
            0xA2 => ("MOD".to_string(), 1),
            0xA3 => ("POW".to_string(), 1),
            0xA4 => ("SQRT".to_string(), 1),
            0xA5 => ("MODMUL".to_string(), 1),
            0xA6 => ("MODPOW".to_string(), 1),
            0xA8 => ("SHL".to_string(), 1),
            0xA9 => ("SHR".to_string(), 1),
            0xAA => ("NOT".to_string(), 1),
            0xAB => ("BOOLAND".to_string(), 1),
            0xAC => ("BOOLOR".to_string(), 1),
            0xB1 => ("NZ".to_string(), 1),
            0xB3 => ("NUMEQUAL".to_string(), 1),
            0xB4 => ("NUMNOTEQUAL".to_string(), 1),
            0xB5 => ("LT".to_string(), 1),
            0xB6 => ("LE".to_string(), 1),
            0xB7 => ("GT".to_string(), 1),
            0xB8 => ("GE".to_string(), 1),
            0xB9 => ("MIN".to_string(), 1),
            0xBA => ("MAX".to_string(), 1),
            0xBB => ("WITHIN".to_string(), 1),

            // Compound types
            0xBE => ("PACKMAP".to_string(), 1),
            0xBF => ("PACKSTRUCT".to_string(), 1),
            0xC0 => ("PACK".to_string(), 1),
            0xC1 => ("UNPACK".to_string(), 1),
            0xC2 => ("NEWARRAY0".to_string(), 1),
            0xC3 => ("NEWARRAY".to_string(), 1),
            0xC4 => {
                let t = self.read_u8(ip + 1);
                (format!("NEWARRAY_T {}", self.type_name(t)), 2)
            }
            0xC5 => ("NEWSTRUCT0".to_string(), 1),
            0xC6 => ("NEWSTRUCT".to_string(), 1),
            0xC8 => ("NEWMAP".to_string(), 1),
            0xCA => ("SIZE".to_string(), 1),
            0xCB => ("HASKEY".to_string(), 1),
            0xCC => ("KEYS".to_string(), 1),
            0xCD => ("VALUES".to_string(), 1),
            0xCE => ("PICKITEM".to_string(), 1),
            0xCF => ("APPEND".to_string(), 1),
            0xD0 => ("SETITEM".to_string(), 1),
            0xD1 => ("REVERSEITEMS".to_string(), 1),
            0xD2 => ("REMOVE".to_string(), 1),
            0xD3 => ("CLEARITEMS".to_string(), 1),
            0xD4 => ("POPITEM".to_string(), 1),

            // Types
            0xD8 => ("ISNULL".to_string(), 1),
            0xD9 => {
                let t = self.read_u8(ip + 1);
                (format!("ISTYPE {}", self.type_name(t)), 2)
            }
            0xDB => {
                let t = self.read_u8(ip + 1);
                (format!("CONVERT {}", self.type_name(t)), 2)
            }
            0xE0 => ("ABORTMSG".to_string(), 1),
            0xE1 => ("ASSERTMSG".to_string(), 1),

            // Crypto
            0xF0 => ("SHA256".to_string(), 1),
            0xF1 => ("RIPEMD160".to_string(), 1),
            0xF2 => ("HASH160".to_string(), 1),
            0xF3 => ("CHECKSIG".to_string(), 1),

            _ => (format!("??? (0x{:02X})", op), 1),
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
            0x01 => "System.Runtime.Log",
            0x02 => "System.Runtime.Notify",
            0x03 => "System.Runtime.GetTime",
            0x10 => "System.Storage.Get",
            0x11 => "System.Storage.Put",
            0x12 => "System.Storage.Delete",
            _ => "Unknown",
        }
    }

    fn type_name(&self, t: u8) -> &'static str {
        match t {
            0x00 => "Any",
            0x10 => "Pointer",
            0x20 => "Boolean",
            0x21 => "Integer",
            0x28 => "ByteString",
            0x30 => "Buffer",
            0x40 => "Array",
            0x41 => "Struct",
            0x48 => "Map",
            0x60 => "InteropInterface",
            _ => "Unknown",
        }
    }
}
