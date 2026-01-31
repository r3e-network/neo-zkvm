//! Basic VM usage example
use neo_vm_core::{NeoVM, VMState};

fn main() {
    let mut vm = NeoVM::new(1_000_000);

    // 2 + 3 = 5
    let _ = vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]);

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    println!("Result: {:?}", vm.eval_stack.pop());
}
