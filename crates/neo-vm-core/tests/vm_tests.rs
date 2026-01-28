//! Unit tests for Neo VM Core

#[cfg(test)]
mod tests {
    use neo_vm_core::{NeoVM, VMState, StackItem};

    #[test]
    fn test_push_operations() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x11, 0x12, 0x13, 0x40]); // PUSH1, PUSH2, PUSH3, RET
        
        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }
        
        assert!(matches!(vm.state, VMState::Halt));
        assert_eq!(vm.eval_stack.len(), 3);
    }

    #[test]
    fn test_add_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]); // PUSH2, PUSH3, ADD, RET
        
        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }
        
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
    }

    #[test]
    fn test_sub_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x12, 0x9F, 0x40]); // PUSH5, PUSH2, SUB, RET
        
        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }
        
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
    }
}

#[cfg(test)]
mod arithmetic_tests {
    use neo_vm_core::{NeoVM, VMState, StackItem};

    #[test]
    fn test_mul_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x13, 0x14, 0xA0, 0x40]); // PUSH3, PUSH4, MUL, RET
        
        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }
        
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(12)));
    }

    #[test]
    fn test_div_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x18, 0x12, 0xA1, 0x40]); // PUSH8, PUSH2, DIV, RET
        
        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }
        
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(4)));
    }
}
