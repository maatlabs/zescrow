use std::path::PathBuf;

use clap::{Parser, Subcommand};
use zescrow_client::ZescrowClient;
use zescrow_core::interface::{
    load_escrow_data, save_escrow_data, ESCROW_METADATA_PATH, ESCROW_PARAMS_PATH,
};
use zescrow_core::{EscrowMetadata, EscrowParams};

#[derive(Parser)]
#[command(name = "zescrow-cli")]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create an escrow using the specified parameters in
    /// `templates/escrow_params.json`.
    /// Generates `templates/escrow_metadata.json` on success.
    Create,
    /// Complete/release an existing escrow to the beneficiary.
    /// Reads `templates/escrow_metadata.json`.
    Finish {
        /// Path to the escrow recipient's keypair for signing.
        #[arg(long)]
        recipient_keypair_path: PathBuf,
    },
    /// Cancel/refund an existing escrow to the creator.
    /// Reads `templates/escrow_metadata.json`.
    Cancel,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create => {
            let params: EscrowParams = load_escrow_data(ESCROW_PARAMS_PATH)?;
            let client = ZescrowClient::new(params.asset.chain(), &params.chain_config, None)?;
            let metadata = client.create_escrow(&params).await?;

            save_escrow_data(ESCROW_METADATA_PATH, &metadata)?;
            tracing::info!(
                "Escrow created successfully; metadata written to `{}`",
                ESCROW_METADATA_PATH
            );
        }
        Commands::Finish {
            recipient_keypair_path,
        } => {
            let metadata: EscrowMetadata = load_escrow_data(ESCROW_METADATA_PATH)?;
            let client = ZescrowClient::new(
                &metadata.chain_config.chain_id(),
                &metadata.chain_config,
                Some(recipient_keypair_path),
            )?;
            client.finish_escrow(&metadata).await?;
            tracing::info!("Escrow completed and released successfully");
        }
        Commands::Cancel => {
            let metadata: EscrowMetadata = load_escrow_data(ESCROW_METADATA_PATH)?;
            let client = ZescrowClient::new(
                &metadata.chain_config.chain_id(),
                &metadata.chain_config,
                None,
            )?;
            client.cancel_escrow(&metadata).await?;
            tracing::info!("Escrow cancelled and refunded successfully");
        }
    }
    Ok(())
}
