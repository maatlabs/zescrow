use risc0_zkvm::guest::env;
use zescrow_core::escrow::Escrow;

/// Expects two inputs explicitly from the host:
/// - `Escrow` object containing escrow transaction details.
/// - `current_block` (u64) representing the current blockchain height.
fn main() {
    let mut escrow: Escrow = env::read();
    let current_block: u64 = env::read();

    match escrow.execute(current_block) {
        Ok(state) => env::commit(&state),
        Err(error) => env::commit(&format!("Escrow execution failed: {}", error)),
    }
}
