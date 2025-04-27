use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub fn load_escrow_input_data<P, T>(path: P) -> anyhow::Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            anyhow::bail!(
                "Input file {:?} not found. 
                You can create one with `zescrow-cli init` or pass
                --config/--params explicitly.",
                path
            );
        }
        Err(e) => return Err(e).context(format!("opening file {:?}", path)),
    };
    serde_json::from_reader(file).with_context(|| format!("parsing JSON from {:?}", path))
}

pub fn save_escrow_metadata<P, T>(path: P, data: &T) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {:?}", parent))?;
    }

    let file = File::create(path).with_context(|| format!("creating file {:?}", path))?;
    serde_json::to_writer_pretty(file, data)
        .with_context(|| format!("serializing to JSON to {:?}", path))
}

/// Escrow input parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowParams {
    /// Target blockchain network
    pub chain: Chain,
    /// Depositor's blockchain address
    pub depositor: String,
    /// Beneficiary's blockchain address
    pub beneficiary: String,
    /// Escrow amount in native token units
    pub amount: u64,
    /// Expiration block/slot number
    pub expiry: u64,
}

/// Result of escrow creation, release, or refund.
#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowMetadata {
    /// Original blockchain network
    pub chain: Chain,
    /// Depositor's address
    pub depositor: String,
    /// Beneficiary's address
    pub beneficiary: String,
    /// Locked amount
    pub amount: u64,
    /// Expiration block/slot
    pub expiry: u64,
    /// Escrow transaction ID
    pub tx_id: String,
    /// The full config used to create the escrow
    pub config: ChainConfig,
    /// Chain-specific metadata
    /// returned from escrow creation
    #[serde(flatten)]
    pub chain_data: ChainMetadata,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Ethereum,
    Solana,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChainMetadata {
    Ethereum {
        contract_address: String,
        block_number: u64,
    },
    Solana {
        program_id: String,
        pda: String,
        bump: u8,
    },
}

/// Chain-specific network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChainConfig {
    /// Ethereum network configuration
    Ethereum {
        /// JSON-RPC endpoint URL
        rpc_url: String,
        /// Private key
        /// in wallet import format (WIF)
        private_key: String,
        /// Escrow smart contract address
        contract_address: String,
    },
    /// Solana network configuration
    Solana {
        /// JSON-RPC endpoint URL
        rpc_url: String,
        /// Path to payer keypair file
        keypair_path: String,
        /// Program ID for escrow program
        program_id: String,
    },
}
