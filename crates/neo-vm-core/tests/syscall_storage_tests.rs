//! VM syscall-level storage tests.

use neo_vm_core::{engine::syscall, NeoVM, StackItem, VMError, VMState};

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
fn test_storage_syscall_put_get_delete_roundtrip() {
    let mut vm = NeoVM::new(1_000_000);
    vm.set_storage_context([1u8; 20], false);

    let mut script = Vec::new();
    script.extend(push_data(b"k"));
    script.extend(push_data(b"v"));
    script.extend(emit_syscall(syscall::SYSTEM_STORAGE_PUT));
    script.extend(push_data(b"k"));
    script.extend(emit_syscall(syscall::SYSTEM_STORAGE_GET));
    script.extend(push_data(b"k"));
    script.extend(emit_syscall(syscall::SYSTEM_STORAGE_DELETE));
    script.extend(push_data(b"k"));
    script.extend(emit_syscall(syscall::SYSTEM_STORAGE_GET));
    script.push(0x40); // RET

    vm.load_script(script).expect("script should load");
    run_vm(&mut vm);

    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.len(), 2);
    assert_eq!(vm.eval_stack[0], StackItem::ByteString(b"v".to_vec()));
    assert_eq!(vm.eval_stack[1], StackItem::Null);
}

#[test]
fn test_storage_syscall_get_missing_returns_null() {
    let mut vm = NeoVM::new(1_000_000);
    vm.set_storage_context([2u8; 20], false);

    let mut script = Vec::new();
    script.extend(push_data(b"missing"));
    script.extend(emit_syscall(syscall::SYSTEM_STORAGE_GET));
    script.push(0x40); // RET

    vm.load_script(script).expect("script should load");
    run_vm(&mut vm);

    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Null));
}

#[test]
fn test_storage_syscall_put_respects_read_only_context() {
    let mut vm = NeoVM::new(1_000_000);
    vm.set_storage_context([3u8; 20], true);

    let mut script = Vec::new();
    script.extend(push_data(b"k"));
    script.extend(push_data(b"v"));
    script.extend(emit_syscall(syscall::SYSTEM_STORAGE_PUT));

    vm.load_script(script).expect("script should load");
    vm.execute_next().expect("push key");
    vm.execute_next().expect("push value");
    let err = vm
        .execute_next()
        .expect_err("put should fail in read-only mode");
    assert!(matches!(err, VMError::StorageReadOnly));
}

#[test]
fn test_storage_syscall_put_rejects_non_bytes_key() {
    let mut vm = NeoVM::new(1_000_000);
    vm.set_storage_context([4u8; 20], false);

    let mut script = Vec::new();
    script.push(0x11); // PUSH1 (integer key, invalid)
    script.extend(push_data(b"v"));
    script.extend(emit_syscall(syscall::SYSTEM_STORAGE_PUT));

    vm.load_script(script).expect("script should load");
    vm.execute_next().expect("push integer key");
    vm.execute_next().expect("push value");
    let err = vm
        .execute_next()
        .expect_err("put should reject non-bytes key type");
    assert!(matches!(err, VMError::InvalidType));
}

#[test]
fn test_storage_context_isolated_by_script_hash() {
    let mut vm = NeoVM::new(1_000_000);
    vm.set_storage_context([5u8; 20], false);

    let mut put_script = Vec::new();
    put_script.extend(push_data(b"k"));
    put_script.extend(push_data(b"v"));
    put_script.extend(emit_syscall(syscall::SYSTEM_STORAGE_PUT));
    put_script.push(0x40); // RET
    vm.load_script(put_script).expect("put script should load");
    run_vm(&mut vm);
    assert!(matches!(vm.state, VMState::Halt));

    vm.state = VMState::None;
    vm.eval_stack.clear();
    vm.set_storage_context([6u8; 20], false);

    let mut get_script = Vec::new();
    get_script.extend(push_data(b"k"));
    get_script.extend(emit_syscall(syscall::SYSTEM_STORAGE_GET));
    get_script.push(0x40); // RET
    vm.load_script(get_script).expect("get script should load");
    run_vm(&mut vm);

    assert!(matches!(vm.state, VMState::Halt));
    assert_eq!(vm.eval_stack.pop(), Some(StackItem::Null));
}
