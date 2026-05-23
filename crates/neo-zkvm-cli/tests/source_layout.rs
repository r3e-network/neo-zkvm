use std::{fs, path::Path};

#[test]
fn cli_opcode_decoding_uses_shared_opcode_enum() {
    let source = read_source("src/disassembler.rs");

    assert!(
        source.contains("OpCode::try_from"),
        "disassembler should decode through neo-vm-rs::OpCode"
    );
    assert!(
        !source.contains("0x0F => (\"PUSHM1\".to_string(), 1)")
            && !source.contains("0x10 => (\"PUSH0\".to_string(), 1)")
            && !source.contains("0x21 => (\"NOP\".to_string(), 1)"),
        "disassembler must not maintain an independent hard-coded opcode table"
    );
}

#[test]
fn cli_gas_estimation_uses_shared_opcode_enum() {
    let source = read_source("src/main.rs");

    assert!(
        source.contains("OpCode::try_from"),
        "gas estimation should classify canonical opcodes through neo-vm-rs::OpCode"
    );
    assert!(
        !source.contains("0x00..=0x20")
            && !source.contains("0x21..=0x40")
            && !source.contains("0xA0..=0xDF"),
        "gas estimation must not classify opcode ranges by hard-coded byte values"
    );
}

#[test]
fn cli_assembler_uses_shared_opcode_enum() {
    let source = read_source("src/assembler.rs");

    assert!(
        source.contains("OpCode::from_name")
            && source.contains("OpCode::")
            && source.contains(".byte()"),
        "assembler should encode canonical opcodes through neo-vm-rs::OpCode"
    );
    assert!(
        !source.contains("bytecode.push(0x0F)")
            && !source.contains("bytecode.push(0x10)")
            && !source.contains("bytecode.push(0x5E)")
            && !source.contains("bytecode.push(0x81)")
            && !source.contains("\"PUSH0\" | \"PUSHF\" | \"FALSE\""),
        "assembler must not maintain an independent hard-coded opcode byte table"
    );
}

fn read_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path))
        .unwrap_or_else(|error| panic!("{relative_path} should be readable: {error}"))
}
