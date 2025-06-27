use std::fs;
use std::time::Instant;

use bincode::config::standard;
use groth_16_verifier::client::receipt_to_proof;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use tracing::{debug, error, info};
use zescrow_core::interface::{
    save_escrow_data, ExecutionResult, Groth16Proof, ProofData, ESCROW_METADATA_PATH,
    PROOF_DATA_PATH,
};
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
        .prove_with_opts(env, ZESCROW_GUEST_ELF, &ProverOpts::groth16())
        .expect("Proof generation failed")
        .receipt;

    let groth16_receipt = receipt
        .inner
        .groth16()
        .expect("Unable to get Groth16 proof from main receipt");

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

    debug!("Converting Groth16 receipt to Groth16 Proof");
    let proof =
        receipt_to_proof(groth16_receipt).expect("Unable to generate proof from Groth16 Receipt");

    let output = receipt.journal.bytes;

    debug!("Decoding journal ({} bytes)", output.len());
    let (result, _) =
        bincode::decode_from_slice(&output, standard()).expect("Failed to decode journal");

    match result {
        ExecutionResult::Ok(ExecutionState::ConditionsMet) => {
            println!("\nEscrow conditions fulfilled!\n");

            let proof_data = ProofData {
                image_id: ZESCROW_GUEST_ID,
                proof: Groth16Proof {
                    pi_a: proof.pi_a,
                    pi_b: proof.pi_b,
                    pi_c: proof.pi_c,
                },
                output,
            };
            // save proof data for on-chain verification
            save_escrow_data(PROOF_DATA_PATH, &proof_data)
                .expect("Failed to write proof data to JSON");
        }
        ExecutionResult::Ok(state) => {
            println!("\nInvalid escrow state: {:?}\n", state);
        }
        ExecutionResult::Err(err) => {
            println!("\nExecution failed: {}\n", err);
        }
    }
}
