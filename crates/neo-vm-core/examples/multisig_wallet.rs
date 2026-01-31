//! Multi-Signature Wallet Example
//!
//! Demonstrates a 2-of-3 multi-signature wallet using Neo zkVM.
//! Requires 2 out of 3 signatures to authorize a transfer.

use neo_vm_core::{NeoVM, StackItem, VMState};

fn main() {
    println!("=== Neo zkVM Multi-Signature Wallet Example ===\n");

    // Part 1: Setup - Define signers and threshold
    println!("--- Part 1: Wallet Setup ---\n");

    let signer_a = b"pubkey_a_123456789012345678901234567890";
    let signer_b = b"pubkey_b_123456789012345678901234567890";
    let signer_c = b"pubkey_c_123456789012345678901234567890";

    println!("Multi-sig wallet: 2-of-3");
    println!("Signers:");
    println!("  A: {:?}...", String::from_utf8_lossy(&signer_a[..8]));
    println!("  B: {:?}...", String::from_utf8_lossy(&signer_b[..8]));
    println!("  C: {:?}...", String::from_utf8_lossy(&signer_c[..8]));

    // Part 2: Create a transfer proposal
    println!("\n--- Part 2: Transfer Proposal ---\n");

    let recipient = b"recipient_address_1234";
    let amount: u64 = 1000;

    println!("Proposed transfer:");
    println!("  Amount: {} GAS", amount);
    println!("  To: {:?}", String::from_utf8_lossy(recipient));

    // Part 3: Collect signatures (simulated via VM script)
    println!("\n--- Part 3: Signature Collection ---\n");

    // Simulate signature verification
    // In a real scenario, these would be actual ECDSA signatures
    let signatures_collected = vec![
        ("Signer A", true),  // Valid signature from A
        ("Signer B", true),  // Valid signature from B
        ("Signer C", false), // No signature from C
    ];

    let valid_count = signatures_collected.iter().filter(|(_, v)| *v).count();
    println!("Signatures collected:");
    for (name, valid) in &signatures_collected {
        println!(
            "  {}: {}",
            name,
            if *valid { "✓ Valid" } else { "✗ Missing" }
        );
    }
    println!("\nValid signatures: {}/3", valid_count);

    // Part 4: Verify threshold using VM
    println!("\n--- Part 4: Threshold Verification ---\n");

    let mut vm = NeoVM::new(1_000_000);

    // Script to check if threshold (2) is met
    // Stack: [sig_count, threshold]
    let threshold_script = vec![
        0x00,
        valid_count as u8, // PUSHINT8 <valid_count>
        0x12,              // PUSH2 (threshold)
        0xB8,              // GE (greater than or equal)
        0x40,              // RET
    ];

    vm.load_script(threshold_script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    let threshold_met = match vm.eval_stack.pop() {
        Some(StackItem::Boolean(b)) => b,
        _ => false,
    };

    println!(
        "Threshold check: {}",
        if threshold_met {
            "PASSED ✓"
        } else {
            "FAILED ✗"
        }
    );

    if threshold_met {
        println!("\n✓ Transfer approved! Executing...");

        // Part 5: Execute the transfer
        println!("\n--- Part 5: Transfer Execution ---\n");

        // In a real implementation, this would update balances
        println!("Transfer executed successfully!");
        println!(
            "  {} GAS sent to {:?}",
            amount,
            String::from_utf8_lossy(recipient)
        );
        println!("  Signatures: A, B");
        println!("  Transaction hash: 0x{}", hex::encode([0xABu8; 32]));
    } else {
        println!("\n✗ Transfer rejected - insufficient signatures");
    }

    // Part 6: Show gas usage
    println!("\n--- Part 6: Gas Analysis ---\n");
    println!("Verification gas used: {}", vm.gas_consumed);
    println!("State: {:?}", vm.state);

    // Part 7: Failed attempt simulation (1-of-3)
    println!("\n--- Part 7: Failed Attempt Simulation ---\n");

    let mut vm2 = NeoVM::new(1_000_000);
    let failed_script = vec![
        0x00, 0x01, // PUSHINT8 1 (only 1 signature)
        0x12, // PUSH2 (threshold)
        0xB8, // GE
        0x40, // RET
    ];

    vm2.load_script(failed_script).unwrap();

    while !matches!(vm2.state, VMState::Halt | VMState::Fault) {
        vm2.execute_next().unwrap();
    }

    let failed_threshold = match vm2.eval_stack.pop() {
        Some(StackItem::Boolean(b)) => b,
        _ => false,
    };

    println!(
        "Attempt with 1 signature: {}",
        if failed_threshold {
            "PASSED"
        } else {
            "REJECTED ✓"
        }
    );
    println!("(Correctly rejected - need at least 2 signatures)");

    println!("\n=== Multi-Sig Wallet Example Complete ===");
}
