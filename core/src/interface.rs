//! Core types for JSON (de)serialization of escrow parameters and metadata.
//!
//! These types power two files at runtime:
//! - `escrow_params.json`: input to the CLI `client` for creating an escrow.
//! - `escrow_metadata.json`: output from the CLI after on-chain creation.

use serde::{Deserialize, Serialize};

use crate::{Asset, Condition, Escrow, EscrowError, Party, Result};

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

    /// Optional cryptographic condition (hashlock, signature, threshold).
    #[serde(default, flatten)]
    pub condition: Option<Condition>,

    /// Optional UNIX timestamp (seconds since epoch) after which `execute` is allowed.
    pub finish_after: Option<i64>,

    /// Optional UNIX timestamp (seconds since epoch) after which `cancel` is allowed.
    pub cancel_after: Option<i64>,
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

    /// The specified cryptographic condition (if any).
    #[serde(default, flatten)]
    pub condition: Option<Condition>,

    /// Chain-specific accounts/programs to finish or cancel with.
    #[serde(flatten)]
    pub chain_data: ChainMetadata,

    /// Where in the lifecycle an escrow is.
    pub state: EscrowState,
}

impl EscrowMetadata {
    pub fn to_escrow(self) -> Escrow {
        let Self {
            asset,
            sender,
            recipient,
            condition,
            state,
            ..
        } = self;
        Escrow {
            asset,
            recipient,
            sender,
            condition,
            state,
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
