//! Core types for JSON (de)serialization of escrow parameters and metadata.

use std::fs::File;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{Asset, EscrowError, Party, Result};

pub const ESCROW_PARAMS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../templates/escrow_params.json"
);
pub const ESCROW_METADATA_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../templates/escrow_metadata.json"
);
pub const ESCROW_CONDITIONS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../templates/escrow_conditions.json"
);

/// Reads JSON-encoded escrow params, metadata, and
/// chain-specific configs from the given `path`.
pub fn load_escrow_data<P, T>(path: P) -> anyhow::Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let file = std::fs::read_to_string(path)
        .with_context(|| format!("loading escrow data: {:?}", path))?;
    serde_json::from_str(&file).with_context(|| format!("parsing JSON from {:?}", path))
}

/// Writes JSON-encoded `data` to the given `path`.
pub fn save_escrow_data<P, T>(path: P, data: &T) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let path = path.as_ref();
    let file = File::create(path).with_context(|| format!("creating file {:?}", path))?;
    serde_json::to_writer_pretty(file, data)
        .with_context(|| format!("serializing to JSON to {:?}", path))
}

/// Where in the lifecycle an escrow is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EscrowState {
    Funded,
    Released,
}

/// Parameters for **creating** an escrow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowParams {
    /// Chain-specific configuration.
    #[serde(flatten)]
    pub chain_config: ChainConfig,
    /// Exactly which asset to lock (native, token, NFT, pool-share, etc).
    #[serde(flatten)]
    pub asset: Asset,
    /// Who’s funding the escrow.
    pub sender: Party,
    /// Who will receive the funds once conditions pass.
    pub recipient: Party,
    /// Optional block height or slot after which `execute` is allowed.
    pub finish_after: Option<u64>,
    /// Optional block height or slot after which `cancel` is allowed.
    pub cancel_after: Option<u64>,
    /// Specify whether this escrow is subject to any cryptographic conditions.
    pub has_conditions: bool,
}

/// Metadata **returned** from on-chain escrow creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowMetadata {
    /// Chain-specific configuration.
    #[serde(flatten)]
    pub chain_config: ChainConfig,
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

/// Chain-specific on-chain escrow metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChainMetadata {
    Ethereum {
        /// The escrow smart-contract address.
        escrow_address: String,
    },
    Solana {
        /// Escrow program’s ID.
        escrow_program_id: String,
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

    /// Get address of deployed escrow contract.
    pub fn get_eth_contract_address(&self) -> Result<String> {
        match self {
            Self::Ethereum { escrow_address, .. } => Ok(escrow_address.to_string()),
            _ => Err(EscrowError::InvalidChainOp("Not applicable".to_string())),
        }
    }
}

/// Chain-specific network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "network", content = "chain_config", rename_all = "snake_case")]
pub enum ChainConfig {
    /// Ethereum network configuration
    Ethereum {
        /// JSON-RPC endpoint URL
        rpc_url: String,
        /// Sender's private key in wallet import format (WIF)
        sender_private_key: String,
        /// Address of the `EscrowFactory` smart contract
        escrow_factory_address: String,
        /// On-chain ZK verifier contract address
        verifier_address: String,
    },
    /// Solana network configuration
    Solana {
        /// JSON-RPC endpoint URL
        rpc_url: String,
        /// Path to payer keypair file
        sender_keypair_path: String,
        /// On-chain escrow program ID
        escrow_program_id: String,
        /// On-chain ZK verifier program ID
        verifier_program_id: String,
    },
}

impl ChainConfig {
    pub fn chain_id(&self) -> Chain {
        match self {
            Self::Ethereum { .. } => Chain::Ethereum,
            Self::Solana { .. } => Chain::Solana,
        }
    }

    pub fn eth_escrow_factory_contract(&self) -> Result<String> {
        match self {
            Self::Ethereum {
                escrow_factory_address,
                ..
            } => Ok(escrow_factory_address.clone()),
            _ => Err(EscrowError::InvalidChainOp("Not applicable".to_string())),
        }
    }

    pub fn eth_verifier_contract(&self) -> Result<String> {
        match self {
            Self::Ethereum {
                verifier_address, ..
            } => Ok(verifier_address.clone()),
            _ => Err(EscrowError::InvalidChainOp("Not applicable".to_string())),
        }
    }

    pub fn sol_verifier_program(&self) -> Result<String> {
        match self {
            Self::Solana {
                verifier_program_id,
                ..
            } => Ok(verifier_program_id.clone()),
            _ => Err(EscrowError::InvalidChainOp("Not applicable".to_string())),
        }
    }
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
