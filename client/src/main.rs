use clap::{Parser, Subcommand};
use zescrow_client::utils::{load_chain_config, load_escrow_input_data, save_escrow_metadata};
use zescrow_client::ZescrowClient;
use zescrow_core::{EscrowMetadata, EscrowParams};

const ESCROW_PARAMS_PATH: &str = "templates/escrow_params.json";
const ESCROW_METADATA_PATH: &str = "templates/escrow_metadata.json";

#[derive(Parser)]
#[command(name = "zescrow-cli")]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create an escrow using the templates in `templates/`
    Create,

    /// Complete/release an existing escrow to the beneficiary.
    /// Reads `templates/escrow_metadata.json`
    Finish,

    /// Cancel/refund an existing escrow to the depositor.
    /// Reads `templates/escrow_metadata.json`
    Cancel,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create => {
            let params: EscrowParams = load_escrow_input_data(ESCROW_PARAMS_PATH)?;
            let config = load_chain_config(params.chain)?;

            let client = ZescrowClient::new(params.chain, config)?;
            let metadata = client.create_escrow(&params).await?;

            save_escrow_metadata(ESCROW_METADATA_PATH, &metadata)?;
            tracing::info!(
                "Escrow created successfully; metadata written to `{}`",
                ESCROW_METADATA_PATH
            );
        }
        Commands::Finish => {
            let metadata: EscrowMetadata = load_escrow_input_data(ESCROW_METADATA_PATH)?;
            let config = load_chain_config(metadata.chain)?;

            let client = ZescrowClient::new(metadata.chain, config)?;
            client.finish_escrow(&metadata).await?;
            tracing::info!("Escrow completed and released successfully");
        }
        Commands::Cancel => {
            let metadata: EscrowMetadata = load_escrow_input_data(ESCROW_METADATA_PATH)?;
            let config = load_chain_config(metadata.chain)?;

            let client = ZescrowClient::new(metadata.chain, config)?;
            client.cancel_escrow(&metadata).await?;
            tracing::info!("Escrow cancelled and refunded successfully");
        }
    }
    Ok(())
}
