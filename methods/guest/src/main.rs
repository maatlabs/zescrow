use risc0_zkvm::guest::env;
use zescrow_core::escrow::Escrow;

fn main() {
    let mut escrow: Escrow = env::read();

    match escrow.execute() {
        Ok(escrow) => env::commit(&escrow),
        Err(error) => env::commit(&format!("Escrow execution failed: {}", error)),
    }
}
