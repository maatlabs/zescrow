use bincode::config::standard;
use risc0_zkvm::{default_prover, ExecutorEnv};
use zescrow_core::interface::{ExecutionResult, ESCROW_METADATA_PATH};
use zescrow_core::{Escrow, EscrowMetadata, ExecutionState};
use zescrow_methods::{ZESCROW_GUEST_ELF, ZESCROW_GUEST_ID};

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let s = std::fs::read_to_string(ESCROW_METADATA_PATH)
        .expect("Failed to read escrow metadata JSON file.");
    let metadata: EscrowMetadata = serde_json::from_str(&s).expect("Invalid escrow metadata JSON");
    let escrow = Escrow::from_metadata(metadata).expect("Failed to read escrow metadata");
    // Encode `escrow` into a byte slice
    let escrow =
        bincode::encode_to_vec(&escrow, standard()).expect("Failed to encode escrow to vec");

    let env = ExecutorEnv::builder().write_frame(&escrow).build().unwrap();
    let receipt = default_prover()
        .prove(env, ZESCROW_GUEST_ELF)
        .unwrap()
        .receipt;
    receipt.verify(ZESCROW_GUEST_ID).expect("Invalid receipt");

    let journal_bytes: Vec<u8> = receipt.journal.bytes.clone();
    let (public, _): (ExecutionResult, _) =
        bincode::decode_from_slice(&journal_bytes, standard()).expect("Failed to decode journal");

    match public {
        ExecutionResult::Ok(ExecutionState::ConditionsMet) => {
            println!("\nEscrow conditions fulfilled!\n")
        }
        ExecutionResult::Ok(state) => {
            println!("\nInvalid escrow state: {:?}\n", state)
        }
        ExecutionResult::Err(err) => {
            println!("\nExecution failed: {}\n", err)
        }
    }
}
