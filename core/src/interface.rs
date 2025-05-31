//! Core types for JSON (de)serialization of escrow parameters and metadata.

use std::fs::File;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{Asset, EscrowError, Party, Result};

/// Default path to escrow params template.
pub const ESCROW_PARAMS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../templates/escrow_params.json"
);

/// Default path to on-chain escrow metadata.
pub const ESCROW_METADATA_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../templates/escrow_metadata.json"
);

/// Default path to escrow conditions template.
pub const ESCROW_CONDITIONS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../templates/escrow_conditions.json"
);

/// Reads a JSON-encoded file from the given `path` and deserializes into type `T`.
///
/// # Errors
///
/// Returns an `anyhow::Error` if the file cannot be opened, read, or parsed.
///
/// # Examples
///
/// ```ignore
/// # use zescrow_core::interface::load_escrow_data;
///
/// #[derive(Deserialize)]
/// struct MyParams { /* fields matching JSON */ }
///
/// let _params: MyParams = load_escrow_data(./my_params.json).unwrap();
/// ```
pub fn load_escrow_data<P, T>(path: P) -> anyhow::Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("loading escrow data: {:?}", path))?;
    serde_json::from_str(&content).with_context(|| format!("parsing JSON from {:?}", path))
}

/// Writes `data` (serializable) as pretty-printed JSON to the given `path`.
///
/// # Errors
///
/// Returns an `anyhow::Error` if the file cannot be created or data cannot be serialized.
///
/// # Examples
///
/// ```ignore
/// # use zescrow_core::interface::save_escrow_data;
/// # use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct MyMetadata { /* fields */ }
///
/// let metadata = MyMetadata { /* ... */ };
/// save_escrow_data("./metadata.json", &metadata).unwrap();
/// ```
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

/// Lifecycle of an escrow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EscrowState {
    /// Funds have been deposited; awaiting release or cancellation.
    Funded,
    /// Conditions (if any) met; funds have been released to the recipient.
    Released,
}

/// Parameters required to **create** an escrow on-chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EscrowParams {
    /// Chain-specific network configuration.
    #[serde(flatten)]
    pub chain_config: ChainConfig,

    /// Exactly which asset to lock (native, token, NFT, pool-share, etc).
    #[serde(flatten)]
    pub asset: Asset,

    /// Who is funding (locking) the escrow.
    pub sender: Party,

    /// Who will receive the funds once conditions pass.
    pub recipient: Party,

    /// Optional block height or slot after which `execute` is allowed.
    /// Must be `None` or less than `cancel_after` if both are set.
    pub finish_after: Option<u64>,

    /// Optional block height or slot after which `cancel` is allowed.
    /// Must be `None` or greater than `finish_after` if both are set.
    pub cancel_after: Option<u64>,

    /// Whether this escrow is subject to cryptographic conditions.
    pub has_conditions: bool,
}

/// Metadata **returned** from on-chain escrow creation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EscrowMetadata {
    /// Chain-specific network configuration.
    #[serde(flatten)]
    pub chain_config: ChainConfig,

    /// Exactly which asset got locked.
    #[serde(flatten)]
    pub asset: Asset,

    /// The party who funded the escrow.
    pub sender: Party,

    /// The party who will receive the funds.
    pub recipient: Party,

    /// Denotes whether this escrow is subject to cryptographic conditions.
    pub has_conditions: bool,

    /// Chain-specific on-chain accounts/programs
    /// used to finish or cancel the escrow.
    #[serde(flatten)]
    pub chain_data: ChainMetadata,

    /// Where in the lifecycle the escrow currently is.
    pub state: EscrowState,
}

/// Chain-specific on-chain escrow metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChainMetadata {
    /// Data relating to an escrow on Etherum (and/or any EVM chain).
    Ethereum {
        /// The escrow smart-contract address.
        escrow_address: String,
    },

    /// Data relating to an escrow on Solana.
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
    /// Get the PDA for a Solana escrow.
    ///
    /// # Errors
    ///
    /// Returns an `EscrowError::InvalidChainOp` error if called on a non-Solana variant.
    pub fn get_pda(&self) -> Result<String> {
        match self {
            Self::Solana { pda, .. } => Ok(pda.clone()),
            _ => Err(EscrowError::InvalidChainOp(
                "PDA computation not applicable".to_string(),
            )),
        }
    }

    /// Get the Ethereum contract address for the escrow contract.
    ///
    /// # Errors
    ///
    /// Returns an `EscrowError::InvalidChainOp` error if called on a non-Ethereum variant.
    pub fn get_eth_contract_address(&self) -> Result<String> {
        match self {
            Self::Ethereum { escrow_address, .. } => Ok(escrow_address.clone()),
            _ => Err(EscrowError::InvalidChainOp(
                "Ethereum contract address applicable".to_string(),
            )),
        }
    }
}

/// Chain-specific network configuration for creating or querying escrows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "network", content = "chain_config", rename_all = "snake_case")]
pub enum ChainConfig {
    /// Ethereum network configuration.
    Ethereum {
        /// JSON-RPC endpoint URL.
        rpc_url: String,
        /// Sender's private key in wallet import format (WIF) or hex.
        sender_private_key: String,
        /// Address of the `EscrowFactory` smart contract.
        escrow_factory_address: String,
        /// On-chain ZK verifier contract address.
        verifier_address: String,
    },

    /// Solana network configuration.
    Solana {
        /// JSON-RPC endpoint URL.
        rpc_url: String,
        /// Path to payer keypair file (e.g., `~/.config/solana/id.json`).
        sender_keypair_path: String,
        /// On-chain escrow program ID (base58 string).
        escrow_program_id: String,
        /// On-chain ZK verifier program ID (base58 string).
        verifier_program_id: String,
    },
}

impl ChainConfig {
    /// Returns the `Chain` enum corresponding to this variant.
    pub fn chain_id(&self) -> Chain {
        match self {
            Self::Ethereum { .. } => Chain::Ethereum,
            Self::Solana { .. } => Chain::Solana,
        }
    }

    /// Get the Ethereum `EscrowFactory` contract address.
    ///
    /// # Errors
    ///
    /// Returns an `EscrowError::InvalidChainOp` error if called on a non-Ethereum variant.
    pub fn eth_escrow_factory_contract(&self) -> Result<String> {
        match self {
            Self::Ethereum {
                escrow_factory_address,
                ..
            } => Ok(escrow_factory_address.clone()),
            _ => Err(EscrowError::InvalidChainOp(
                "Ethereum escrow factory address not applicable".to_string(),
            )),
        }
    }

    /// Get the Ethereum ZK verifier contract address.
    ///
    /// # Errors
    ///
    /// Returns an `EscrowError::InvalidChainOp` error if called on a non-Ethereum variant.
    pub fn eth_verifier_contract(&self) -> Result<String> {
        match self {
            Self::Ethereum {
                verifier_address, ..
            } => Ok(verifier_address.clone()),
            _ => Err(EscrowError::InvalidChainOp(
                "Ethereum verifier contract not applicable".to_string(),
            )),
        }
    }

    /// Get the Solana escrow program ID.
    ///
    /// # Errors
    ///
    /// Returns an `EscrowError::InvalidChainOp` error if called on a non-Solana variant.
    pub fn sol_escrow_program(&self) -> Result<String> {
        match self {
            Self::Solana {
                escrow_program_id, ..
            } => Ok(escrow_program_id.clone()),
            _ => Err(EscrowError::InvalidChainOp(
                "Solana escrow program ID not applicable".to_string(),
            )),
        }
    }

    /// Get the Solana ZK verifier program ID.
    ///
    /// # Errors
    ///
    /// Returns an `EscrowError::InvalidChainOp` error if called on a non-Solana variant.
    pub fn sol_verifier_program(&self) -> Result<String> {
        match self {
            Self::Solana {
                verifier_program_id,
                ..
            } => Ok(verifier_program_id.clone()),
            _ => Err(EscrowError::InvalidChainOp(
                "Solana verifier program not applicable".to_string(),
            )),
        }
    }

    /// Get the Solana payer keypair file path.
    ///
    /// # Errors
    ///
    /// Returns an `EscrowError::InvalidChainOp` error if called on a non-Solana variant.
    pub fn sol_sender_keypair_path(&self) -> Result<String> {
        match self {
            Self::Solana {
                sender_keypair_path,
                ..
            } => Ok(sender_keypair_path.clone()),
            _ => Err(EscrowError::InvalidChainOp(
                "Solana sender keypair path not applicable".to_string(),
            )),
        }
    }
}

/// Supported blockchain networks.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Chain {
    /// Ethereum and other EVM-compatible chains.
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

    /// Parses a string into a `Chain` enum (case-insensitive).
    ///
    /// Acceptable values:
    /// - "ethereum" or "eth" => `Chain::Ethereum`
    /// - "solana" or "sol" => `Chain::Solana`
    ///
    /// # Errors
    ///
    /// Returns `EscrowError::UnsupportedChain` on unrecognized input.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Ok(Self::Ethereum),
            "solana" | "sol" => Ok(Self::Solana),
            _ => Err(EscrowError::UnsupportedChain),
        }
    }
}
