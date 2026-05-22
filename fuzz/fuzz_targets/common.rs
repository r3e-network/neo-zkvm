pub fn append_bounded_neo_vm_sequence(script: &mut Vec<u8>, byte: u8) {
    match byte % 12 {
        0 => script.extend_from_slice(&[0x10]),       // PUSH0
        1 => script.extend_from_slice(&[0x11, 0x45]), // PUSH1 DROP
        2 => script.extend_from_slice(&[0x12, 0x13, 0x9E, 0x45]), // PUSH2 PUSH3 ADD DROP
        3 => script.extend_from_slice(&[0x13, 0x11, 0x9F, 0x45]), // PUSH3 PUSH1 SUB DROP
        4 => script.extend_from_slice(&[0x11, 0x12, 0xA0, 0x45]), // PUSH1 PUSH2 MUL DROP
        5 => script.extend_from_slice(&[0x11, 0x11, 0xB3, 0x45]), // PUSH1 PUSH1 NUMEQUAL DROP
        6 => script.extend_from_slice(&[0x11, 0xAA, 0x45]), // PUSH1 NOT DROP
        7 => script.extend_from_slice(&[0x11, 0x4A, 0x45, 0x45]), // PUSH1 DUP DROP DROP
        8 => script.extend_from_slice(&[0x11, 0x12, 0x50, 0x45, 0x45]), // PUSH1 PUSH2 SWAP DROP DROP
        9 => script.extend_from_slice(&[0x11, 0x45]),                   // PUSH1 DROP
        10 => script.extend_from_slice(&[0x12, 0x45]),                  // PUSH2 DROP
        _ => script.extend_from_slice(&[0x21]),                         // NOP
    }
}
