//! The RISC Zero host (zkVM)

use std::fs;
use std::time::Instant;

use anyhow::Context;
use bincode::config::standard;
use risc0_zkvm::{default_prover, ExecutorEnv};
use tracing::{error, info};
use zescrow_core::interface::{ExecutionResult, ESCROW_METADATA_PATH};
use zescrow_core::{Escrow, EscrowMetadata, ExecutionState};
use zescrow_methods::{ZESCROW_GUEST_ELF, ZESCROW_GUEST_ID};

/// Runs the zero-knowledge proof workflow for an escrow.
pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    info!("Reading escrow metadata from {}", ESCROW_METADATA_PATH);
    let json = fs::read_to_string(ESCROW_METADATA_PATH)
        .with_context(|| "failed to read escrow metadata JSON file")?;

    let metadata: EscrowMetadata =
        serde_json::from_str(&json).with_context(|| "invalid escrow metadata JSON")?;

    let escrow =
        Escrow::from_metadata(metadata).with_context(|| "failed to build Escrow from metadata")?;
    let escrow_bin =
        bincode::encode_to_vec(&escrow, standard()).with_context(|| "failed to encode escrow")?;

    let env = ExecutorEnv::builder()
        .write_frame(&escrow_bin)
        .build()
        .with_context(|| "failed to build ExecutorEnv")?;

    info!("Starting zkVM proof generation");
    let start = Instant::now();
    let receipt = default_prover()
        .prove(env, ZESCROW_GUEST_ELF)
        .with_context(|| "Proof generation failed")?
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

    let (result, _) = bincode::decode_from_slice(&receipt.journal.bytes, standard())
        .with_context(|| "Failed to decode journal")?;
    match result {
        ExecutionResult::Ok(ExecutionState::ConditionsMet) => {
            info!("\nEscrow conditions fulfilled!\n");
        }
        ExecutionResult::Ok(state) => {
            info!("\nInvalid escrow state: {state:?}\n");
        }
        ExecutionResult::Err(err) => {
            info!("\nExecution failed: {err}\n");
        }
    }
    Ok(())
}
