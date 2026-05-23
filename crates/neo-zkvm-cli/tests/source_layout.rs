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

#[test]
fn cli_disassembler_uses_shared_stack_item_type_names() {
    let source = read_source("src/disassembler.rs");

    assert!(
        source.contains("StackItemType::from_byte(t)")
            && source.contains(".map(StackItemType::name)"),
        "disassembler should format StackItemType operands through neo-vm-rs::StackItemType"
    );
    for duplicate in [
        "NEOVM_STACK_ITEM_TYPE_ANY",
        "NEOVM_STACK_ITEM_TYPE_POINTER",
        "NEOVM_STACK_ITEM_TYPE_BOOLEAN",
        "NEOVM_STACK_ITEM_TYPE_INTEGER",
        "NEOVM_STACK_ITEM_TYPE_BYTESTRING",
        "NEOVM_STACK_ITEM_TYPE_BUFFER",
        "NEOVM_STACK_ITEM_TYPE_ARRAY",
        "NEOVM_STACK_ITEM_TYPE_STRUCT",
        "NEOVM_STACK_ITEM_TYPE_MAP",
        "NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE",
    ] {
        assert!(
            !source.contains(duplicate),
            "disassembler must not keep a private StackItemType name table: {duplicate}"
        );
    }
}

#[test]
fn guest_crate_reexports_shared_opcode_enum() {
    let source = read_workspace_source("crates/neo-vm-guest/src/lib.rs");

    assert!(
        source.contains("OpCode"),
        "neo-vm-guest should re-export neo-vm-rs::OpCode for scripts built outside the CLI"
    );
}

#[test]
fn workspace_examples_build_scripts_from_shared_opcode_enum() {
    for relative_path in [
        "crates/neo-vm-guest/src/lib.rs",
        "crates/neo-vm-guest/tests/shared_vm.rs",
        "crates/neo-zkvm-cli/tests/integration_tests.rs",
        "crates/neo-zkvm-prover/src/lib.rs",
        "crates/neo-zkvm-verifier/src/lib.rs",
        "crates/neo-zkvm-program/src/main.rs",
        "crates/neo-zkvm-examples/src/basic.rs",
        "crates/neo-zkvm-examples/src/private_inputs.rs",
        "crates/neo-zkvm-examples/src/batch_verification.rs",
        "crates/neo-zkvm-examples/src/proof_generation.rs",
        "crates/neo-zkvm-examples/src/tamper_resistance.rs",
        "crates/neo-zkvm-examples/src/zk_preimage.rs",
        "fuzz/fuzz_targets/common.rs",
        "fuzz/fuzz_targets/fuzz_vm_execution.rs",
        "fuzz/fuzz_targets/fuzz_script_parser.rs",
    ] {
        let source = read_workspace_source(relative_path);

        assert!(
            source.contains("OpCode::") && source.contains(".byte()"),
            "{relative_path} should build NeoVM scripts through shared OpCode metadata"
        );
        for duplicate in [
            "vec![0x12, 0x13, 0x9E, 0x40]",
            "vec![0x15, 0x12, 0x9F, 0x40]",
            "vec![0x15, 0x10, 0xA1, 0x40]",
            "vec![0x4A, 0xA0, 0x40]",
            "0x12, 0x13, 0x9E, 0x45",
            "0x13, 0x11, 0x9F, 0x45",
            "0x11, 0x12, 0xA0, 0x45",
            "script.push(0x40)",
            "vec![0x41]",
        ] {
            assert!(
                !source.contains(duplicate),
                "{relative_path} must not duplicate NeoVM opcode byte constants: {duplicate}"
            );
        }
    }
}

fn read_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path))
        .unwrap_or_else(|error| panic!("{relative_path} should be readable: {error}"))
}

fn read_workspace_source(relative_path: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative_path),
    )
    .unwrap_or_else(|error| panic!("{relative_path} should be readable: {error}"))
}
