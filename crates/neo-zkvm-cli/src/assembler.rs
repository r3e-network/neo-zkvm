//! Neo zkVM Assembler with macro support and syntax sugar
//!
//! Features:
//! - Full Neo N3 opcode support
//! - Macro definitions and expansion
//! - Labels and symbolic jumps
//! - Syntax sugar for common patterns
//! - Comprehensive error messages

#![allow(dead_code)]

use neo_vm_rs::interop_hash;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AssemblerError {
    UnknownOpcode(String, usize),
    InvalidOperand(String, usize),
    UndefinedLabel(String, usize),
    DuplicateLabel(String, usize),
    UndefinedMacro(String, usize),
    InvalidMacroDefinition(String, usize),
    SyntaxError(String, usize),
}

impl std::fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownOpcode(op, line) => write!(f, "Unknown opcode '{}' at line {}", op, line),
            Self::InvalidOperand(msg, line) => {
                write!(f, "Invalid operand at line {}: {}", line, msg)
            }
            Self::UndefinedLabel(label, line) => {
                write!(f, "Undefined label '{}' at line {}", label, line)
            }
            Self::DuplicateLabel(label, line) => {
                write!(f, "Duplicate label '{}' at line {}", label, line)
            }
            Self::UndefinedMacro(name, line) => {
                write!(f, "Undefined macro '{}' at line {}", name, line)
            }
            Self::InvalidMacroDefinition(msg, line) => {
                write!(f, "Invalid macro at line {}: {}", line, msg)
            }
            Self::SyntaxError(msg, line) => write!(f, "Syntax error at line {}: {}", line, msg),
        }
    }
}

#[derive(Debug, Clone)]
struct Macro {
    params: Vec<String>,
    body: Vec<String>,
}

#[derive(Debug, Clone)]
struct PendingLabel {
    pos: usize,
    base_ip: usize,
    label: String,
    line_num: usize,
    is_long_jump: bool,
}

const MAX_MACRO_DEPTH: usize = 100;

pub struct Assembler {
    labels: HashMap<String, usize>,
    macros: HashMap<String, Macro>,
    pending_labels: Vec<PendingLabel>,
    warnings: Vec<String>,
    macro_depth: usize,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            macros: HashMap::new(),
            pending_labels: Vec::new(),
            warnings: Vec::new(),
            macro_depth: 0,
        }
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn assemble(&mut self, source: &str) -> Result<Vec<u8>, String> {
        self.labels.clear();
        self.macros.clear();
        self.pending_labels.clear();
        self.warnings.clear();
        self.macro_depth = 0;

        // First pass: collect macros and labels
        let expanded = self.preprocess(source)?;

        // Second pass: generate bytecode
        let mut bytecode = Vec::new();

        for (line_num, line) in expanded.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            // Handle labels
            if line.ends_with(':') {
                let label = line.trim_end_matches(':').to_string();
                if self.labels.contains_key(&label) {
                    return Err(AssemblerError::DuplicateLabel(label, line_num + 1).to_string());
                }
                self.labels.insert(label, bytecode.len());
                continue;
            }

            self.assemble_line(line, &mut bytecode, line_num + 1)?;
        }

        // Resolve pending label references
        self.resolve_labels(&mut bytecode)?;

        Ok(bytecode)
    }

    fn preprocess(&mut self, source: &str) -> Result<Vec<String>, String> {
        let mut result = Vec::new();
        let mut in_macro = false;
        let mut current_macro_name = String::new();
        let mut current_macro_params = Vec::new();
        let mut current_macro_body = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Macro definition start
            if trimmed.starts_with(".macro") || trimmed.starts_with("%macro") {
                in_macro = true;
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 2 {
                    return Err(AssemblerError::InvalidMacroDefinition(
                        "Missing macro name".to_string(),
                        line_num + 1,
                    )
                    .to_string());
                }
                current_macro_name = parts[1].to_string();
                current_macro_params = parts[2..].iter().map(|s| s.to_string()).collect();
                current_macro_body.clear();
                continue;
            }

            // Macro definition end
            if trimmed == ".endmacro" || trimmed == "%endmacro" {
                in_macro = false;
                self.macros.insert(
                    current_macro_name.clone(),
                    Macro {
                        params: current_macro_params.clone(),
                        body: current_macro_body.clone(),
                    },
                );
                continue;
            }

            if in_macro {
                current_macro_body.push(line.to_string());
                continue;
            }

            // Macro invocation
            if trimmed.starts_with('%') && !trimmed.starts_with("%macro") {
                let expanded = self.expand_macro(trimmed, line_num + 1)?;
                result.extend(expanded);
                continue;
            }

            // Syntax sugar expansion
            let expanded = self.expand_sugar(trimmed, line_num + 1)?;
            result.extend(expanded);
        }

        if in_macro {
            return Err(AssemblerError::InvalidMacroDefinition(
                format!("Unterminated macro definition '{}'", current_macro_name),
                source.lines().count().max(1),
            )
            .to_string());
        }

        Ok(result)
    }

    fn expand_macro(&mut self, line: &str, line_num: usize) -> Result<Vec<String>, String> {
        if self.macro_depth >= MAX_MACRO_DEPTH {
            return Err(format!(
                "Macro expansion exceeded maximum depth {} at line {}",
                MAX_MACRO_DEPTH, line_num
            )
            .to_string());
        }
        self.macro_depth += 1;

        let parts: Vec<&str> = line.split_whitespace().collect();
        let name = parts[0].trim_start_matches('%');

        let macro_def = self.macros.get(name).ok_or_else(|| {
            AssemblerError::UndefinedMacro(name.to_string(), line_num).to_string()
        })?;

        let args: Vec<&str> = parts[1..].to_vec();

        if args.len() < macro_def.params.len() {
            self.macro_depth -= 1;
            return Err(format!(
                "Macro '{}' requires {} arguments but got {} at line {}",
                name,
                macro_def.params.len(),
                args.len(),
                line_num
            )
            .to_string());
        }

        let mut result = Vec::new();

        for body_line in &macro_def.body {
            let mut expanded = body_line.clone();
            for (i, param) in macro_def.params.iter().enumerate() {
                if i < args.len() {
                    expanded = expanded.replace(param, args[i]);
                }
            }
            result.push(expanded);
        }

        self.macro_depth -= 1;
        Ok(result)
    }

    fn expand_sugar(&self, line: &str, _line_num: usize) -> Result<Vec<String>, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(vec![line.to_string()]);
        }

        // Check if this looks like multiple simple opcodes on one line
        // (all parts are valid simple opcodes without operands)
        if parts.len() > 1 && parts.iter().all(|p| self.is_simple_opcode(p)) {
            return Ok(parts.iter().map(|s| s.to_uppercase()).collect());
        }

        let op = parts[0].to_uppercase();

        // Syntax sugar expansions
        match op.as_str() {
            // PUSH <n> - auto-select optimal push instruction
            "PUSH" if parts.len() > 1 => {
                if let Ok(n) = parts[1].parse::<i64>() {
                    return Ok(vec![self.optimal_push(n)]);
                }
            }
            // INC2, INC3, etc. - multiple increments
            s if s.starts_with("INC") && s.len() > 3 => {
                if let Ok(n) = s[3..].parse::<usize>() {
                    return Ok(vec!["INC".to_string(); n]);
                }
            }
            // DEC2, DEC3, etc. - multiple decrements
            s if s.starts_with("DEC") && s.len() > 3 => {
                if let Ok(n) = s[3..].parse::<usize>() {
                    return Ok(vec!["DEC".to_string(); n]);
                }
            }
            // DUP2, DUP3, etc. - multiple duplicates
            s if s.starts_with("DUP") && s.len() > 3 => {
                if let Ok(n) = s[3..].parse::<usize>() {
                    return Ok(vec!["DUP".to_string(); n]);
                }
            }
            // DROP2, DROP3, etc. - multiple drops
            s if s.starts_with("DROP") && s.len() > 4 => {
                if let Ok(n) = s[4..].parse::<usize>() {
                    return Ok(vec!["DROP".to_string(); n]);
                }
            }
            // NOP2, NOP3, etc. - multiple nops
            s if s.starts_with("NOP") && s.len() > 3 => {
                if let Ok(n) = s[3..].parse::<usize>() {
                    return Ok(vec!["NOP".to_string(); n]);
                }
            }
            _ => {}
        }

        Ok(vec![line.to_string()])
    }

    fn is_simple_opcode(&self, s: &str) -> bool {
        let op = s.to_uppercase();
        matches!(
            op.as_str(),
            "PUSH0"
                | "PUSH1"
                | "PUSH2"
                | "PUSH3"
                | "PUSH4"
                | "PUSH5"
                | "PUSH6"
                | "PUSH7"
                | "PUSH8"
                | "PUSH9"
                | "PUSH10"
                | "PUSH11"
                | "PUSH12"
                | "PUSH13"
                | "PUSH14"
                | "PUSH15"
                | "PUSH16"
                | "PUSHM1"
                | "PUSHNULL"
                | "TRUE"
                | "FALSE"
                | "NOP"
                | "RET"
                | "ABORT"
                | "ASSERT"
                | "THROW"
                | "THROWIFNOT"
                | "DEPTH"
                | "DROP"
                | "NIP"
                | "CLEAR"
                | "DUP"
                | "OVER"
                | "PICK"
                | "TUCK"
                | "SWAP"
                | "ROT"
                | "ROLL"
                | "REVERSE3"
                | "REVERSE4"
                | "REVERSEN"
                | "XDROP"
                | "ADD"
                | "SUB"
                | "MUL"
                | "DIV"
                | "MOD"
                | "POW"
                | "SQRT"
                | "SHL"
                | "SHR"
                | "INC"
                | "DEC"
                | "SIGN"
                | "ABS"
                | "NEGATE"
                | "NEG"
                | "INVERT"
                | "AND"
                | "OR"
                | "XOR"
                | "EQUAL"
                | "NOTEQUAL"
                | "NOT"
                | "BOOLAND"
                | "BOOLOR"
                | "NZ"
                | "LT"
                | "LE"
                | "GT"
                | "GE"
                | "MIN"
                | "MAX"
                | "WITHIN"
                | "NUMEQUAL"
                | "NUMNOTEQUAL"
                | "NEWARRAY0"
                | "NEWARRAY"
                | "NEWSTRUCT0"
                | "NEWSTRUCT"
                | "NEWMAP"
                | "SIZE"
                | "HASKEY"
                | "KEYS"
                | "VALUES"
                | "PICKITEM"
                | "APPEND"
                | "SETITEM"
                | "REVERSEITEMS"
                | "REMOVE"
                | "CLEARITEMS"
                | "POPITEM"
                | "PACK"
                | "UNPACK"
                | "ISNULL"
                | "SHA256"
                | "RIPEMD160"
                | "HASH160"
                | "CHECKSIG"
                | "LDLOC0"
                | "LDLOC1"
                | "LDLOC2"
                | "LDLOC3"
                | "LDLOC4"
                | "LDLOC5"
                | "STLOC0"
                | "STLOC1"
                | "STLOC2"
                | "STLOC3"
                | "STLOC4"
                | "STLOC5"
                | "LDARG0"
                | "LDARG1"
                | "LDARG2"
                | "LDARG3"
                | "LDARG4"
                | "LDARG5"
                | "STARG0"
                | "STARG1"
                | "STARG2"
                | "STARG3"
                | "STARG4"
                | "STARG5"
                | "LDSFLD0"
                | "LDSFLD1"
                | "LDSFLD2"
                | "LDSFLD3"
                | "LDSFLD4"
                | "LDSFLD5"
                | "STSFLD0"
                | "STSFLD1"
                | "STSFLD2"
                | "STSFLD3"
                | "STSFLD4"
                | "STSFLD5"
                | "NEWBUFFER"
                | "MEMCPY"
                | "CAT"
                | "SUBSTR"
                | "LEFT"
                | "RIGHT"
                | "PACKMAP"
                | "PACKSTRUCT"
                | "MODMUL"
                | "MODPOW"
                | "CALLA"
                | "ENDFINALLY"
                | "ABORTMSG"
                | "ASSERTMSG"
        )
    }

    fn optimal_push(&self, n: i64) -> String {
        match n {
            -1 => "PUSHM1".to_string(),
            0..=16 => format!("PUSH{}", n),
            -128..=127 => format!("PUSHINT8 {}", n),
            -32768..=32767 => format!("PUSHINT16 {}", n),
            _ if i32::try_from(n).is_ok() => format!("PUSHINT32 {}", n),
            _ => format!("PUSHINT64 {}", n),
        }
    }

    fn assemble_line(
        &mut self,
        line: &str,
        bytecode: &mut Vec<u8>,
        line_num: usize,
    ) -> Result<(), String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let op = parts[0].to_uppercase();
        let operands = &parts[1..];

        match op.as_str() {
            // Constants
            "PUSHINT8" => {
                bytecode.push(0x00);
                let val = self.parse_i8(operands, line_num)?;
                bytecode.push(val as u8);
            }
            "PUSHINT16" => {
                bytecode.push(0x01);
                let val = self.parse_i16(operands, line_num)?;
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            "PUSHINT32" => {
                bytecode.push(0x02);
                let val = self.parse_i32(operands, line_num)?;
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            "PUSHINT64" => {
                bytecode.push(0x03);
                let val = self.parse_int(operands, line_num)?;
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            "PUSHINT128" => {
                bytecode.push(0x04);
                let val = self.parse_i128(operands, line_num)?;
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            "PUSHINT256" => {
                bytecode.push(0x05);
                let bytes = if let Ok(val) = self.parse_i128(operands, line_num) {
                    let mut out = [0u8; 32];
                    out[..16].copy_from_slice(&val.to_le_bytes());
                    out[16..].fill(if val < 0 { 0xFF } else { 0x00 });
                    out.to_vec()
                } else {
                    let data = self.parse_data(operands, line_num)?;
                    if data.len() != 32 {
                        return Err(AssemblerError::InvalidOperand(
                            format!(
                                "PUSHINT256 requires either i128 value or 32-byte literal (got {} bytes)",
                                data.len()
                            ),
                            line_num,
                        )
                        .to_string());
                    }
                    data
                };
                bytecode.extend_from_slice(&bytes);
            }
            "PUSHA" => {
                bytecode.push(0x0A);
                let offset = self.parse_i32(operands, line_num)?;
                bytecode.extend_from_slice(&offset.to_le_bytes());
            }
            "PUSHNULL" => bytecode.push(0x0B),
            "PUSHDATA1" => {
                bytecode.push(0x0C);
                let data = self.parse_data(operands, line_num)?;
                let len = data.len();
                if len > 255 {
                    return Err(format!(
                        "PUSHDATA1 length {} exceeds maximum 255 at line {}",
                        len, line_num
                    )
                    .to_string());
                }
                bytecode.push(len as u8);
                bytecode.extend_from_slice(&data);
            }
            "PUSHDATA2" => {
                bytecode.push(0x0D);
                let data = self.parse_data(operands, line_num)?;
                let len = data.len();
                if len > u16::MAX as usize {
                    return Err(format!(
                        "PUSHDATA2 length {} exceeds maximum {} at line {}",
                        len,
                        u16::MAX,
                        line_num
                    )
                    .to_string());
                }
                bytecode.extend_from_slice(&(len as u16).to_le_bytes());
                bytecode.extend_from_slice(&data);
            }
            "PUSHDATA4" => {
                bytecode.push(0x0E);
                let data = self.parse_data(operands, line_num)?;
                let len = data.len();
                if len > u32::MAX as usize {
                    return Err(format!(
                        "PUSHDATA4 length {} exceeds maximum {} at line {}",
                        len,
                        u32::MAX,
                        line_num
                    ));
                }
                bytecode.extend_from_slice(&(len as u32).to_le_bytes());
                bytecode.extend_from_slice(&data);
            }
            "PUSHM1" => bytecode.push(0x0F),
            "PUSH0" | "PUSHF" | "FALSE" => bytecode.push(0x10),
            "PUSH1" | "PUSHT" | "TRUE" => bytecode.push(0x11),
            "PUSH2" => bytecode.push(0x12),
            "PUSH3" => bytecode.push(0x13),
            "PUSH4" => bytecode.push(0x14),
            "PUSH5" => bytecode.push(0x15),
            "PUSH6" => bytecode.push(0x16),
            "PUSH7" => bytecode.push(0x17),
            "PUSH8" => bytecode.push(0x18),
            "PUSH9" => bytecode.push(0x19),
            "PUSH10" => bytecode.push(0x1A),
            "PUSH11" => bytecode.push(0x1B),
            "PUSH12" => bytecode.push(0x1C),
            "PUSH13" => bytecode.push(0x1D),
            "PUSH14" => bytecode.push(0x1E),
            "PUSH15" => bytecode.push(0x1F),
            "PUSH16" => bytecode.push(0x20),

            // Flow control
            "NOP" => bytecode.push(0x21),
            "JMP" => {
                bytecode.push(0x22);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMP_L" => {
                bytecode.push(0x23);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "JMPIF" => {
                bytecode.push(0x24);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPIF_L" => {
                bytecode.push(0x25);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "JMPIFNOT" => {
                bytecode.push(0x26);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPIFNOT_L" => {
                bytecode.push(0x27);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "JMPEQ" => {
                bytecode.push(0x28);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPEQ_L" => {
                bytecode.push(0x29);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "JMPNE" => {
                bytecode.push(0x2A);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPNE_L" => {
                bytecode.push(0x2B);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "JMPGT" => {
                bytecode.push(0x2C);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPGT_L" => {
                bytecode.push(0x2D);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "JMPGE" => {
                bytecode.push(0x2E);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPGE_L" => {
                bytecode.push(0x2F);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "JMPLT" => {
                bytecode.push(0x30);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPLT_L" => {
                bytecode.push(0x31);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "JMPLE" => {
                bytecode.push(0x32);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPLE_L" => {
                bytecode.push(0x33);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "CALL" => {
                bytecode.push(0x34);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "CALL_L" => {
                bytecode.push(0x35);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "CALLA" => bytecode.push(0x36),
            "CALLT" => {
                bytecode.push(0x37);
                let token = self.parse_u16(operands, line_num)?;
                bytecode.extend_from_slice(&token.to_le_bytes());
            }
            "ABORT" => bytecode.push(0x38),
            "ASSERT" => bytecode.push(0x39),
            "THROW" => bytecode.push(0x3A),
            "TRY" => {
                bytecode.push(0x3B);
                if operands.len() != 2 {
                    return Err(AssemblerError::InvalidOperand(
                        "TRY requires two offsets: <catch> <finally>".to_string(),
                        line_num,
                    )
                    .to_string());
                }
                let base_ip = bytecode.len().checked_sub(1).ok_or_else(|| {
                    AssemblerError::SyntaxError("Missing opcode".to_string(), line_num).to_string()
                })?;
                self.emit_relative_offset_with_base(
                    bytecode,
                    operands[0],
                    line_num,
                    base_ip,
                    false,
                )?;
                self.emit_relative_offset_with_base(
                    bytecode,
                    operands[1],
                    line_num,
                    base_ip,
                    false,
                )?;
            }
            "TRY_L" => {
                bytecode.push(0x3C);
                if operands.len() != 2 {
                    return Err(AssemblerError::InvalidOperand(
                        "TRY_L requires two offsets: <catch> <finally>".to_string(),
                        line_num,
                    )
                    .to_string());
                }
                let base_ip = bytecode.len().checked_sub(1).ok_or_else(|| {
                    AssemblerError::SyntaxError("Missing opcode".to_string(), line_num).to_string()
                })?;
                self.emit_relative_offset_with_base(
                    bytecode,
                    operands[0],
                    line_num,
                    base_ip,
                    true,
                )?;
                self.emit_relative_offset_with_base(
                    bytecode,
                    operands[1],
                    line_num,
                    base_ip,
                    true,
                )?;
            }
            "ENDTRY" => {
                bytecode.push(0x3D);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "ENDTRY_L" => {
                bytecode.push(0x3E);
                self.emit_jump_offset_long(bytecode, operands, line_num)?;
            }
            "ENDFINALLY" => bytecode.push(0x3F),
            "RET" => bytecode.push(0x40),
            "SYSCALL" => {
                bytecode.push(0x41);
                let id = self.parse_syscall_id(operands, line_num)?;
                bytecode.extend_from_slice(&id.to_le_bytes());
            }

            // Stack operations
            "DEPTH" => bytecode.push(0x43),
            "DROP" => bytecode.push(0x45),
            "NIP" => bytecode.push(0x46),
            "XDROP" => bytecode.push(0x48),
            "CLEAR" => bytecode.push(0x49),
            "DUP" => bytecode.push(0x4A),
            "OVER" => bytecode.push(0x4B),
            "PICK" => bytecode.push(0x4D),
            "TUCK" => bytecode.push(0x4E),
            "SWAP" => bytecode.push(0x50),
            "ROT" => bytecode.push(0x51),
            "ROLL" => bytecode.push(0x52),
            "REVERSE3" => bytecode.push(0x53),
            "REVERSE4" => bytecode.push(0x54),
            "REVERSEN" => bytecode.push(0x55),

            // Slot operations
            "INITSSLOT" => {
                bytecode.push(0x56);
                let count = self.parse_u8(operands, line_num)?;
                bytecode.push(count);
            }
            "INITSLOT" => {
                bytecode.push(0x57);
                let (locals, args) = self.parse_slot_args(operands, line_num)?;
                bytecode.push(locals);
                bytecode.push(args);
            }
            "LDSFLD0" => bytecode.push(0x58),
            "LDSFLD1" => bytecode.push(0x59),
            "LDSFLD2" => bytecode.push(0x5A),
            "LDSFLD3" => bytecode.push(0x5B),
            "LDSFLD4" => bytecode.push(0x5C),
            "LDSFLD5" => bytecode.push(0x5D),
            "LDSFLD" => {
                bytecode.push(0x5E);
                let idx = self.parse_u8(operands, line_num)?;
                bytecode.push(idx);
            }
            "STSFLD0" => bytecode.push(0x5F),
            "STSFLD1" => bytecode.push(0x60),
            "STSFLD2" => bytecode.push(0x61),
            "STSFLD3" => bytecode.push(0x62),
            "STSFLD4" => bytecode.push(0x63),
            "STSFLD5" => bytecode.push(0x64),
            "STSFLD" => {
                bytecode.push(0x65);
                let idx = self.parse_u8(operands, line_num)?;
                bytecode.push(idx);
            }
            "LDLOC0" => bytecode.push(0x66),
            "LDLOC1" => bytecode.push(0x67),
            "LDLOC2" => bytecode.push(0x68),
            "LDLOC3" => bytecode.push(0x69),
            "LDLOC4" => bytecode.push(0x6A),
            "LDLOC5" => bytecode.push(0x6B),
            "LDLOC" => {
                bytecode.push(0x6C);
                let idx = self.parse_u8(operands, line_num)?;
                bytecode.push(idx);
            }
            "STLOC0" => bytecode.push(0x6D),
            "STLOC1" => bytecode.push(0x6E),
            "STLOC2" => bytecode.push(0x6F),
            "STLOC3" => bytecode.push(0x70),
            "STLOC4" => bytecode.push(0x71),
            "STLOC5" => bytecode.push(0x72),
            "STLOC" => {
                bytecode.push(0x73);
                let idx = self.parse_u8(operands, line_num)?;
                bytecode.push(idx);
            }
            "LDARG0" => bytecode.push(0x74),
            "LDARG1" => bytecode.push(0x75),
            "LDARG2" => bytecode.push(0x76),
            "LDARG3" => bytecode.push(0x77),
            "LDARG4" => bytecode.push(0x78),
            "LDARG5" => bytecode.push(0x79),
            "LDARG" => {
                bytecode.push(0x7A);
                let idx = self.parse_u8(operands, line_num)?;
                bytecode.push(idx);
            }
            "STARG0" => bytecode.push(0x7B),
            "STARG1" => bytecode.push(0x7C),
            "STARG2" => bytecode.push(0x7D),
            "STARG3" => bytecode.push(0x7E),
            "STARG4" => bytecode.push(0x7F),
            "STARG5" => bytecode.push(0x80),
            "STARG" => {
                bytecode.push(0x81);
                let idx = self.parse_u8(operands, line_num)?;
                bytecode.push(idx);
            }

            // Splice operations
            "NEWBUFFER" => bytecode.push(0x88),
            "MEMCPY" => bytecode.push(0x89),
            "CAT" => bytecode.push(0x8B),
            "SUBSTR" => bytecode.push(0x8C),
            "LEFT" => bytecode.push(0x8D),
            "RIGHT" => bytecode.push(0x8E),

            // Bitwise operations
            "INVERT" => bytecode.push(0x90),
            "AND" => bytecode.push(0x91),
            "OR" => bytecode.push(0x92),
            "XOR" => bytecode.push(0x93),
            "EQUAL" => bytecode.push(0x97),
            "NOTEQUAL" => bytecode.push(0x98),

            // Arithmetic
            "SIGN" => bytecode.push(0x99),
            "ABS" => bytecode.push(0x9A),
            "NEGATE" | "NEG" => bytecode.push(0x9B),
            "INC" => bytecode.push(0x9C),
            "DEC" => bytecode.push(0x9D),
            "ADD" => bytecode.push(0x9E),
            "SUB" => bytecode.push(0x9F),
            "MUL" => bytecode.push(0xA0),
            "DIV" => bytecode.push(0xA1),
            "MOD" => bytecode.push(0xA2),
            "POW" => bytecode.push(0xA3),
            "SQRT" => bytecode.push(0xA4),
            "MODMUL" => bytecode.push(0xA5),
            "MODPOW" => bytecode.push(0xA6),
            "SHL" => bytecode.push(0xA8),
            "SHR" => bytecode.push(0xA9),
            "NOT" => bytecode.push(0xAA),
            "BOOLAND" => bytecode.push(0xAB),
            "BOOLOR" => bytecode.push(0xAC),
            "NZ" => bytecode.push(0xB1),
            "NUMEQUAL" => bytecode.push(0xB3),
            "NUMNOTEQUAL" => bytecode.push(0xB4),
            "LT" => bytecode.push(0xB5),
            "LE" => bytecode.push(0xB6),
            "GT" => bytecode.push(0xB7),
            "GE" => bytecode.push(0xB8),
            "MIN" => bytecode.push(0xB9),
            "MAX" => bytecode.push(0xBA),
            "WITHIN" => bytecode.push(0xBB),

            // Compound types
            "PACKMAP" => bytecode.push(0xBE),
            "PACKSTRUCT" => bytecode.push(0xBF),
            "PACK" => bytecode.push(0xC0),
            "UNPACK" => bytecode.push(0xC1),
            "NEWARRAY0" => bytecode.push(0xC2),
            "NEWARRAY" => bytecode.push(0xC3),
            "NEWARRAY_T" => {
                bytecode.push(0xC4);
                let type_id = self.parse_u8(operands, line_num)?;
                bytecode.push(type_id);
            }
            "NEWSTRUCT0" => bytecode.push(0xC5),
            "NEWSTRUCT" => bytecode.push(0xC6),
            "NEWMAP" => bytecode.push(0xC8),
            "SIZE" => bytecode.push(0xCA),
            "HASKEY" => bytecode.push(0xCB),
            "KEYS" => bytecode.push(0xCC),
            "VALUES" => bytecode.push(0xCD),
            "PICKITEM" => bytecode.push(0xCE),
            "APPEND" => bytecode.push(0xCF),
            "SETITEM" => bytecode.push(0xD0),
            "REVERSEITEMS" => bytecode.push(0xD1),
            "REMOVE" => bytecode.push(0xD2),
            "CLEARITEMS" => bytecode.push(0xD3),
            "POPITEM" => bytecode.push(0xD4),

            // Types
            "ISNULL" => bytecode.push(0xD8),
            "ISTYPE" => {
                bytecode.push(0xD9);
                let type_id = self.parse_u8(operands, line_num)?;
                bytecode.push(type_id);
            }
            "TYPE" => bytecode.push(0xDA),
            "CONVERT" => {
                bytecode.push(0xDB);
                let type_id = self.parse_u8(operands, line_num)?;
                bytecode.push(type_id);
            }
            "ABORTMSG" => bytecode.push(0xE0),
            "ASSERTMSG" => bytecode.push(0xE1),
            "THROWIFNOT" => bytecode.push(0xF1),

            // Convenience syscall aliases. These are emitted as canonical
            // SYSCALL instructions instead of private crypto opcodes.
            "SHA256" => self.emit_named_syscall(bytecode, "System.Crypto.SHA256"),
            "RIPEMD160" => self.emit_named_syscall(bytecode, "System.Crypto.RIPEMD160"),
            "HASH160" => self.emit_named_syscall(bytecode, "System.Crypto.Hash160"),
            "CHECKSIG" => self.emit_named_syscall(bytecode, "System.Crypto.CheckSig"),

            // Raw byte emission
            "DB" | ".BYTE" => {
                for operand in operands {
                    let byte = self.parse_byte(operand, line_num)?;
                    bytecode.push(byte);
                }
            }

            _ => {
                return Err(AssemblerError::UnknownOpcode(op, line_num).to_string());
            }
        }

        Ok(())
    }

    fn emit_named_syscall(&self, bytecode: &mut Vec<u8>, name: &str) {
        bytecode.push(0x41);
        bytecode.extend_from_slice(&interop_hash(name).to_le_bytes());
    }

    fn emit_jump_offset(
        &mut self,
        bytecode: &mut Vec<u8>,
        operands: &[&str],
        line_num: usize,
    ) -> Result<(), String> {
        if operands.is_empty() {
            return Err(AssemblerError::InvalidOperand(
                "Missing jump target".to_string(),
                line_num,
            )
            .to_string());
        }

        let base_ip = bytecode.len().checked_sub(1).ok_or_else(|| {
            AssemblerError::SyntaxError("Missing opcode".to_string(), line_num).to_string()
        })?;
        self.emit_relative_offset_with_base(bytecode, operands[0], line_num, base_ip, false)
    }

    fn emit_jump_offset_long(
        &mut self,
        bytecode: &mut Vec<u8>,
        operands: &[&str],
        line_num: usize,
    ) -> Result<(), String> {
        if operands.is_empty() {
            return Err(AssemblerError::InvalidOperand(
                "Missing jump target".to_string(),
                line_num,
            )
            .to_string());
        }

        let base_ip = bytecode.len().checked_sub(1).ok_or_else(|| {
            AssemblerError::SyntaxError("Missing opcode".to_string(), line_num).to_string()
        })?;
        self.emit_relative_offset_with_base(bytecode, operands[0], line_num, base_ip, true)
    }

    fn emit_relative_offset_with_base(
        &mut self,
        bytecode: &mut Vec<u8>,
        target: &str,
        line_num: usize,
        base_ip: usize,
        is_long_jump: bool,
    ) -> Result<(), String> {
        let parse_as_number = Self::looks_numeric_literal(target);
        if parse_as_number {
            if is_long_jump {
                let offset = self.parse_i32(&[target], line_num)?;
                bytecode.extend_from_slice(&offset.to_le_bytes());
            } else {
                let offset = self.parse_i8(&[target], line_num)?;
                bytecode.push(offset as u8);
            }
            return Ok(());
        }

        self.pending_labels.push(PendingLabel {
            pos: bytecode.len(),
            base_ip,
            label: target.to_string(),
            line_num,
            is_long_jump,
        });
        if is_long_jump {
            bytecode.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            bytecode.push(0);
        }
        Ok(())
    }

    fn looks_numeric_literal(value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        let unsigned = value
            .strip_prefix('+')
            .or_else(|| value.strip_prefix('-'))
            .unwrap_or(value);
        if let Some(hex) = unsigned
            .strip_prefix("0x")
            .or_else(|| unsigned.strip_prefix("0X"))
        {
            return !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit());
        }
        !unsigned.is_empty() && unsigned.chars().all(|c| c.is_ascii_digit())
    }

    fn resolve_labels(&self, bytecode: &mut [u8]) -> Result<(), String> {
        for pending in &self.pending_labels {
            let target = self.labels.get(&pending.label).ok_or_else(|| {
                AssemblerError::UndefinedLabel(pending.label.clone(), pending.line_num).to_string()
            })?;

            let offset = (*target as isize) - (pending.base_ip as isize);

            if pending.is_long_jump {
                if i32::MIN as isize <= offset && offset <= i32::MAX as isize {
                    let offset_bytes = (offset as i32).to_le_bytes();
                    bytecode[pending.pos] = offset_bytes[0];
                    bytecode[pending.pos + 1] = offset_bytes[1];
                    bytecode[pending.pos + 2] = offset_bytes[2];
                    bytecode[pending.pos + 3] = offset_bytes[3];
                } else {
                    return Err(format!(
                        "Jump offset {} too large for long jump at line {}",
                        offset, pending.line_num
                    ));
                }
            } else if (-128..=127).contains(&offset) {
                bytecode[pending.pos] = offset as i8 as u8;
            } else {
                return Err(format!(
                    "Jump offset {} too large for short jump at line {}",
                    offset, pending.line_num
                ));
            }
        }

        Ok(())
    }

    fn parse_int(&self, operands: &[&str], line_num: usize) -> Result<i64, String> {
        if operands.is_empty() {
            return Err(AssemblerError::InvalidOperand(
                "Missing integer value".to_string(),
                line_num,
            )
            .to_string());
        }

        let s = operands[0];
        if s.starts_with("0x") || s.starts_with("0X") {
            i64::from_str_radix(&s[2..], 16)
        } else {
            s.parse()
        }
        .map_err(|_| {
            AssemblerError::InvalidOperand(format!("Invalid integer: {}", s), line_num).to_string()
        })
    }

    fn parse_i8(&self, operands: &[&str], line_num: usize) -> Result<i8, String> {
        let val = self.parse_int(operands, line_num)?;
        i8::try_from(val).map_err(|_| {
            AssemblerError::InvalidOperand(format!("Value {} out of i8 range", val), line_num)
                .to_string()
        })
    }

    fn parse_i16(&self, operands: &[&str], line_num: usize) -> Result<i16, String> {
        let val = self.parse_int(operands, line_num)?;
        i16::try_from(val).map_err(|_| {
            AssemblerError::InvalidOperand(format!("Value {} out of i16 range", val), line_num)
                .to_string()
        })
    }

    fn parse_i32(&self, operands: &[&str], line_num: usize) -> Result<i32, String> {
        let val = self.parse_int(operands, line_num)?;
        i32::try_from(val).map_err(|_| {
            AssemblerError::InvalidOperand(format!("Value {} out of i32 range", val), line_num)
                .to_string()
        })
    }

    fn parse_i128(&self, operands: &[&str], line_num: usize) -> Result<i128, String> {
        if operands.is_empty() {
            return Err(AssemblerError::InvalidOperand(
                "Missing integer value".to_string(),
                line_num,
            )
            .to_string());
        }

        let s = operands[0];
        if s.starts_with("0x") || s.starts_with("0X") {
            i128::from_str_radix(&s[2..], 16)
        } else {
            s.parse::<i128>()
        }
        .map_err(|_| {
            AssemblerError::InvalidOperand(format!("Invalid i128 value: {}", s), line_num)
                .to_string()
        })
    }

    fn parse_u8(&self, operands: &[&str], line_num: usize) -> Result<u8, String> {
        let val = self.parse_int(operands, line_num)?;
        if !(0..=255).contains(&val) {
            return Err(AssemblerError::InvalidOperand(
                format!("Value {} out of u8 range", val),
                line_num,
            )
            .to_string());
        }
        Ok(val as u8)
    }

    fn parse_u16(&self, operands: &[&str], line_num: usize) -> Result<u16, String> {
        let val = self.parse_int(operands, line_num)?;
        if !(0..=u16::MAX as i64).contains(&val) {
            return Err(AssemblerError::InvalidOperand(
                format!("Value {} out of u16 range", val),
                line_num,
            )
            .to_string());
        }
        Ok(val as u16)
    }

    fn parse_byte(&self, s: &str, line_num: usize) -> Result<u8, String> {
        let s = s.trim_start_matches("0x").trim_start_matches("0X");
        u8::from_str_radix(s, 16)
            .or_else(|_| s.parse())
            .map_err(|_| {
                AssemblerError::InvalidOperand(format!("Invalid byte: {}", s), line_num).to_string()
            })
    }

    fn parse_data(&self, operands: &[&str], line_num: usize) -> Result<Vec<u8>, String> {
        if operands.is_empty() {
            return Err(
                AssemblerError::InvalidOperand("Missing data".to_string(), line_num).to_string(),
            );
        }

        let s = operands.join(" ");

        // String literal
        if s.starts_with('"') && s.ends_with('"') {
            return Ok(s.as_bytes()[1..s.len() - 1].to_vec());
        }

        // Hex data
        let hex_str = s.trim_start_matches("0x").replace(" ", "");
        hex::decode(&hex_str).map_err(|_| {
            AssemblerError::InvalidOperand(format!("Invalid hex data: {}", s), line_num).to_string()
        })
    }

    fn parse_slot_args(&self, operands: &[&str], line_num: usize) -> Result<(u8, u8), String> {
        if operands.len() < 2 {
            return Err(AssemblerError::InvalidOperand(
                "INITSLOT requires two arguments: <locals> <args>".to_string(),
                line_num,
            )
            .to_string());
        }

        let locals = operands[0].parse().map_err(|_| {
            AssemblerError::InvalidOperand("Invalid locals count".to_string(), line_num).to_string()
        })?;
        let args = operands[1].parse().map_err(|_| {
            AssemblerError::InvalidOperand("Invalid args count".to_string(), line_num).to_string()
        })?;

        Ok((locals, args))
    }

    fn parse_syscall_id(&self, operands: &[&str], line_num: usize) -> Result<u32, String> {
        if operands.is_empty() {
            return Err(
                AssemblerError::InvalidOperand("Missing syscall ID".to_string(), line_num)
                    .to_string(),
            );
        }

        let s = operands[0];

        // Named syscalls
        match s.to_uppercase().as_str() {
            "LOG" => return Ok(interop_hash("System.Runtime.Log")),
            "NOTIFY" => return Ok(interop_hash("System.Runtime.Notify")),
            "GETTIME" => return Ok(interop_hash("System.Runtime.GetTime")),
            "STORAGE.GET" => return Ok(interop_hash("System.Storage.Get")),
            "STORAGE.PUT" => return Ok(interop_hash("System.Storage.Put")),
            "STORAGE.DELETE" => return Ok(interop_hash("System.Storage.Delete")),
            "SYSTEM.RUNTIME.LOG" => return Ok(interop_hash("System.Runtime.Log")),
            "SYSTEM.RUNTIME.NOTIFY" => return Ok(interop_hash("System.Runtime.Notify")),
            "SYSTEM.RUNTIME.GETTIME" => return Ok(interop_hash("System.Runtime.GetTime")),
            "SYSTEM.STORAGE.GET" => return Ok(interop_hash("System.Storage.Get")),
            "SYSTEM.STORAGE.PUT" => return Ok(interop_hash("System.Storage.Put")),
            "SYSTEM.STORAGE.DELETE" => return Ok(interop_hash("System.Storage.Delete")),
            "SYSTEM.CRYPTO.SHA256" => return Ok(interop_hash("System.Crypto.SHA256")),
            "SYSTEM.CRYPTO.RIPEMD160" => return Ok(interop_hash("System.Crypto.RIPEMD160")),
            "SYSTEM.CRYPTO.HASH160" => return Ok(interop_hash("System.Crypto.Hash160")),
            "SYSTEM.CRYPTO.CHECKSIG" => return Ok(interop_hash("System.Crypto.CheckSig")),
            _ => {}
        }

        // Numeric ID
        if s.starts_with("0x") || s.starts_with("0X") {
            u32::from_str_radix(&s[2..], 16)
        } else {
            s.parse()
        }
        .map_err(|_| {
            AssemblerError::InvalidOperand(format!("Invalid syscall ID: {}", s), line_num)
                .to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Assembler;
    use neo_vm_rs::interop_hash;

    #[test]
    fn test_pushint8_out_of_range_returns_error() {
        let mut assembler = Assembler::new();
        let err = assembler.assemble("PUSHINT8 128").unwrap_err();
        assert!(err.contains("out of i8 range"));
    }

    #[test]
    fn test_push_sugar_uses_pushint64_for_large_values() {
        let mut assembler = Assembler::new();
        let bytes = assembler.assemble("PUSH 2147483648").unwrap();
        assert_eq!(bytes[0], 0x03); // PUSHINT64
    }

    #[test]
    fn test_assemble_resets_state_between_calls() {
        let mut assembler = Assembler::new();
        assembler
            .assemble(
                "
                start:
                    PUSH1
                    RET
            ",
            )
            .unwrap();

        let err = assembler
            .assemble(
                "
                JMP start
                RET
            ",
            )
            .unwrap_err();
        assert!(err.contains("Undefined label"));
    }

    #[test]
    fn test_unterminated_macro_definition_returns_error() {
        let mut assembler = Assembler::new();
        let err = assembler
            .assemble(
                "
                .macro INC_TWO x
                    PUSH 1
                    ADD
            ",
            )
            .unwrap_err();
        assert!(err.contains("Unterminated macro definition"));
    }

    #[test]
    fn test_assembles_extended_flow_control_opcodes() {
        let mut assembler = Assembler::new();
        let bytes = assembler
            .assemble(
                "
                JMPIF_L 4
                TRY 2 -1
                TRY_L 256 -2
                ENDTRY_L 8
                ENDFINALLY
                CALL_L -4
                CALLT 513
                ABORTMSG
                ASSERTMSG
            ",
            )
            .unwrap();

        assert_eq!(bytes[0], 0x25); // JMPIF_L
        assert_eq!(&bytes[1..5], &[0x04, 0x00, 0x00, 0x00]);
        assert_eq!(&bytes[5..8], &[0x3B, 0x02, 0xFF]); // TRY
        assert_eq!(
            &bytes[8..17],
            &[0x3C, 0x00, 0x01, 0x00, 0x00, 0xFE, 0xFF, 0xFF, 0xFF]
        ); // TRY_L
        assert_eq!(&bytes[17..22], &[0x3E, 0x08, 0x00, 0x00, 0x00]); // ENDTRY_L
        assert_eq!(bytes[22], 0x3F); // ENDFINALLY
        assert_eq!(&bytes[23..28], &[0x35, 0xFC, 0xFF, 0xFF, 0xFF]); // CALL_L -4
        assert_eq!(&bytes[28..31], &[0x37, 0x01, 0x02]); // CALLT 513
        assert_eq!(&bytes[31..33], &[0xE0, 0xE1]); // ABORTMSG, ASSERTMSG
    }

    #[test]
    fn test_crypto_aliases_emit_canonical_syscalls() {
        let mut assembler = Assembler::new();
        let bytes = assembler.assemble("SHA256").unwrap();

        let mut expected = vec![0x41];
        expected.extend_from_slice(&interop_hash("System.Crypto.SHA256").to_le_bytes());

        assert_eq!(bytes, expected);
        assert_ne!(bytes, vec![0xF0]);
    }

    #[test]
    fn test_syscall_accepts_canonical_name() {
        let mut assembler = Assembler::new();
        let bytes = assembler.assemble("SYSCALL System.Crypto.SHA256").unwrap();

        let mut expected = vec![0x41];
        expected.extend_from_slice(&interop_hash("System.Crypto.SHA256").to_le_bytes());

        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_throwifnot_is_canonical_opcode() {
        let mut assembler = Assembler::new();
        let bytes = assembler.assemble("THROWIFNOT").unwrap();

        assert_eq!(bytes, vec![0xF1]);
    }

    #[test]
    fn test_try_supports_label_operands() {
        let mut assembler = Assembler::new();
        let bytes = assembler
            .assemble(
                "
                start:
                    TRY catch finally
                    PUSH0
                    JMP end
                catch:
                    PUSH1
                    ENDTRY end
                finally:
                    PUSH2
                    ENDFINALLY
                end:
                    RET
            ",
            )
            .unwrap();

        assert_eq!(&bytes[0..3], &[0x3B, 0x06, 0x09]); // TRY catch, finally
        assert_eq!(&bytes[4..6], &[0x22, 0x07]); // JMP end
        assert_eq!(&bytes[7..9], &[0x3D, 0x04]); // ENDTRY end
    }

    #[test]
    fn test_try_l_supports_label_operands() {
        let mut assembler = Assembler::new();
        let bytes = assembler
            .assemble(
                "
                start:
                    TRY_L catch finally
                    RET
                catch:
                    RET
                finally:
                    RET
            ",
            )
            .unwrap();

        assert_eq!(bytes[0], 0x3C);
        assert_eq!(&bytes[1..5], &[0x0A, 0x00, 0x00, 0x00]); // catch label
        assert_eq!(&bytes[5..9], &[0x0B, 0x00, 0x00, 0x00]); // finally label
    }

    #[test]
    fn test_assembles_extended_slot_and_type_opcodes() {
        let mut assembler = Assembler::new();
        let bytes = assembler
            .assemble(
                "
                INITSSLOT 3
                LDSFLD 2
                STSFLD 1
                STARG 4
                NEWARRAY_T 0x40
                ISTYPE 0x21
                CONVERT 0x28
            ",
            )
            .unwrap();

        assert_eq!(
            bytes,
            vec![
                0x56, 0x03, 0x5E, 0x02, 0x65, 0x01, 0x81, 0x04, 0xC4, 0x40, 0xD9, 0x21, 0xDB, 0x28
            ]
        );
    }

    #[test]
    fn test_istype_requires_type_operand() {
        let mut assembler = Assembler::new();
        let err = assembler.assemble("ISTYPE").unwrap_err();
        assert!(err.contains("Missing integer value"));
    }

    #[test]
    fn test_pushint128_and_pushint256_encoding() {
        let mut assembler = Assembler::new();
        let bytes = assembler.assemble("PUSHINT128 -1\nPUSHINT256 -1").unwrap();

        assert_eq!(bytes[0], 0x04);
        assert!(bytes[1..17].iter().all(|b| *b == 0xFF));

        assert_eq!(bytes[17], 0x05);
        assert!(bytes[18..50].iter().all(|b| *b == 0xFF));
    }

    #[test]
    fn test_pushint256_accepts_32_byte_literal() {
        let mut assembler = Assembler::new();
        let bytes = assembler
            .assemble(
                "PUSHINT256 0x000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F",
            )
            .unwrap();

        assert_eq!(bytes[0], 0x05);
        assert_eq!(bytes.len(), 33);
        assert_eq!(bytes[1], 0x00);
        assert_eq!(bytes[32], 0x1F);
    }
}
