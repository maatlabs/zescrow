use risc0_zkvm::guest::env;
use zescrow_core::escrow::Escrow;

/// Expects from the host:
/// - `Escrow` object containing escrow transaction details.
fn main() {
    let mut escrow: Escrow = env::read();

    match escrow.execute() {
        Ok(state) => env::commit(&state),
        Err(error) => env::commit(&format!("Escrow execution failed: {}", error)),
    }
}
