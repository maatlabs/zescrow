use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::{ClientError, Result};

const TEMPLATES_DIR: &str = "templates";

/// Reads chain-specific configuration given a target `chain`
/// (e.g., ethereum, solana).
pub fn load_chain_config(chain: Chain) -> anyhow::Result<ChainConfig> {
    let config_path = format!("{}/{}_config.json", TEMPLATES_DIR, chain.as_ref());
    load_escrow_input_data(&config_path)
}

/// Reads JSON-encoded escrow params and chain-specific configs
/// from the given `path`.
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
                Please run `zescrow-cli init --chain <chain>`
                or create it manually.",
                path
            );
        }
        Err(e) => return Err(e).context(format!("opening file {:?}", path)),
    };
    serde_json::from_reader(file).with_context(|| format!("parsing JSON from {:?}", path))
}

/// Writes JSON-encoded `data` to the given `path`,
/// creating parent directories as needed.
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

/// Target blockchains
#[derive(Debug, Copy, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    // Ethereum and other EVM-compatible chains
    Ethereum,
    Solana,
}

impl AsRef<str> for Chain {
    fn as_ref(&self) -> &str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Solana => "solana",
        }
    }
}

/// Chain-specific metadata for smart contracts/programs
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

impl ChainMetadata {
    /// Get PDA for Solana escrows
    pub fn get_pda(&self) -> Result<&str> {
        match self {
            ChainMetadata::Solana { pda, .. } => Ok(pda),
            _ => Err(ClientError::InvalidChainOperation),
        }
    }
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

/// Parameters for creating an escrow
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

/// Metadata returned from escrow creation and
/// used for release/refund commands
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
    /// Chain-specific metadata for smart contracts/programs
    #[serde(flatten)]
    pub chain_data: ChainMetadata,
}
