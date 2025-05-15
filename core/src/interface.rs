use core::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::AssetError;
use crate::{Asset, Condition, Escrow, EscrowError, EscrowState, Party, Result};

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
    let asset = match &data.chain_data {
        ChainMetadata::Ethereum { asset, .. } => asset.clone(),
        ChainMetadata::Solana { asset, .. } => asset.clone(),
    };
    let sender = Party::from_str(&data.sender)?;
    let recipient = Party::from_str(&data.recipient)?;

    Ok(Escrow {
        asset,
        recipient,
        sender,
        condition: data.condition,
        state: EscrowState::Released,
    })
}

/// Format smallest-unit integer into a fixed-width decimal.
pub fn format_amount(amount: &u128, decimals: &u8) -> Result<String> {
    let (amount, decimals) = (*amount, *decimals);
    // TODO: check for differences in cross-chain implementations
    const MAX_DECIMALS: u8 = 38;

    if decimals > MAX_DECIMALS {
        return Err(AssetError::InvalidDecimals(decimals).into());
    }
    let ten_pow = 10u128
        .checked_pow(decimals as u32)
        .ok_or(AssetError::FormatOverflow(amount, decimals))?;
    let whole = amount / ten_pow;
    let rem = amount % ten_pow;
    let rem_str = format!("{:0>width$}", rem, width = decimals as usize);
    Ok(format!("{}.{}", whole, rem_str))
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
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl std::str::FromStr for Chain {
    type Err = EscrowError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "ethereum" | "Ethereum" | "eth" => Ok(Self::Ethereum),
            "solana" | "Solana" | "sol" => Ok(Self::Solana),
            _ => Err(EscrowError::UnsupportedChain),
        }
    }
}

/// Chain-specific metadata for smart contracts/programs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChainMetadata {
    Ethereum { asset: Asset, block_number: u64 },
    Solana { asset: Asset, pda: String, bump: u8 },
}
