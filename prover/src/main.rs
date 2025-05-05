use std::fs;
use std::path::Path;

use risc0_zkvm::{default_prover, ExecutorEnv};
use zescrow_core::escrow::EscrowState;
use zescrow_methods::{ZESCROW_GUEST_ELF, ZESCROW_GUEST_ID};

mod utils;

/// File containing escrow transaction details.
const ESCROW_METADATA_PATH: &str = "./templates/escrow_metadata.json";

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // An example Escrow transaction
    let escrow_json = fs::read_to_string(Path::new(ESCROW_METADATA_PATH))
        .expect("Failed to read escrow metadata JSON file.");
    let escrow_metadata: utils::EscrowMetadata =
        serde_json::from_str(&escrow_json).expect("Invalid escrow metadata JSON structure");
    let escrow = utils::map_escrow_metadata(escrow_metadata);

    // Dummy current block height
    // TODO: Fetch this via RPC
    let current_block: u64 = 1_250;

    let env = ExecutorEnv::builder()
        .write(&escrow)
        .unwrap()
        .write(&current_block)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prover_info = prover.prove(env, ZESCROW_GUEST_ELF).unwrap();
    let receipt = prover_info.receipt;

    match receipt.journal.decode::<EscrowState>() {
        Ok(escrow_state) => {
            if escrow_state == EscrowState::Released {
                println!("\nEscrow executed successfully!\n");
            } else {
                println!(
                    "\nINVALID escrow state: {:#?}. Execution failed!",
                    escrow_state
                );
            }
        }
        Err(_) => {
            let err: String = receipt.journal.decode().unwrap();
            println!("\nEscrow execution failed: {}\n", err);
        }
    }

    // Sanity check
    receipt
        .verify(ZESCROW_GUEST_ID)
        .expect("This should not happen!");
}
