//! Token Contract Example
//!
//! This example demonstrates a simple NEP-17 compatible token contract
//! that tracks balances and performs transfers with verifiable state.

use neo_vm_core::{
    NativeContract, NativeRegistry, NeoVM, StackItem, StorageBackend, StorageContext,
    TrackedStorage, VMState,
};

fn main() {
    println!("=== Neo zkVM Token Contract Example ===\n");

    // Initialize storage with tracking for auditability
    let mut storage = TrackedStorage::new();
    let ctx = StorageContext::default();

    // Contract owner
    let owner = b"owner_address_1234";
    let alice = b"alice_address_1234";
    let bob = b"bob_address_1234";

    // Part 1: Initialize Token Contract
    println!("--- Part 1: Token Initialization ---\n");

    // Mint initial supply to owner
    let initial_supply: u64 = 1_000_000_000 * 10u64.pow(8); // 1 billion NEOX
    storage.put(
        &ctx,
        &[b"balance:", owner.as_slice()].concat(),
        &initial_supply.to_le_bytes(),
    );
    storage.put(&ctx, b"total_supply", &initial_supply.to_le_bytes());
    storage.put(&ctx, b"symbol", b"NEOX");
    storage.put(&ctx, b"decimals", &[8u8]);

    println!("Token: NEOX (8 decimals)");
    println!("Initial supply: {} NEOX", format_tokens(initial_supply, 8));
    println!("Minted to: {:?}", String::from_utf8_lossy(owner));

    // Part 2: Transfer Tokens
    println!("\n--- Part 2: Token Transfer ---\n");

    // Owner transfers 10 NEOX to Alice
    let transfer_amount: u64 = 10 * 10u64.pow(8); // 10 NEOX

    // Get owner balance
    let owner_balance = get_balance(&storage, &ctx, owner);
    println!(
        "Owner balance before: {} NEOX",
        format_tokens(owner_balance, 8)
    );

    // Perform transfer (simplified - in real contract this would be VM execution)
    let new_owner_balance = owner_balance
        .checked_sub(transfer_amount)
        .expect("Insufficient balance");
    let alice_balance = get_balance(&storage, &ctx, alice)
        .checked_add(transfer_amount)
        .expect("Balance overflow");

    storage.put(
        &ctx,
        &[b"balance:", owner.as_slice()].concat(),
        &new_owner_balance.to_le_bytes(),
    );
    storage.put(
        &ctx,
        &[b"balance:", alice.as_slice()].concat(),
        &alice_balance.to_le_bytes(),
    );

    // Record transfer event
    record_transfer(&mut storage, &ctx, owner, alice, transfer_amount);

    println!("Transferred: {} NEOX", format_tokens(transfer_amount, 8));
    println!("  From: {:?}", String::from_utf8_lossy(owner));
    println!("  To: {:?}", String::from_utf8_lossy(alice));

    // Part 3: Check Balances
    println!("\n--- Part 3: Balance Check ---\n");

    println!(
        "Owner balance: {} NEOX",
        format_tokens(get_balance(&storage, &ctx, owner), 8)
    );
    println!(
        "Alice balance: {} NEOX",
        format_tokens(get_balance(&storage, &ctx, alice), 8)
    );
    println!(
        "Bob balance: {} NEOX",
        format_tokens(get_balance(&storage, &ctx, bob), 8)
    );

    // Part 4: VM Execution for Smart Contract Logic
    println!("\n--- Part 4: VM Contract Execution ---\n");

    // Create a VM to execute contract logic
    let mut vm = NeoVM::new(1_000_000);

    // Script to verify: 5 >= 2 (true - simulating sufficient balance check)
    let verification_script = vec![
        0x15, // PUSH5 (balance)
        0x12, // PUSH2 (required)
        0xB8, // GE (greater than or equal)
        0x40, // RET
    ];

    vm.load_script(verification_script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    if let Some(StackItem::Boolean(valid)) = vm.eval_stack.pop() {
        println!(
            "Transfer validation: {}",
            if valid { "VALID ✓" } else { "INVALID ✗" }
        );
    }

    // Part 5: State Verification with Merkle Proofs
    println!("\n--- Part 5: State Verification ---\n");

    // Get Merkle root of current state
    let merkle_root = storage.merkle_root();
    println!("State Merkle root: 0x{}", hex::encode(&merkle_root[..8]));

    // Review all changes
    println!("\nRecorded changes:");
    for (i, change) in storage.changes().iter().enumerate() {
        let key = String::from_utf8_lossy(&change.key);
        println!("  {}. Key: {}", i + 1, key);
        if let Some(old) = &change.old_value {
            println!("     Old: {} bytes", old.len());
        }
        if let Some(new) = &change.new_value {
            println!("     New: {} bytes", new.len());
        }
    }

    // Part 6: Native Contract Integration
    println!("\n--- Part 6: Crypto Verification ---\n");

    let registry = NativeRegistry::new();
    let stdlib_hash = registry.get_stdlib_hash();
    let cryptolib_hash = registry.get_cryptolib_hash();

    println!("StdLib contract: 0x{}", hex::encode(stdlib_hash));
    println!("CryptoLib contract: 0x{}", hex::encode(cryptolib_hash));

    // Simulate hash verification (used in real contracts)
    let test_data = b"transfer_signature_data";
    let _hash_result = neo_vm_core::CryptoLib::new()
        .invoke("sha256", vec![StackItem::ByteString(test_data.to_vec())]);

    println!("Signature verification: ready (data hashed)");

    println!("\n=== Token Contract Example Complete ===");
}

fn get_balance(storage: &TrackedStorage, ctx: &StorageContext, address: &[u8]) -> u64 {
    let key = [b"balance:", address].concat();
    match storage.get(ctx, &key) {
        Some(bytes) if bytes.len() == 8 => u64::from_le_bytes(bytes.try_into().unwrap()),
        _ => 0,
    }
}

fn record_transfer(
    storage: &mut TrackedStorage,
    ctx: &StorageContext,
    from: &[u8],
    to: &[u8],
    amount: u64,
) {
    // In a real implementation, this would append to a transfer log
    // For this example, we just store the latest transfer
    let transfer_key = b"last_transfer".to_vec();
    let transfer_data = [
        from,
        b"->".as_slice(),
        to,
        b":".as_slice(),
        &amount.to_le_bytes(),
    ]
    .concat();
    storage.put(ctx, &transfer_key, &transfer_data);
}

fn format_tokens(amount: u64, decimals: u32) -> String {
    let divisor = 10u64.pow(decimals);
    let whole = amount / divisor;
    let frac = amount % divisor;
    format!("{}.{:08}", whole, frac)
}

// Extension trait for NativeRegistry
pub trait NativeRegistryExt {
    fn get_stdlib_hash(&self) -> [u8; 20];
    fn get_cryptolib_hash(&self) -> [u8; 20];
}

impl NativeRegistryExt for NativeRegistry {
    fn get_stdlib_hash(&self) -> [u8; 20] {
        // Return the hash directly
        [
            0xac, 0xce, 0x6f, 0xd8, 0x0d, 0x44, 0xe1, 0xa3, 0x92, 0x6d, 0xe2, 0x1c, 0xcf, 0x30,
            0x96, 0x9a, 0x22, 0x4b, 0xc0, 0x6b,
        ]
    }

    fn get_cryptolib_hash(&self) -> [u8; 20] {
        [
            0x72, 0x6c, 0xb6, 0xe0, 0xcd, 0x8b, 0x0a, 0xc3, 0x3c, 0xe1, 0xde, 0xc0, 0xd4, 0x7e,
            0x5c, 0x3c, 0x4a, 0x6b, 0x8a, 0x0d,
        ]
    }
}
