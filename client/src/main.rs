use clap::{Parser, Subcommand};
use zescrow_client::interface::{
    load_escrow_input_data, save_escrow_metadata, Chain, ChainConfig, EscrowMetadata, EscrowParams,
};
use zescrow_client::ZescrowClient;

const TEMPLATES_DIR: &str = "templates";
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

    /// Release an existing escrow to the beneficiary.
    /// Reads `templates/escrow_metadata.json`
    Release,

    /// Refund an existing escrow to the depositor.
    /// Reads `templates/escrow_metadata.json`
    Refund,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create => {
            let params: EscrowParams = load_escrow_input_data(ESCROW_PARAMS_PATH)?;
            let config = get_chain_config(params.chain)?;

            let client = ZescrowClient::new(params.chain, config)?;
            let metadata = client.create_escrow(&params).await?;

            save_escrow_metadata(ESCROW_METADATA_PATH, &metadata)?;
            tracing::info!(
                "Escrow created successfully; metadata written to `{}`",
                ESCROW_METADATA_PATH
            );
        }
        Commands::Release => {
            let metadata: EscrowMetadata = load_escrow_input_data(ESCROW_METADATA_PATH)?;
            let config = get_chain_config(metadata.chain)?;

            let client = ZescrowClient::new(metadata.chain, config)?;
            client.release_escrow(&metadata).await?;
            tracing::info!("Escrow released successfully");
        }
        Commands::Refund => {
            let metadata: EscrowMetadata = load_escrow_input_data(ESCROW_METADATA_PATH)?;
            let config = get_chain_config(metadata.chain)?;

            let client = ZescrowClient::new(metadata.chain, config)?;
            client.refund_escrow(&metadata).await?;
            tracing::info!("Escrow refunded successfully");
        }
    }
    Ok(())
}

fn get_chain_config(chain: Chain) -> anyhow::Result<ChainConfig> {
    let config_path = format!("{}/{}_config.json", TEMPLATES_DIR, chain.as_ref());
    load_escrow_input_data(&config_path)
}
