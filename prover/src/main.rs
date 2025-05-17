use risc0_zkvm::{default_prover, ExecutorEnv};
use zescrow_core::interface::ESCROW_METADATA_PATH;
use zescrow_core::{Escrow, EscrowMetadata, EscrowState};
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
