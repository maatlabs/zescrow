use risc0_zkvm::{default_prover, ExecutorEnv};
use zescrow_core::interface::{ExecutionResult, ESCROW_METADATA_PATH};
use zescrow_core::{Escrow, EscrowMetadata, ExecutionState};
use zescrow_methods::{ZESCROW_GUEST_ELF, ZESCROW_GUEST_ID};

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let metadata = std::fs::read_to_string(ESCROW_METADATA_PATH)
        .expect("Failed to read escrow metadata JSON file.");
    let escrow_metadata: EscrowMetadata =
        serde_json::from_str(&metadata).expect("Invalid escrow metadata JSON");
    let escrow = Escrow::from_metadata(escrow_metadata).expect("Failed to read escrow metadata");

    let env = ExecutorEnv::builder()
        .write(&escrow)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prover_info = prover.prove(env, ZESCROW_GUEST_ELF).unwrap();
    let receipt = prover_info.receipt;

    receipt.verify(ZESCROW_GUEST_ID).expect("Invalid receipt");

    match receipt.journal.decode::<ExecutionResult>() {
        Ok(ExecutionResult::Ok(ExecutionState::ConditionsMet)) => {
            println!("\nEscrow conditions fulfilled!\n")
        }
        Ok(ExecutionResult::Ok(state)) => println!("\nInvalid escrow state: {:?}", state),
        Ok(ExecutionResult::Err(err)) => println!("\nExecution failed: {}\n", err),
        Err(e) => println!("Error decoding journal: {}\n", e),
    }
}
