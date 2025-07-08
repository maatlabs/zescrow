#[cfg(feature = "json")]
use std::fs::File;
#[cfg(feature = "json")]
use std::path::Path;

#[cfg(feature = "json")]
use anyhow::Context;
use bincode::{Decode, Encode};
#[cfg(feature = "json")]
use serde::de::DeserializeOwned;
#[cfg(feature = "json")]
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

/// Default path to proof data.
pub const PROOF_DATA_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../templates/proof_data.json");

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
#[cfg(feature = "json")]
pub fn load_escrow_data<P, T>(path: P) -> anyhow::Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let content =
        std::fs::read_to_string(path).with_context(|| format!("loading escrow data: {path:?}"))?;
    serde_json::from_str(&content).with_context(|| format!("parsing JSON from {path:?}"))
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
#[cfg(feature = "json")]
pub fn save_escrow_data<P, T>(path: P, data: &T) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let path = path.as_ref();
    let file = File::create(path).with_context(|| format!("creating file {path:?}"))?;
    serde_json::to_writer_pretty(file, data)
        .with_context(|| format!("serializing to JSON to {path:?}"))
}

/// State of escrow execution in the `client`.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq)]
pub enum ExecutionState {
    /// Escrow object created.
    Initialized,

    /// Funds have been deposited; awaiting release or cancellation.
    Funded,

    /// Conditions (if any) have been fulfilled;
    /// funds will be released to the recipient if the proof verifies on-chain.
    ConditionsMet,
}

/// Result of escrow execution in the `client`.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode)]
pub enum ExecutionResult {
    /// Happy path; no errors in execution.
    Ok(ExecutionState),
    /// Unsuccessful escrow execution, with the error message.
    Err(String),
}

/// Parameters required to **create** an escrow on-chain.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode)]
pub struct EscrowParams {
    /// Chain-specific network configuration.
    pub chain_config: ChainConfig,

    /// Exactly which asset to lock (native, token, NFT, pool-share, etc).
    pub asset: Asset,

    /// Who is funding (locking) the escrow.
    pub sender: Party,

    /// Who will receive the funds once conditions pass.
    pub recipient: Party,

    /// Optional block height or slot after which "release" is allowed.
    /// Must be `None` or less than `cancel_after` if both are set.
    pub finish_after: Option<u64>,

    /// Optional block height or slot after which "cancel" is allowed.
    /// Must be `None` or greater than `finish_after` if both are set.
    pub cancel_after: Option<u64>,

    /// Denotes whether this escrow is subject to cryptographic conditions.
    pub has_conditions: bool,
}

/// Metadata **returned** from on-chain escrow creation.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode)]
pub struct EscrowMetadata {
    /// Chain-specific network configuration.
    pub chain_config: ChainConfig,

    /// Exactly which asset got locked.
    pub asset: Asset,

    /// The party who funded the escrow.
    pub sender: Party,

    /// The party who will receive the funds.
    pub recipient: Party,

    /// Denotes whether this escrow is subject to cryptographic conditions.
    pub has_conditions: bool,

    /// Escrow program ID or contract address.
    pub agent_id: String,

    /// State of escrow execution in the `client`.
    pub state: ExecutionState,
}

/// Chain-specific network configuration for creating or querying escrows.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(tag = "chain", rename_all = "lowercase"))]
#[derive(Debug, Clone, Encode, Decode)]
pub enum ChainConfig {
    /// Ethereum network configuration.
    Ethereum {
        /// JSON-RPC endpoint URL.
        rpc_url: String,
        /// Sender's private key in wallet import format (WIF) or hex.
        sender_private_key: String,
        /// Address of the `EscrowFactory` smart contract.
        escrow_factory_address: String,
    },

    /// Solana network configuration.
    Solana {
        /// JSON-RPC endpoint URL.
        rpc_url: String,
        /// Path to payer keypair file (e.g., `~/.config/solana/id.json`).
        sender_keypair_path: String,
        /// On-chain escrow program ID (base58 string).
        escrow_program_id: String,
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
    /// Returns an `EscrowError::InvalidChainOp` if called on a non-Ethereum variant.
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
}

/// Supported blockchain networks.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(rename_all = "lowercase"))]
#[derive(Debug, Copy, Clone, Encode, Decode)]
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
