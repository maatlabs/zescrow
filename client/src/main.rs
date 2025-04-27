use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};
use zescrow_client::interface::{
    load_escrow_input_data, save_escrow_metadata, Chain, ChainConfig, EscrowMetadata, EscrowParams,
};
use zescrow_client::ZescrowClient;

const DEFAULT_CHAIN_CONFIG_PATH: &str = "./chain_config.json";
const DEFAULT_ESCROW_PARAMS_PATH: &str = "./escrow_params.json";
const DEFAULT_ESCROW_METADATA_PATH: &str = "./escrow_metadata.json";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            chain,
            config,
            params,
            outfile,
        } => {
            let config: ChainConfig = load_escrow_input_data(&config)?;
            let params: EscrowParams = load_escrow_input_data(&params)?;

            let client = ZescrowClient::new(chain, config.clone())?;
            let mut metadata = client.create_escrow(&params).await?;
            // For reuse later during `Release` or `Refund`
            metadata.config = config;

            save_escrow_metadata(&outfile, &metadata)?;
            tracing::info!("Escrow created successfully");
        }
        Commands::Release { metadata } => {
            let metadata: EscrowMetadata = load_escrow_input_data(&metadata)?;

            let client = ZescrowClient::new(metadata.chain, metadata.config.clone())?;
            client.release_escrow(&metadata).await?;
            tracing::info!("Escrow released successfully");
        }
        Commands::Refund { metadata } => {
            let metadata: EscrowMetadata = load_escrow_input_data(&metadata)?;

            let client = ZescrowClient::new(metadata.chain, metadata.config.clone())?;
            client.refund_escrow(&metadata).await?;
            tracing::info!("Escrow refunded successfully");
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(name = "zescrow-cli")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create {
        #[arg(short, long, value_enum)]
        chain: Chain,

        #[arg(short, long,
            value_parser,
            default_value = DEFAULT_CHAIN_CONFIG_PATH,
            value_hint = ValueHint::FilePath)]
        config: PathBuf,

        #[arg(short, long,
            value_parser,
            default_value = DEFAULT_ESCROW_PARAMS_PATH,
            value_hint = ValueHint::FilePath)]
        params: PathBuf,

        #[arg(short, long,
            value_parser,
            default_value = DEFAULT_ESCROW_METADATA_PATH,
            value_hint = ValueHint::FilePath)]
        outfile: PathBuf,
    },
    Release {
        #[arg(short, long,
            value_parser,
            default_value = DEFAULT_ESCROW_METADATA_PATH,
            value_hint = ValueHint::FilePath)]
        metadata: PathBuf,
    },
    Refund {
        #[arg(short, long,
            value_parser,
            default_value = DEFAULT_ESCROW_METADATA_PATH,
            value_hint = ValueHint::FilePath)]
        metadata: PathBuf,
    },
}
