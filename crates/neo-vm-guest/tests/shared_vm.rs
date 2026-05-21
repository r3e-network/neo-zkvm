use neo_vm_guest::{execute, ProofInput, StackItem};

fn syscall(name: &str) -> Vec<u8> {
    let mut script = vec![0x41];
    script.extend_from_slice(&neo_vm_rs::interop_hash(name).to_le_bytes());
    script
}

#[test]
fn proof_input_stack_item_is_the_shared_neo_vm_rs_type() {
    let item = StackItem::Integer(1);
    let shared: neo_vm_rs::StackValue = item;

    assert_eq!(shared, neo_vm_rs::StackValue::Integer(1));
}

#[test]
fn proof_guest_does_not_depend_on_legacy_neo_vm_core() {
    let manifest = include_str!("../Cargo.toml");
    let lib = include_str!("../src/lib.rs");

    assert!(!manifest.contains("neo-vm-core"));
    assert!(!lib.contains("neo_vm_core"));
}

#[test]
fn proof_execution_uses_canonical_throwifnot_opcode() {
    let output = execute(ProofInput {
        // PUSHDATA1 empty bytes, THROWIFNOT, RET.
        //
        // 0xF1 is canonical NeoVM THROWIFNOT. The previous zkVM-local engine
        // treated it as a RIPEMD160 pseudo opcode, which silently produced a
        // hash and halted instead of faulting on a falsey condition.
        script: vec![0x0c, 0x00, 0xf1, 0x40],
        arguments: vec![],
        gas_limit: 1_000_000,
    });

    assert_eq!(output.state, 1);
    assert!(output
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("THROWIFNOT"));
}

#[test]
fn deterministic_crypto_uses_syscall_not_pseudo_opcode() {
    let mut script = syscall("System.Crypto.SHA256");
    script.push(0x40);

    let output = execute(ProofInput {
        script,
        arguments: vec![StackItem::ByteString(b"abc".to_vec())],
        gas_limit: 1_000_000,
    });

    assert_eq!(output.state, 0);
    assert_eq!(
        output.result,
        Some(StackItem::ByteString(vec![
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ]))
    );
}
