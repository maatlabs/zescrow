//! Chain-agnostic asset types for escrow transactions.
//!
//! This module defines the various asset kinds supported (native coins,
//! fungible tokens, non-fungible tokens, multi-tokens, and liquidity pool shares),
//! along with validation, human-readable formatting, and (de)serialization logic.

use serde::{Deserialize, Serialize};

use crate::error::AssetError;
use crate::identity::ID;
use crate::{Chain, EscrowError, Result};

/// All the "kinds" of assets we might escrow on any chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "asset_type", content = "asset", rename_all = "snake_case")]
pub enum Asset {
    /// Native chain coin (e.g., ETH, SOL).
    Native {
        /// Blockchain network identifier.
        chain: Chain,
        /// Quantity in smallest unit (e.g., wei, lamports).
        amount: u64,
    },

    /// Fungible token (e.g., ERC-20, SPL).
    Token {
        /// Blockchain network identifier.
        chain: Chain,
        /// Token contract address or program ID.
        contract: ID,
        /// Quantity in token's smallest unit.
        amount: u64,
        /// Number of decimals the token uses.
        decimals: u8,
    },

    /// Non-fungible token (e.g., ERC-721, SPL NFT)
    Nft {
        /// Blockchain network identifier.
        chain: Chain,
        /// Token contract address or program ID.
        contract: ID,
        /// Unique token identifier.
        token_id: String,
    },

    /// Multi-token (e.g., ERC-1155) with fractional ownership via `amount`.
    MultiToken {
        /// Blockchain network identifier.
        chain: Chain,
        /// Token contract address or program ID.
        contract: ID,
        /// Token identifier.
        token_id: String,
        /// Quantity in token's smallest unit.
        amount: u64,
    },

    /// Liquidity pool share (proportional ownership).
    PoolShare {
        /// Blockchain network identifier.
        chain: Chain,
        /// Pool contract address or ID.
        pool: ID,
        /// User's share quantity.
        share: u64,
        /// Total supply of pool tokens.
        total_supply: u64,
        /// Number of decimals the token uses.
        decimals: u8,
    },
}

impl Asset {
    /// Create a native asset.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zescrow_core::{Asset, Chain};
    /// let _asset = Asset::native(Chain::Ethereum, 1_000);
    /// ```
    pub fn native(chain: Chain, amount: u64) -> Self {
        Self::Native { chain, amount }
    }

    /// Create a fungible token asset.
    pub fn token(chain: Chain, contract: ID, amount: u64, decimals: u8) -> Self {
        Self::Token {
            chain,
            contract,
            amount,
            decimals,
        }
    }

    /// Create an NFT asset.
    pub fn nft(chain: Chain, contract: ID, token_id: &str) -> Self {
        Self::Nft {
            chain,
            contract,
            token_id: token_id.into(),
        }
    }

    /// Create a multi-token asset.
    pub fn multi_token(chain: Chain, contract: ID, token_id: &str, amount: u64) -> Self {
        Self::MultiToken {
            chain,
            contract,
            token_id: token_id.into(),
            amount,
        }
    }

    /// Create a liquidity pool share asset.
    pub fn pool_share(chain: Chain, pool: ID, share: u64, total_supply: u64, decimals: u8) -> Self {
        Self::PoolShare {
            chain,
            pool,
            share,
            total_supply,
            decimals,
        }
    }

    /// Ensure asset parameters are semantically valid.
    ///
    /// - Zero `amount` for fungible assets is invalid.
    /// - Pool share must be > 0, <= `total_supply`, and `total_supply` > 0.
    // TODO: Add more robust checks.
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Native { amount, .. }
            | Self::Token { amount, .. }
            | Self::MultiToken { amount, .. }
                if *amount == 0 =>
            {
                Err(AssetError::ZeroAmount.into())
            }

            Self::PoolShare {
                share,
                total_supply,
                ..
            } => {
                if *share == 0 || *total_supply == 0 || *share > *total_supply {
                    Err(AssetError::InvalidShare(*share, *total_supply).into())
                } else {
                    Ok(())
                }
            }

            _ => Ok(()),
        }
    }

    /// Returns a human-readable representation of the asset
    /// (e.g., "1.2345 USDC", "12.3456% of Pool(0xdeadbeef)").
    pub fn to_human(&self) -> Result<String> {
        self.validate()?;

        match self {
            Self::Native { chain, amount } => {
                let s = Self::format_amount(*amount, 18)?;
                Ok(format!("{} {}", s, chain.as_ref()))
            }
            Self::Token {
                contract,
                amount,
                decimals,
                ..
            } => {
                let s = Self::format_amount(*amount, *decimals)?;
                Ok(format!("{} @{}", s, contract))
            }
            Self::Nft {
                contract, token_id, ..
            } => Ok(format!("NFT {}@{}", token_id, contract)),
            Self::MultiToken {
                amount,
                token_id,
                contract,
                ..
            } => {
                let s = Self::format_amount(*amount, 0)?;
                Ok(format!("{}x{}@{}", s, token_id, contract))
            }
            Self::PoolShare {
                share,
                total_supply,
                pool,
                ..
            } => {
                let percentage = (*share as f64) / (*total_supply as f64) * 100.0;
                Ok(format!("{:.4}% of {}", percentage, pool))
            }
        }
    }

    /// Format an integer `amount` using `decimals` as fixed-point precision.
    fn format_amount(amount: u64, decimals: u8) -> Result<String> {
        let scale = 10u64
            .checked_pow(decimals as u32)
            .ok_or(AssetError::FormatOverflow(amount, decimals))?;
        let whole = amount / scale;
        let frac = amount % scale;
        let frac_str = format!("{:0>width$}", frac, width = decimals as usize);
        Ok(format!("{}.{}", whole, frac_str))
    }

    /// Returns the underlying raw quantity for this asset:
    /// - `Native`, `Token`, `MultiToken`: the `amount` field.
    /// - `PoolShare`: the `share` field.
    /// - `Nft`: implicitly `1`.
    pub fn amount(&self) -> u64 {
        match self {
            Asset::Native { amount, .. }
            | Asset::Token { amount, .. }
            | Asset::MultiToken { amount, .. } => *amount,

            Asset::PoolShare { share, .. } => *share,
            Asset::Nft { .. } => 1,
        }
    }

    /// Returns `true` if asset is native coin.
    pub fn is_native(&self) -> bool {
        matches!(self, Self::Native { .. })
    }

    /// Returns number of decimals for Token/PoolShare, otherwise None.
    pub fn decimals(&self) -> Option<u8> {
        match self {
            Self::Token { decimals, .. } | Self::PoolShare { decimals, .. } => Some(*decimals),
            _ => None,
        }
    }

    /// Returns the associated chain identifier for asset.
    pub fn chain(&self) -> &Chain {
        match self {
            Self::Native { chain, .. }
            | Self::Token { chain, .. }
            | Self::Nft { chain, .. }
            | Self::MultiToken { chain, .. }
            | Self::PoolShare { chain, .. } => chain,
        }
    }
}

impl std::fmt::Display for Asset {
    /// Compact representation for logging.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", json)
    }
}

impl std::str::FromStr for Asset {
    type Err = EscrowError;

    /// Parse an `Asset` from its JSON representation.
    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str::<Self>(s).map_err(|e| AssetError::Parsing(e.to_string()).into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn validate_zero_amount() {
        let asset = Asset::native(Chain::Ethereum, 0);
        assert!(asset.validate().is_err());
    }

    #[test]
    fn validate_pool_share_bounds() {
        let bad_asset = Asset::pool_share(Chain::Solana, ID::from(vec![]), 0, 100, 0);
        assert!(bad_asset.validate().is_err());
        let good_asset = Asset::pool_share(Chain::Solana, ID::from(vec![]), 50, 100, 0);
        assert!(good_asset.validate().is_ok());
    }
}
