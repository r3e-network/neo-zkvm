//! Fuzz the NeoASM assembler with arbitrary source text.
//!
//! Goal: no panics; errors must stay structured `String`s.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use neo_zkvm_cli::Assembler;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let capped = if source.len() > 4_096 {
        &source[..4_096]
    } else {
        &source
    };

    let mut assembler = Assembler::new();
    let _ = assembler.assemble(capped);

    // Mix structured valid / invalid fragments for higher hit rate on real paths.
    if !data.is_empty() {
        let mut mixed = String::new();
        for b in data.iter().take(32) {
            match b % 8 {
                0 => mixed.push_str("PUSH1\n"),
                1 => mixed.push_str("PUSH2 PUSH3 ADD\n"),
                2 => mixed.push_str("NOP\n"),
                3 => mixed.push_str("DUP DROP\n"),
                4 => mixed.push_str("SHA256\n"),
                5 => mixed.push_str("CHECKSIG\n"), // must error, not panic
                6 => mixed.push_str(".macro m x\nPUSH x\n.endmacro\n%m 1\n"),
                _ => mixed.push_str("RET\n"),
            }
        }
        mixed.push_str("RET\n");
        let mut a2 = Assembler::new();
        let _ = a2.assemble(&mixed);
    }
});
