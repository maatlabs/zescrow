use bincode::config::standard;
use risc0_zkvm::guest::env;
use zescrow_core::interface::ExecutionResult;
use zescrow_core::Escrow;

/// Expects from the host:
/// - `Escrow` object containing escrow transaction details.
fn main() {
    // Read raw frame from host
    let bytes: Vec<u8> = env::read_frame();
    // Decode into our Escrow
    let (mut escrow, _): (Escrow, _) = bincode::decode_from_slice(&bytes, standard()).unwrap();
    // Execute and map to ExecutionResult
    let result = escrow
        .execute()
        .map(ExecutionResult::Ok)
        .unwrap_or_else(|e| ExecutionResult::Err(e.to_string()));
    // Re-encode and commit
    let result = bincode::encode_to_vec(&result, standard()).unwrap();
    env::commit_slice(&result);
}
