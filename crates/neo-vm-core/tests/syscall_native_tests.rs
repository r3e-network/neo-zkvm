//! VM syscall-level native contract tests.

use neo_vm_core::{engine::syscall, NeoVM, StackItem, VMState};

fn push_data(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() <= u8::MAX as usize);
    let mut script = vec![0x0C, bytes.len() as u8];
    script.extend_from_slice(bytes);
    script
}

fn emit_syscall(id: u32) -> Vec<u8> {
    let mut script = vec![0x41];
    script.extend_from_slice(&id.to_le_bytes());
    script
}

fn run_vm(vm: &mut NeoVM) {
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        if vm.execute_next().is_err() {
            break;
        }
    }
}

#[test]
fn test_native_syscall_sha256_returns_expected_hash() {
    let mut vm = NeoVM::new(1_000_000);
    let mut script = Vec::new();
    script.extend(push_data(b"abc"));
    script.extend(emit_syscall(syscall::SYSTEM_CRYPTO_SHA256));
    script.push(0x40); // RET

    vm.load_script(script).expect("script should load");
    run_vm(&mut vm);

    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(
        vm.eval_stack.pop(),
        Some(StackItem::ByteString(vec![
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ]))
    );
}

#[test]
fn test_native_syscall_ripemd160_returns_expected_hash() {
    let mut vm = NeoVM::new(1_000_000);
    let mut script = Vec::new();
    script.extend(push_data(b"abc"));
    script.extend(emit_syscall(syscall::SYSTEM_CRYPTO_RIPEMD160));
    script.push(0x40); // RET

    vm.load_script(script).expect("script should load");
    run_vm(&mut vm);

    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(
        vm.eval_stack.pop(),
        Some(StackItem::ByteString(vec![
            0x8e, 0xb2, 0x08, 0xf7, 0xe0, 0x5d, 0x98, 0x7a, 0x9b, 0x04, 0x4a, 0x8e, 0x98, 0xc6,
            0xb0, 0x87, 0xf1, 0x5a, 0x0b, 0xfc,
        ]))
    );
}

#[test]
fn test_native_syscall_base64_roundtrip() {
    let mut vm = NeoVM::new(1_000_000);
    let mut script = Vec::new();
    script.extend(push_data(b"neo-zkvm"));
    script.extend(emit_syscall(syscall::SYSTEM_STDLIB_BASE64_ENCODE));
    script.extend(emit_syscall(syscall::SYSTEM_STDLIB_BASE64_DECODE));
    script.push(0x40); // RET

    vm.load_script(script).expect("script should load");
    run_vm(&mut vm);

    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(
        vm.eval_stack.pop(),
        Some(StackItem::ByteString(b"neo-zkvm".to_vec()))
    );
}

#[test]
fn test_native_syscall_invalid_arg_type_faults() {
    let mut vm = NeoVM::new(1_000_000);
    let mut script = Vec::new();
    script.push(0x11); // PUSH1 (invalid type for sha256)
    script.extend(emit_syscall(syscall::SYSTEM_CRYPTO_SHA256));

    vm.load_script(script).expect("script should load");
    vm.execute_next().expect("push integer arg");
    let err = vm
        .execute_next()
        .expect_err("sha256 should reject integer input");
    assert!(err.to_string().contains("ByteString"));
}

#[test]
fn test_native_syscall_murmur32_returns_four_bytes() {
    let mut vm = NeoVM::new(1_000_000);
    let mut script = Vec::new();
    script.extend(push_data(b"abc"));
    script.push(0x10); // PUSH0 seed
    script.extend(emit_syscall(syscall::SYSTEM_CRYPTO_MURMUR32));
    script.push(0x40); // RET

    vm.load_script(script).expect("script should load");
    run_vm(&mut vm);

    assert!(matches!(vm.state, VMState::Halt));
    match vm.eval_stack.pop() {
        Some(StackItem::ByteString(bytes)) => assert_eq!(bytes.len(), 4),
        other => panic!("expected 4-byte murmur32 hash, got {other:?}"),
    }
}

#[test]
fn test_native_syscall_checksig_rejects_invalid_message_type() {
    let mut vm = NeoVM::new(1_000_000);
    let mut script = Vec::new();
    script.push(0x11); // PUSH1 (invalid message type)
    script.extend(push_data(&[1, 2, 3])); // signature bytes
    script.extend(push_data(&[4, 5, 6])); // pubkey bytes
    script.extend(emit_syscall(syscall::SYSTEM_CRYPTO_CHECKSIG));

    vm.load_script(script).expect("script should load");
    vm.execute_next().expect("push invalid message");
    vm.execute_next().expect("push signature");
    vm.execute_next().expect("push pubkey");
    let err = vm
        .execute_next()
        .expect_err("checkSig should reject non-bytes message");
    assert!(err.to_string().contains("checkSig"));
}
