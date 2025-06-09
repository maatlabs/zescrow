#![no_main]
#![no_std]

use risc0_zkvm::guest::{entry, env};
use zescrow_core::interface::ExecutionResult;
use zescrow_core::Escrow;

entry!(main);

/// Expects from the host:
/// - `Escrow` object containing escrow transaction details.
fn main() {
    let mut escrow: Escrow = env::read();

    let result = escrow
        .execute()
        .map(ExecutionResult::Ok)
        .unwrap_or_else(|e| ExecutionResult::Err(e.to_str()));

    env::commit(&result);
}
