//! The RISC Zero guest

use bincode::config::standard;
use risc0_zkvm::guest::env;
use zescrow_core::interface::ExecutionResult;
use zescrow_core::Escrow;

/// Expects from the host:
/// - `Escrow` object decoded from bytes containing escrow transaction details.
fn main() {
    let bytes: Vec<u8> = env::read_frame();
    let (mut escrow, _): (Escrow, _) =
        bincode::decode_from_slice(&bytes, standard()).expect("failed to decode from slice");

    let result = escrow
        .execute()
        .map(ExecutionResult::Ok)
        .unwrap_or_else(|e| ExecutionResult::Err(e.to_string()));

    let result = bincode::encode_to_vec(&result, standard()).expect("failed to encode to vec");
    env::commit_slice(&result);
}
