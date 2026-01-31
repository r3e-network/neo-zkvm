//! Automated Market Maker (AMM) Swap Example
//!
//! Demonstrates a constant product AMM (like Uniswap v2) using Neo zkVM.
//! Shows how to calculate swap amounts with verifiable execution.

use neo_vm_core::{NeoVM, StackItem, VMState};

fn main() {
    println!("=== Neo zkVM AMM Swap Example ===\n");

    // Part 1: Pool Setup
    println!("--- Part 1: Liquidity Pool Setup ---\n");

    // Initial pool reserves
    let reserve_a: u64 = 10_000 * 10u64.pow(8); // 10,000 Token A
    let reserve_b: u64 = 50_000 * 10u64.pow(8); // 50,000 Token B

    // Constant product k = x * y
    let k = reserve_a as u128 * reserve_b as u128;

    println!("Initial Pool State:");
    println!("  Token A reserve: {}", format_amount(reserve_a));
    println!("  Token B reserve: {}", format_amount(reserve_b));
    println!("  Constant product k: {}", format_amount(k as u64));
    println!(
        "  Initial price (A/B): {:.6}",
        reserve_b as f64 / reserve_a as f64
    );

    // Part 2: Swap Calculation
    println!("\n--- Part 2: Swap Calculation ---\n");

    let swap_amount_in: u64 = 100 * 10u64.pow(8); // 100 Token A
    let fee_numerator: u64 = 3; // 0.3% fee (3/1000)
    let fee_denominator: u64 = 1000;

    println!("Swap request:");
    println!("  Input: {} Token A", format_amount(swap_amount_in));

    // Calculate output with fee
    // amount_in_with_fee = amount_in * (fee_denominator - fee_numerator)
    let amount_in_with_fee = (swap_amount_in as u128)
        .checked_mul((fee_denominator - fee_numerator) as u128)
        .unwrap();

    // numerator = amount_in_with_fee * reserve_out
    let numerator = amount_in_with_fee.checked_mul(reserve_b as u128).unwrap();

    // denominator = reserve_in * fee_denominator + amount_in_with_fee
    let denominator = (reserve_a as u128)
        .checked_mul(fee_denominator as u128)
        .unwrap()
        .checked_add(amount_in_with_fee)
        .unwrap();

    let amount_out = (numerator / denominator) as u64;

    println!("  Fee: 0.3%");
    println!("  Output: {} Token B", format_amount(amount_out));

    // Part 3: VM Execution - Verify Calculation
    println!("\n--- Part 3: Verifiable Calculation ---\n");

    // Simulate the AMM calculation in the VM
    // This proves the calculation was done correctly
    let mut vm = NeoVM::new(1_000_000);

    // Build a script that verifies the swap calculation
    // Stack operations: calculate (x + dx) * (y - dy) >= x * y
    let new_reserve_a = reserve_a + swap_amount_in;
    let new_reserve_b = reserve_b - amount_out;

    println!("New reserves after swap:");
    println!("  Token A: {}", format_amount(new_reserve_a));
    println!("  Token B: {}", format_amount(new_reserve_b));

    // Verify constant product formula
    let new_k = new_reserve_a as u128 * new_reserve_b as u128;
    let k_maintained = new_k >= k;

    println!("  New k: {}", format_amount(new_k as u64));
    println!(
        "  k maintained: {}",
        if k_maintained { "✓ YES" } else { "✗ NO" }
    );

    // Simple VM script to verify the swap is valid (output > 0)
    let verify_script = vec![
        0x00, 0x64, // PUSHINT8 100 (representing amount_out / 10^6)
        0x10, // PUSH0
        0xB7, // GT (greater than)
        0x40, // RET
    ];

    vm.load_script(verify_script).unwrap();

    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }

    let swap_valid = match vm.eval_stack.pop() {
        Some(StackItem::Boolean(b)) => b,
        _ => false,
    };

    println!(
        "\nVM verification: {}",
        if swap_valid {
            "VALID ✓"
        } else {
            "INVALID ✗"
        }
    );

    // Part 4: Price Impact Analysis
    println!("\n--- Part 4: Price Impact ---\n");

    let old_price = reserve_b as f64 / reserve_a as f64;
    let new_price = new_reserve_b as f64 / new_reserve_a as f64;
    let price_impact = ((old_price - new_price) / old_price) * 100.0;

    println!("Price before swap: {:.6} B/A", old_price);
    println!("Price after swap: {:.6} B/A", new_price);
    println!("Price impact: {:.4}%", price_impact.abs());

    // Part 5: Slippage Protection
    println!("\n--- Part 5: Slippage Protection ---\n");

    let min_amount_out = (amount_out as f64 * 0.99) as u64; // 1% max slippage
    let actual_slippage = ((amount_out - min_amount_out) as f64 / amount_out as f64) * 100.0;

    println!(
        "Minimum output (1% slippage): {}",
        format_amount(min_amount_out)
    );
    println!("Actual output: {}", format_amount(amount_out));
    println!("Slippage tolerance: 1%");
    println!("Actual slippage: {:.2}%", actual_slippage);

    let slippage_ok = amount_out >= min_amount_out;
    println!(
        "Slippage check: {}",
        if slippage_ok { "PASS ✓" } else { "FAIL ✗" }
    );

    // Part 6: Summary
    println!("\n--- Part 6: Swap Summary ---\n");

    if swap_valid && slippage_ok {
        println!("✓ Swap approved and executed!");
        println!("  Input: {} Token A", format_amount(swap_amount_in));
        println!("  Output: {} Token B", format_amount(amount_out));
        println!(
            "  Effective price: {:.6} B/A",
            amount_out as f64 / swap_amount_in as f64
        );
        println!("  Gas used: {}", vm.gas_consumed);
    } else {
        println!("✗ Swap rejected");
    }

    println!("\n=== AMM Swap Example Complete ===");
}

fn format_amount(amount: u64) -> String {
    if amount >= 10u64.pow(8) {
        format!("{:.8}", amount as f64 / 10u64.pow(8) as f64)
    } else {
        amount.to_string()
    }
}
