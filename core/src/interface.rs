use core::str::FromStr;

use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::EscrowError;
use crate::{Asset, Condition, Escrow, EscrowState, Party, Result};

#[cfg(test)]
pub fn assert_err<T, E>(res: Result<T>, expected: E)
where
    E: std::fmt::Debug + PartialEq<E>,
    EscrowError: Into<E> + PartialEq<E>,
{
    match res {
        Err(e) => assert_eq!(e.into(), expected),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

// TODO: create a proper `AssetId` with methods
pub fn to_escrow(data: EscrowMetadata) -> Result<Escrow> {
    // derive asset ID from chain_data for now:
    let asset_id = match &data.chain_data {
        ChainMetadata::Ethereum {
            contract_address, ..
        } => contract_address.clone(),
        ChainMetadata::Solana { program_id, .. } => program_id.clone(),
    };
    let sender = Party::from_str(&data.sender)?;
    let recipient = Party::from_str(&data.recipient)?;

    Ok(Escrow {
        asset: Asset::Fungible {
            id: asset_id,
            amount: data.amount,
        },
        recipient,
        sender,
        condition: data.condition,
        created_block: data.created_block,
        state: EscrowState::Released,
    })
}

/// Metadata returned from escrow creation
#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowMetadata {
    /// Original target blockchain network
    pub chain: Chain,
    /// Escrow creator's blockchain address
    pub sender: String,
    /// Escrow beneficiary's blockchain address
    pub recipient: String,
    /// Locked amount in native token units
    pub amount: u64,
    /// Optional UNIX timestamp after which funds can be released
    pub finish_after: Option<i64>,
    /// Optional UNIX timestamp after which sender can reclaim funds
    pub cancel_after: Option<i64>,
    /// Block height when this escrow was created.
    pub created_block: u64,
    /// Optional cryptographic (e.g., SHA-256 preimage) condition
    #[serde(flatten)]
    pub condition: Option<Condition>,
    /// Chain-specific metadata for smart contracts/programs
    #[serde(flatten)]
    pub chain_data: ChainMetadata,
}

/// Target blockchains
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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
