use std::path::PathBuf;

use anyhow::{anyhow, Context};
use clap::{value_parser, Parser, Subcommand};
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

    /// Generate a cryptographic condition JSON file.
    Generate(GenerateOpts),
}

/// Options for `generate` command
#[derive(Parser, Debug)]
struct GenerateOpts {
    #[command(subcommand)]
    condition: GenerateCmd,
}

#[derive(Subcommand, Debug)]
enum GenerateCmd {
    /// Hashlock: SHA256(preimage) == hash
    Hashlock {
        /// Read preimage from file
        #[arg(
            long,
            value_parser = value_parser!(PathBuf),
            help = "Path to preimage file"
        )]
        preimage: PathBuf,

        /// Output path for condition JSON
        #[arg(
            long,
            default_value = ESCROW_CONDITIONS_PATH,
            value_parser = value_parser!(PathBuf)
        )]
        output: PathBuf,
    },

    /// Ed25519 signature over a message
    Ed25519 {
        /// Hex-encoded public key
        #[arg(long, value_name = "PUBKEY", help = "Hex-encoded public key")]
        pubkey: String,

        /// Hex-encoded message
        #[arg(long, value_name = "MSG", help = "Hex-encoded message")]
        msg: String,

        /// Hex-encoded signature
        #[arg(long, value_name = "SIG", help = "Hex-encoded signature")]
        sig: String,

        /// Output path for condition JSON
        #[arg(
            long,
            default_value = ESCROW_CONDITIONS_PATH,
            value_parser = value_parser!(PathBuf)
        )]
        output: PathBuf,
    },

    /// Secp256k1 signature over a message
    Secp256k1 {
        /// Hex-encoded public key
        #[arg(long, value_name = "PUBKEY", help = "Hex-encoded public key")]
        pubkey: String,

        /// Hex-encoded message
        #[arg(long, value_name = "MSG", help = "Hex-encoded message")]
        msg: String,

        /// Hex-encoded signature
        #[arg(long, value_name = "SIG", help = "Hex-encoded signature")]
        sig: String,

        /// Output path for condition JSON
        #[arg(
            long,
            default_value = ESCROW_CONDITIONS_PATH,
            value_parser = value_parser!(PathBuf)
        )]
        output: PathBuf,
    },

    /// Threshold condition: at least `threshold` of the given
    /// subconditions must hold
    Threshold {
        /// One or more JSON files containing child conditions
        #[arg(
            long,
            value_name = "FILES...",
            value_parser = value_parser!(PathBuf),
            num_args = 1..,
            help = "Comma- or space-separated list of condition JSON files"
        )]
        subconditions: Vec<PathBuf>,

        /// Minimum number of child conditions required
        #[arg(long, help = "Number of conditions to satisfy")]
        threshold: usize,

        /// Output path for condition JSON
        #[arg(
            long,
            default_value = ESCROW_CONDITIONS_PATH,
            value_parser = value_parser!(PathBuf)
        )]
        output: PathBuf,
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
            let client = ZescrowClient::builder(&params.chain_config).build().await?;
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
            let client = ZescrowClient::builder(&metadata.params.chain_config)
                .recipient(recipient)
                .build()
                .await?;

            // Invoke the prover if escrow has cryptographic conditions
            if metadata.params.has_conditions {
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
            let client = ZescrowClient::builder(&metadata.params.chain_config)
                .build()
                .await?;

            info!("Cancelling escrow");
            client.cancel_escrow(&metadata).await?;
            info!("Escrow cancelled and refunded successfully");
        }

        Commands::Generate(opts) => {
            info!("Generating a new conditions JSON file");
            handle_generate_cmd(opts)?;
        }
    }
    Ok(())
}

fn handle_generate_cmd(opts: GenerateOpts) -> anyhow::Result<()> {
    match opts.condition {
        GenerateCmd::Hashlock { preimage, output } => {
            let preimage = std::fs::read_to_string(&preimage)
                .with_context(|| format!("reading preimage file {preimage:?}"))?;
            let hash = Sha256::digest(preimage.as_bytes());
            let cond = Condition::hashlock(hash.into(), preimage.into_bytes());
            save_escrow_data(output, &cond)?;
        }

        GenerateCmd::Ed25519 {
            pubkey,
            msg,
            sig,
            output,
        } => {
            let pk: [u8; 32] = hex::decode(&pubkey)?
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("pubkey wrong length"))?;
            let message = hex::decode(msg)?;
            let signature = hex::decode(sig)?;
            let cond = Condition::ed25519(pk, message, signature);
            save_escrow_data(output, &cond)?;
        }

        GenerateCmd::Secp256k1 {
            pubkey,
            msg,
            sig,
            output,
        } => {
            let pk = hex::decode(&pubkey)?;
            let message = hex::decode(msg)?;
            let signature = hex::decode(sig)?;
            let cond = Condition::secp256k1(pk, message, signature);
            save_escrow_data(output, &cond)?;
        }

        GenerateCmd::Threshold {
            subconditions,
            threshold,
            output,
        } => {
            let mut subs = Vec::with_capacity(subconditions.len());
            for path in subconditions {
                let c: Condition = load_escrow_data(&path)?;
                subs.push(c);
            }
            let cond = Condition::threshold(threshold, subs);
            save_escrow_data(output, &cond)?;
        }
    }
    Ok(())
}
