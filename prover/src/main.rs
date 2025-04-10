use risc0_zkvm::{default_prover, ExecutorEnv};
use zescrow_core::condition::Condition;
use zescrow_core::escrow::{Escrow, EscrowState};
use zescrow_core::identity::{Asset, Party};
use zescrow_methods::{ZESCROW_GUEST_ELF, ZESCROW_GUEST_ID};

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // An example Escrow transaction
    // TODO: Fetch this as a JSON
    let escrow = Escrow {
        id: [0u8; 32],
        asset: Asset::Fungible {
            id: [0u8; 32],
            amount: 100,
        },
        beneficiary: Party {
            identity_hash: [1u8; 32],
        },
        depositor: Party {
            identity_hash: [2u8; 32],
        },
        condition: Condition::TimeLock {
            expiry_block: 1_500,
        },
        created_block: 1_000,
        expiry_block: 1_500,
        state: EscrowState::Funded,
    };

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

    match receipt.journal.decode::<Escrow>() {
        Ok(_escrow) => {
            println!("\nEscrow executed successfully!\n");
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
