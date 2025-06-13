use clap::{Parser, Subcommand};
use tracing::{debug, info, instrument};
use zescrow_client::{Recipient, ZescrowClient};
use zescrow_core::interface::{
    load_escrow_data, save_escrow_data, ESCROW_METADATA_PATH, ESCROW_PARAMS_PATH,
};
use zescrow_core::{EscrowMetadata, EscrowParams};

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    info!("Starting command handling");

    run(cli.command).await
}

#[instrument(skip_all, fields(command = ?command))]
async fn run(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Create => {
            info!("Loading escrow parameters from {}", ESCROW_PARAMS_PATH);
            let params: EscrowParams = load_escrow_data(ESCROW_PARAMS_PATH)?;
            debug!("EscrowParams: {:#?}", params);

            info!("Building ZescrowClient");
            let client = ZescrowClient::builder(*params.asset.chain(), params.chain_config.clone())
                .build()
                .await?;
            info!("Creating escrow on-chain");
            let metadata = client.create_escrow(&params).await?;
            info!("Escrow created!");
            debug!("EscrowMetadata: {:#?}", metadata);

            info!("Saving metadata to {}", ESCROW_METADATA_PATH);
            save_escrow_data(ESCROW_METADATA_PATH, &metadata)?;
        }

        Commands::Finish { recipient } => {
            info!("Loading escrow metadata from {}", ESCROW_METADATA_PATH);
            let metadata: EscrowMetadata = load_escrow_data(ESCROW_METADATA_PATH)?;
            debug!("EscrowMetadata: {:#?}", metadata);

            info!("Building ZescrowClient for `finish`");
            let client = ZescrowClient::builder(
                metadata.chain_config.chain_id(),
                metadata.chain_config.clone(),
            )
            .recipient(recipient)
            .build()
            .await?;

            info!("Finishing escrow");
            client.finish_escrow(&metadata).await?;
            info!("Escrow completed and released successfully");
        }

        Commands::Cancel => {
            info!("Loading escrow metadata from {}", ESCROW_METADATA_PATH);
            let metadata: EscrowMetadata = load_escrow_data(ESCROW_METADATA_PATH)?;
            debug!("EscrowMetadata: {:#?}", metadata);

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
    }
    Ok(())
}
