//! The RISC Zero host (zkVM)

use anyhow::Context;
use bincode::config::standard;
use risc0_zkvm::{default_prover, ExecutorEnv};
use tracing::info;
use zescrow_core::interface::{ExecutionResult, ESCROW_METADATA_PATH};
use zescrow_core::{Escrow, EscrowMetadata, ExecutionState};
use zescrow_methods::{ZESCROW_GUEST_ELF, ZESCROW_GUEST_ID};

use crate::ClientError;

/// Executes the zero-knowledge proof workflow for an escrow transaction.
pub fn run() -> anyhow::Result<()> {
    info!("Reading escrow metadata from {}", ESCROW_METADATA_PATH);
    let json = std::fs::read_to_string(ESCROW_METADATA_PATH)
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
    let start = std::time::Instant::now();
    let receipt = default_prover()
        .prove(env, ZESCROW_GUEST_ELF)
        .with_context(|| "Proof generation failed")?
        .receipt;
    let dur = start.elapsed();
    info!(
        "Proof generated in {:?} (journal: {} bytes)",
        dur,
        receipt.journal.bytes.len()
    );

    info!("Verifying receipt");
    receipt
        .verify(ZESCROW_GUEST_ID)
        .map_err(|e| ClientError::ZkProver(e.to_string()))?;
    info!("Receipt verified successfully");

    let (result, _) = bincode::decode_from_slice(&receipt.journal.bytes, standard())
        .with_context(|| "Failed to decode journal")?;

    match result {
        ExecutionResult::Ok(ExecutionState::ConditionsMet) => {
            info!("\nEscrow conditions fulfilled!\n");
            Ok(())
        }
        ExecutionResult::Ok(state) => {
            Err(ClientError::ZkProver(format!("Invalid escrow state: {state:?}")).into())
        }
        ExecutionResult::Err(err) => {
            Err(ClientError::ZkProver(format!("Escrow conditions not fulfilled: {err:?}")).into())
        }
    }
}
