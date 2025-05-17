//! Core types for JSON (de)serialization of escrow parameters and metadata.

use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{Asset, EscrowError, Party, Result};

const TEMPLATES_DIR: &str = "templates";
pub const ESCROW_PARAMS_PATH: &str = "templates/escrow_params.json";
pub const ESCROW_METADATA_PATH: &str = "templates/escrow_metadata.json";
pub const ESCROW_CONDITIONS_PATH: &str = "templates/escrow_conditions.json";

/// Reads chain-specific configuration given a target `chain`
/// (e.g., ethereum, solana).
pub fn load_chain_config(chain: &Chain) -> anyhow::Result<ChainConfig> {
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
                "Chain config file {:?} not found.
                Please create a <CHAIN>_config.json in /templates",
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

/// Where in the lifecycle an escrow is.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum EscrowState {
    Funded,
    Released,
    Expired,
    Canceled,
}

/// Parameters for **creating** an escrow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowParams {
    /// Exactly which asset to lock (native, token, NFT, pool-share, etc).
    #[serde(flatten)]
    pub asset: Asset,

    /// Who’s funding the escrow.
    pub sender: Party,

    /// Who will receive the funds once conditions pass.
    pub recipient: Party,

    /// Optional UNIX timestamp (seconds since epoch) after which `execute` is allowed.
    pub finish_after: Option<i64>,

    /// Optional UNIX timestamp (seconds since epoch) after which `cancel` is allowed.
    pub cancel_after: Option<i64>,

    /// Specify whether this escrow is subject to any cryptographic conditions.
    pub has_conditions: bool,
}

/// Metadata **returned** from on-chain escrow creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowMetadata {
    /// Exactly which asset got locked.
    #[serde(flatten)]
    pub asset: Asset,

    /// The funding party.
    pub sender: Party,

    /// The beneficiary party.
    pub recipient: Party,

    /// Denotes whether this escrow is subject to any cryptographic conditions.
    pub has_conditions: bool,

    /// Chain-specific accounts/programs to finish or cancel with.
    #[serde(flatten)]
    pub chain_data: ChainMetadata,

    /// Where in the lifecycle an escrow is.
    pub state: EscrowState,
}

/// Supported blockchain networks.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    // Ethereum and other EVM-compatible chains
    Ethereum,
    /// Solana
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

impl std::str::FromStr for Chain {
    type Err = EscrowError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Ok(Self::Ethereum),
            "solana" | "sol" => Ok(Self::Solana),
            _ => Err(EscrowError::UnsupportedChain),
        }
    }
}

/// Chain-specific on-chain escrow metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChainMetadata {
    Ethereum {
        /// The escrow smart-contract address.
        contract_address: String,
    },

    Solana {
        /// Escrow program’s ID.
        program_id: String,
        /// The program-derived address for this escrow account.
        pda: String,
        /// The bump seed used to derive the PDA.
        bump: u8,
    },
}

impl ChainMetadata {
    /// Get PDA for Solana escrows.
    pub fn get_pda(&self) -> Result<String> {
        match self {
            Self::Solana { pda, .. } => Ok(pda.to_string()),
            _ => Err(EscrowError::InvalidChainOp(
                "PDA computation not applicable".to_string(),
            )),
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
