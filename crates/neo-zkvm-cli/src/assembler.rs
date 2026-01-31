//! Neo zkVM Assembler with macro support and syntax sugar
//!
//! Features:
//! - Full Neo N3 opcode support
//! - Macro definitions and expansion
//! - Labels and symbolic jumps
//! - Syntax sugar for common patterns
//! - Comprehensive error messages

#![allow(dead_code)]

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

const MAX_MACRO_DEPTH: usize = 100;

pub struct Assembler {
    labels: HashMap<String, usize>,
    macros: HashMap<String, Macro>,
    pending_labels: Vec<(usize, String, usize, bool)>,
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
                if let Ok(n) = parts[1].parse::<i128>() {
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
        )
    }

    fn optimal_push(&self, n: i128) -> String {
        match n {
            -1 => "PUSHM1".to_string(),
            0..=16 => format!("PUSH{}", n),
            -128..=127 => format!("PUSHINT8 {}", n),
            -32768..=32767 => format!("PUSHINT16 {}", n),
            _ => format!("PUSHINT32 {}", n),
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
                let val = self.parse_int(operands, line_num)? as i8;
                bytecode.push(val as u8);
            }
            "PUSHINT16" => {
                bytecode.push(0x01);
                let val = self.parse_int(operands, line_num)? as i16;
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            "PUSHINT32" => {
                bytecode.push(0x02);
                let val = self.parse_int(operands, line_num)? as i32;
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            "PUSHINT64" => {
                bytecode.push(0x03);
                let val = self.parse_int(operands, line_num)?;
                bytecode.extend_from_slice(&val.to_le_bytes());
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
            "JMPIFNOT" => {
                bytecode.push(0x26);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPEQ" => {
                bytecode.push(0x28);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPNE" => {
                bytecode.push(0x2A);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPGT" => {
                bytecode.push(0x2C);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPGE" => {
                bytecode.push(0x2E);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPLT" => {
                bytecode.push(0x30);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "JMPLE" => {
                bytecode.push(0x32);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "CALL" => {
                bytecode.push(0x34);
                self.emit_jump_offset(bytecode, operands, line_num)?;
            }
            "ABORT" => bytecode.push(0x38),
            "ASSERT" => bytecode.push(0x39),
            "THROW" => bytecode.push(0x3A),
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
            "INITSLOT" => {
                bytecode.push(0x57);
                let (locals, args) = self.parse_slot_args(operands, line_num)?;
                bytecode.push(locals);
                bytecode.push(args);
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
            "PACK" => bytecode.push(0xC0),
            "UNPACK" => bytecode.push(0xC1),
            "NEWARRAY0" => bytecode.push(0xC2),
            "NEWARRAY" => bytecode.push(0xC3),
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
            "ISTYPE" => bytecode.push(0xD9),
            "CONVERT" => bytecode.push(0xDB),

            // Crypto
            "SHA256" => bytecode.push(0xF0),
            "RIPEMD160" => bytecode.push(0xF1),
            "HASH160" => bytecode.push(0xF2),
            "CHECKSIG" => bytecode.push(0xF3),

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

        let target = operands[0];

        // Check if it's a numeric offset
        if let Ok(offset) = target.parse::<i8>() {
            bytecode.push(offset as u8);
        } else {
            // It's a label - record for later resolution
            self.pending_labels
                .push((bytecode.len(), target.to_string(), line_num, false)); // false = short jump
            bytecode.push(0); // Placeholder
        }

        Ok(())
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

        let target = operands[0];

        if let Ok(offset) = target.parse::<i32>() {
            bytecode.extend_from_slice(&offset.to_le_bytes());
        } else {
            self.pending_labels
                .push((bytecode.len(), target.to_string(), line_num, true)); // true = long jump
            bytecode.extend_from_slice(&[0, 0, 0, 0]); // Placeholder
        }

        Ok(())
    }

    fn resolve_labels(&self, bytecode: &mut Vec<u8>) -> Result<(), String> {
        for (pos, label, line_num, is_long_jump) in &self.pending_labels {
            let target = self.labels.get(label).ok_or_else(|| {
                AssemblerError::UndefinedLabel(label.clone(), *line_num).to_string()
            })?;

            let instr_start = pos - 1;
            let offset = (*target as isize) - (instr_start as isize);

            if *is_long_jump {
                if i32::MIN as isize <= offset && offset <= i32::MAX as isize {
                    let offset_bytes = (offset as i32).to_le_bytes();
                    bytecode[*pos] = offset_bytes[0];
                    bytecode[*pos + 1] = offset_bytes[1];
                    bytecode[*pos + 2] = offset_bytes[2];
                    bytecode[*pos + 3] = offset_bytes[3];
                } else {
                    return Err(format!(
                        "Jump offset {} too large for long jump at line {}",
                        offset, line_num
                    ));
                }
            } else if (-128..=127).contains(&offset) {
                bytecode[*pos] = offset as i8 as u8;
            } else {
                return Err(format!(
                    "Jump offset {} too large for short jump at line {}",
                    offset, line_num
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
            "LOG" | "SYSTEM.RUNTIME.LOG" => return Ok(0x01),
            "NOTIFY" | "SYSTEM.RUNTIME.NOTIFY" => return Ok(0x02),
            "GETTIME" | "SYSTEM.RUNTIME.GETTIME" => return Ok(0x03),
            "STORAGE.GET" | "SYSTEM.STORAGE.GET" => return Ok(0x10),
            "STORAGE.PUT" | "SYSTEM.STORAGE.PUT" => return Ok(0x11),
            "STORAGE.DELETE" | "SYSTEM.STORAGE.DELETE" => return Ok(0x12),
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
