//! Unit tests for Neo VM Core

#[cfg(test)]
mod tests {
    use neo_vm_core::{NeoVM, StackItem, VMState};

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
    use neo_vm_core::{NeoVM, StackItem, VMState};

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

    #[test]
    fn test_mod_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x17, 0x13, 0xA2, 0x40]); // PUSH7, PUSH3, MOD, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
    }

    #[test]
    fn test_negate_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x9B, 0x40]); // PUSH5, NEGATE, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(-5)));
    }

    #[test]
    fn test_inc_dec_operations() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x9C, 0x9C, 0x9D, 0x40]); // PUSH5, INC, INC, DEC, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(6)));
    }

    #[test]
    fn test_abs_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x0F, 0x15, 0x9F, 0x9A, 0x40]); // PUSHM1, PUSH5, SUB, ABS, RET (-6 -> 6)

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(6)));
    }

    #[test]
    fn test_sign_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x99, 0x40]); // PUSH5, SIGN, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(1)));
    }

    #[test]
    fn test_min_max_operations() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x13, 0x17, 0xB9, 0x40]); // PUSH3, PUSH7, MIN, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
    }
}

#[cfg(test)]
mod stack_tests {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    #[test]
    fn test_dup_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x4A, 0x40]); // PUSH5, DUP, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.len(), 2);
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
    }

    #[test]
    fn test_drop_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x16, 0x45, 0x40]); // PUSH5, PUSH6, DROP, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.len(), 1);
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
    }

    #[test]
    fn test_swap_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x16, 0x50, 0x40]); // PUSH5, PUSH6, SWAP, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(6)));
    }

    #[test]
    fn test_over_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x16, 0x4B, 0x40]); // PUSH5, PUSH6, OVER, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.len(), 3);
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
    }

    #[test]
    fn test_nip_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x16, 0x46, 0x40]); // PUSH5, PUSH6, NIP, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.len(), 1);
        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(6)));
    }

    #[test]
    fn test_clear_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x16, 0x17, 0x49, 0x40]); // PUSH5, PUSH6, PUSH7, CLEAR, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.len(), 0);
    }

    #[test]
    fn test_depth_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x16, 0x17, 0x43, 0x40]); // PUSH5, PUSH6, PUSH7, DEPTH, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
    }
}

#[cfg(test)]
mod comparison_tests {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    #[test]
    fn test_lt_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x13, 0x15, 0xB5, 0x40]); // PUSH3, PUSH5, LT, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
    }

    #[test]
    fn test_gt_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x13, 0xB7, 0x40]); // PUSH5, PUSH3, GT, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
    }

    #[test]
    fn test_equal_operation() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0x15, 0x97, 0x40]); // PUSH5, PUSH5, EQUAL, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
    }
}

#[cfg(test)]
mod bitwise_tests {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    #[test]
    fn test_and_operation() {
        let mut vm = NeoVM::new(1_000_000);
        // 0x0F (15) AND 0x03 (3) = 0x03 (3)
        vm.load_script(vec![0x1F, 0x13, 0x91, 0x40]); // PUSH15, PUSH3, AND, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
    }

    #[test]
    fn test_or_operation() {
        let mut vm = NeoVM::new(1_000_000);
        // 0x08 (8) OR 0x03 (3) = 0x0B (11)
        vm.load_script(vec![0x18, 0x13, 0x92, 0x40]); // PUSH8, PUSH3, OR, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(11)));
    }

    #[test]
    fn test_xor_operation() {
        let mut vm = NeoVM::new(1_000_000);
        // 0x0F (15) XOR 0x03 (3) = 0x0C (12)
        vm.load_script(vec![0x1F, 0x13, 0x93, 0x40]); // PUSH15, PUSH3, XOR, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(12)));
    }
}

#[cfg(test)]
mod array_tests {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    #[test]
    fn test_newarray0() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0xC2, 0x40]); // NEWARRAY0, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Array(vec![])));
    }

    #[test]
    fn test_newarray_with_size() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x13, 0xC3, 0xCA, 0x40]); // PUSH3, NEWARRAY, SIZE, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(3)));
    }

    #[test]
    fn test_isnull() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x0B, 0xD8, 0x40]); // PUSHNULL, ISNULL, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
    }

    #[test]
    fn test_nz() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x15, 0xB1, 0x40]); // PUSH5, NZ, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Boolean(true)));
    }
}

#[cfg(test)]
mod control_flow_tests {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    #[test]
    fn test_nop() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x21, 0x15, 0x40]); // NOP, PUSH5, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
    }
}

#[cfg(test)]
mod pushdata_tests {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    #[test]
    fn test_pushint8() {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(vec![0x00, 0x7F, 0x40]); // PUSHINT8 127, RET

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(127)));
    }
}

#[cfg(test)]
mod slot_tests {
    use neo_vm_core::{NeoVM, StackItem, VMState};

    #[test]
    fn test_initslot() {
        let mut vm = NeoVM::new(1_000_000);
        // PUSH5, INITSLOT(1 local, 1 arg), LDARG0, RET
        vm.load_script(vec![0x15, 0x57, 0x01, 0x01, 0x74, 0x40]);

        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            vm.execute_next().unwrap();
        }

        assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
    }
}
