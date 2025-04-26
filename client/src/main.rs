use std::path::PathBuf;
use std::str::FromStr;

use clap::{Parser, Subcommand};
use zescrow_client::interface::{
    load_chain_config, load_escrow_input_data, save_escrow_metadata, Chain, ChainConfig,
    EscrowMetadata, EscrowParams,
};
use zescrow_client::ZescrowClient;

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
            let chain = Chain::from_str(&chain)?;
            let config = load_escrow_input_data::<ChainConfig>(&config)?;
            let params = load_escrow_input_data::<EscrowParams>(&params)?;

            let client = ZescrowClient::new(chain, config)?;
            let metadata = client.create_escrow(&params).await?;

            save_escrow_metadata(&outfile, &metadata)?;
            tracing::info!("Escrow created successfully");
        }
        Commands::Release { metadata } => {
            let metadata = load_escrow_input_data::<EscrowMetadata>(&metadata)?;
            let config = load_chain_config(&metadata)?;

            let client = ZescrowClient::new(metadata.chain, config)?;
            client.release_escrow(&metadata).await?;
            tracing::info!("Escrow released successfully");
        }
        Commands::Refund { metadata } => {
            let metadata = load_escrow_input_data::<EscrowMetadata>(&metadata)?;
            let config = load_chain_config(&metadata)?;

            let client = ZescrowClient::new(metadata.chain, config)?;
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
        #[arg(short, long)]
        chain: String,
        #[arg(short, long)]
        config: PathBuf,
        #[arg(short, long)]
        params: PathBuf,
        #[arg(short, long)]
        outfile: PathBuf,
    },
    Release {
        #[arg(short, long)]
        metadata: PathBuf,
    },
    Refund {
        #[arg(short, long)]
        metadata: PathBuf,
    },
}
