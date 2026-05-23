use neo_vm_guest::OpCode;

pub fn append_bounded_neo_vm_sequence(script: &mut Vec<u8>, byte: u8) {
    match byte % 12 {
        0 => script.push(OpCode::PUSH0.byte()),
        1 => script.extend_from_slice(&[OpCode::PUSH1.byte(), OpCode::DROP.byte()]),
        2 => script.extend_from_slice(&[
            OpCode::PUSH2.byte(),
            OpCode::PUSH3.byte(),
            OpCode::ADD.byte(),
            OpCode::DROP.byte(),
        ]),
        3 => script.extend_from_slice(&[
            OpCode::PUSH3.byte(),
            OpCode::PUSH1.byte(),
            OpCode::SUB.byte(),
            OpCode::DROP.byte(),
        ]),
        4 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::PUSH2.byte(),
            OpCode::MUL.byte(),
            OpCode::DROP.byte(),
        ]),
        5 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::PUSH1.byte(),
            OpCode::NUMEQUAL.byte(),
            OpCode::DROP.byte(),
        ]),
        6 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::NOT.byte(),
            OpCode::DROP.byte(),
        ]),
        7 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::DUP.byte(),
            OpCode::DROP.byte(),
            OpCode::DROP.byte(),
        ]),
        8 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::PUSH2.byte(),
            OpCode::SWAP.byte(),
            OpCode::DROP.byte(),
            OpCode::DROP.byte(),
        ]),
        9 => script.extend_from_slice(&[OpCode::PUSH1.byte(), OpCode::DROP.byte()]),
        10 => script.extend_from_slice(&[OpCode::PUSH2.byte(), OpCode::DROP.byte()]),
        _ => script.push(OpCode::NOP.byte()),
    }
}
