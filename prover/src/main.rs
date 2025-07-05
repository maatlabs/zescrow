use std::fs;
use std::time::Instant;

use bincode::config::standard;
use risc0_zkvm::{default_prover, ExecutorEnv};
use tracing::{debug, error, info};
use zescrow_core::interface::{ExecutionResult, ESCROW_METADATA_PATH};
use zescrow_core::{Escrow, EscrowMetadata, ExecutionState};
use zescrow_methods::{ZESCROW_GUEST_ELF, ZESCROW_GUEST_ID};

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    info!("Reading escrow metadata from {}", ESCROW_METADATA_PATH);
    let json =
        fs::read_to_string(ESCROW_METADATA_PATH).expect("Failed to read escrow metadata JSON file");
    debug!("Read metadata JSON ({} bytes)", json.len());

    let metadata: EscrowMetadata =
        serde_json::from_str(&json).expect("Invalid escrow metadata JSON");
    debug!(?metadata, "Parsed EscrowMetadata");

    let escrow = Escrow::from_metadata(metadata).expect("Failed to build Escrow from metadata");
    let escrow_bin = bincode::encode_to_vec(&escrow, standard()).expect("Failed to encode escrow");
    debug!("Encoded Escrow via bincode ({} bytes)", escrow_bin.len());

    let env = ExecutorEnv::builder()
        .write_frame(&escrow_bin)
        .build()
        .expect("Failed to build ExecutorEnv");

    info!("Starting zkVM proof generation");
    let start = Instant::now();
    let receipt = default_prover()
        .prove(env, ZESCROW_GUEST_ELF)
        .expect("Proof generation failed")
        .receipt;
    let dur = start.elapsed();
    info!(
        "Proof generated in {:?} (journal {} bytes)",
        dur,
        receipt.journal.bytes.len()
    );

    if let Err(e) = receipt.verify(ZESCROW_GUEST_ID) {
        error!("Receipt verification failed: {}", e);
        std::process::exit(1);
    }
    info!("Receipt verified successfully");

    debug!("Decoding journal ({} bytes)", receipt.journal.bytes.len());
    let (result, _) = bincode::decode_from_slice(&receipt.journal.bytes, standard())
        .expect("Failed to decode journal");
    match result {
        ExecutionResult::Ok(ExecutionState::ConditionsMet) => {
            println!("\nEscrow conditions fulfilled!\n");
        }
        ExecutionResult::Ok(state) => {
            println!("\nInvalid escrow state: {:?}\n", state);
        }
        ExecutionResult::Err(err) => {
            println!("\nExecution failed: {}\n", err);
        }
    }
}
