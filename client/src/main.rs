use clap::{ArgGroup, Parser, Subcommand};
use sha2::{Digest, Sha256};
use tracing::info;
use zescrow_client::{prover, Recipient, ZescrowClient};
use zescrow_core::interface::{
    load_escrow_data, save_escrow_data, ESCROW_CONDITIONS_PATH, ESCROW_METADATA_PATH,
    ESCROW_PARAMS_PATH,
};
use zescrow_core::{Condition, EscrowMetadata, EscrowParams};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create an escrow using the specified parameters in
    /// `templates/escrow_params.json`.
    /// Generates `templates/escrow_metadata.json` on success.
    Create,

    /// Complete/release an existing escrow to the beneficiary.
    /// Reads `templates/escrow_metadata.json`.
    Finish {
        /// `RECIPIENT` is either:
        /// - a path to a keypair file (e.g., for Solana), or
        /// - a WIF-encoded private key
        #[arg(long, value_name = "RECIPIENT")]
        recipient: Recipient,
    },

    /// Cancel/refund an existing escrow to the creator.
    /// Reads `templates/escrow_metadata.json`.
    Cancel,

    /// Generate a cryptographic condition JSON for `escrow_conditions.json`.
    #[command(
    group(
        ArgGroup::new("kind")
            .required(true)
            .args(&[
                "preimage",
                "ed25519_pubkey","ed25519_sig","ed25519_msg",
                "secp_pubkey","secp_sig","secp_msg",
                "threshold"
        ]),
    ))]
    Generate {
        /// Preimage condition: supply a UTF-8 string
        #[arg(long, value_name = "PREIMAGE", group = "kind")]
        preimage: Option<String>,

        /// Ed25519 condition: hex pubkey and hex signature over message
        #[arg(long, value_name = "PUBKEY", group = "kind")]
        ed25519_pubkey: Option<String>,
        #[arg(long, value_name = "SIG", group = "kind")]
        ed25519_sig: Option<String>,
        #[arg(long, value_name = "MSG", group = "kind")]
        ed25519_msg: Option<String>,

        /// Secp256k1 condition: hex pubkey, hex signature, and hex message
        #[arg(long, value_name = "PUBKEY", group = "kind")]
        secp_pubkey: Option<String>,
        #[arg(long, value_name = "SIG", group = "kind")]
        secp_sig: Option<String>,
        #[arg(long, value_name = "MSG", group = "kind")]
        secp_msg: Option<String>,

        /// Threshold condition: comma-separated list of child condition files
        #[arg(long, value_name = "FILES", group = "kind")]
        threshold: Option<String>,
        /// Minimum number of child conditions required
        #[arg(long, value_name = "N")]
        subconditions: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    info!("Starting command handling");

    execute(cli.command).await
}

async fn execute(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Create => {
            info!("Loading escrow parameters from {}", ESCROW_PARAMS_PATH);
            let params: EscrowParams = load_escrow_data(ESCROW_PARAMS_PATH)?;

            info!("Building ZescrowClient");
            let client = ZescrowClient::builder(*params.asset.chain(), params.chain_config.clone())
                .build()
                .await?;
            info!("Creating escrow on-chain");
            let metadata = client.create_escrow(&params).await?;
            info!("Escrow created!");

            info!("Saving metadata to {}", ESCROW_METADATA_PATH);
            save_escrow_data(ESCROW_METADATA_PATH, &metadata)?;
        }

        Commands::Finish { recipient } => {
            info!("Loading escrow metadata from {}", ESCROW_METADATA_PATH);
            let metadata: EscrowMetadata = load_escrow_data(ESCROW_METADATA_PATH)?;

            info!("Building ZescrowClient for `finish`");
            let client = ZescrowClient::builder(
                metadata.chain_config.chain_id(),
                metadata.chain_config.clone(),
            )
            .recipient(recipient)
            .build()
            .await?;

            // Invoke the prover if escrow has cryptographic conditions
            if metadata.has_conditions {
                prover::run()?;
            }

            info!("Finishing escrow");
            client.finish_escrow(&metadata).await?;
            info!("Escrow completed and released successfully");
        }

        Commands::Cancel => {
            info!("Loading escrow metadata from {}", ESCROW_METADATA_PATH);
            let metadata: EscrowMetadata = load_escrow_data(ESCROW_METADATA_PATH)?;

            info!("Building ZescrowClient for `cancel`");
            let client = ZescrowClient::builder(
                metadata.chain_config.chain_id(),
                metadata.chain_config.clone(),
            )
            .build()
            .await?;

            info!("Cancelling escrow");
            client.cancel_escrow(&metadata).await?;
            info!("Escrow cancelled and refunded successfully");
        }

        Commands::Generate {
            preimage,
            ed25519_pubkey,
            ed25519_sig,
            ed25519_msg,
            secp_pubkey,
            secp_sig,
            secp_msg,
            threshold,
            subconditions,
        } => {
            let condition = if let Some(p) = preimage {
                let hash = Sha256::digest(p.as_bytes());
                Condition::preimage(hash.into(), p.into_bytes())
            } else if let (Some(pk), Some(sig), Some(msg)) =
                (ed25519_pubkey, ed25519_sig, ed25519_msg)
            {
                let pk: [u8; 32] = hex::decode(pk)?
                    .try_into()
                    .expect("Failed to convert array");
                let sig = hex::decode(sig)?;
                let msg = hex::decode(msg)?;
                Condition::ed25519(pk, msg, sig)
            } else if let (Some(pk), Some(sig), Some(msg)) = (secp_pubkey, secp_sig, secp_msg) {
                let pk = hex::decode(pk)?;
                let sig = hex::decode(sig)?;
                let msg = hex::decode(msg)?;
                Condition::secp256k1(pk, msg, sig)
            } else if let Some(files) = threshold {
                let n = subconditions.expect("Minimum number of valid subconditions required.");
                let mut subconds = Vec::new();
                for f in files.split(',') {
                    let cond: Condition = load_escrow_data(f)?;
                    subconds.push(cond);
                }
                Condition::threshold(n, subconds)
            } else {
                unreachable!("This shouldn't happen!");
            };

            info!("Generating a new `templates/escrow_conditions.json`");
            save_escrow_data(ESCROW_CONDITIONS_PATH, &condition)?;
        }
    }
    Ok(())
}
