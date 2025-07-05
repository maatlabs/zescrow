//! Chain-agnostic asset types for escrow transactions.
//!
//! This module defines the various asset kinds supported (native coins,
//! fungible tokens, non-fungible tokens, multi-tokens, and liquidity pool shares),
//! along with validation, human-readable formatting, and (de)serialization logic.

#[cfg(feature = "json")]
use std::str::FromStr;

use bincode::{Decode, Encode};
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::ToPrimitive;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "json")]
use serde_json;

use crate::error::AssetError;
use crate::identity::ID;
#[cfg(feature = "json")]
use crate::EscrowError;
use crate::{BigNumber, Chain, Result};

/// All the "kinds" of assets we might escrow on any chain.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(tag = "asset_type", content = "asset", rename_all = "snake_case")
)]
#[derive(Debug, Clone, Encode, Decode)]
pub enum Asset {
    /// Native chain coin (e.g., ETH, SOL).
    Native {
        /// Blockchain network identifier.
        chain: Chain,
        /// Quantity in smallest unit (e.g., wei, lamports).
        amount: BigNumber,
    },

    /// Fungible token (e.g., ERC-20, SPL).
    Token {
        /// Blockchain network identifier.
        chain: Chain,
        /// Token contract address or program ID.
        contract: ID,
        /// Quantity in token's smallest unit.
        amount: BigNumber,
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
        amount: BigNumber,
    },

    /// Liquidity pool share (proportional ownership).
    PoolShare {
        /// Blockchain network identifier.
        chain: Chain,
        /// Pool contract address or ID.
        pool: ID,
        /// User's share quantity.
        share: BigNumber,
        /// Total supply of pool tokens.
        total_supply: BigNumber,
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
    /// # use num_bigint::BigUint;
    ///
    /// let _asset = Asset::native(Chain::Ethereum, BigUint::from(1_000u64).into());
    /// ```
    pub fn native(chain: Chain, amount: BigNumber) -> Self {
        Self::Native { chain, amount }
    }

    /// Create a fungible token asset.
    pub fn token(chain: Chain, contract: ID, amount: BigNumber, decimals: u8) -> Self {
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
    pub fn multi_token(chain: Chain, contract: ID, token_id: &str, amount: BigNumber) -> Self {
        Self::MultiToken {
            chain,
            contract,
            token_id: token_id.into(),
            amount,
        }
    }

    /// Create a liquidity pool share asset.
    pub fn pool_share(
        chain: Chain,
        pool: ID,
        share: BigNumber,
        total_supply: BigNumber,
        decimals: u8,
    ) -> Self {
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
    /// - **Native**: `amount` must be > 0.
    /// - **Token**: `amount` must be > 0, `contract` must be valid `ID`.
    /// - **MultiToken**: `amount` must be > 0, `contract` must be valid `ID`, `token_id` cannot be empty.
    /// - **Nft**: `contract` must be valid `ID`, `token_id` cannot be empty.
    /// - **PoolShare**: `share` must be > 0, `total_supply` must be > 0, and `share` <= `total_supply`.
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Native { amount, .. } if *amount == Self::zero_amount() => {
                Err(AssetError::ZeroAmount.into())
            }

            Self::Token {
                contract, amount, ..
            } => {
                // verify that `contract` is a valid `ID`.
                contract.validate()?;
                if *amount == Self::zero_amount() {
                    return Err(AssetError::ZeroAmount.into());
                }
                Ok(())
            }

            Self::Nft {
                contract, token_id, ..
            } => {
                contract.validate()?;
                if token_id.is_empty() {
                    return Err(AssetError::InvalidTokenId.into());
                }
                Ok(())
            }

            Self::MultiToken {
                contract,
                token_id,
                amount,
                ..
            } => {
                contract.validate()?;
                if token_id.is_empty() {
                    return Err(AssetError::InvalidTokenId.into());
                }
                if *amount == Self::zero_amount() {
                    return Err(AssetError::ZeroAmount.into());
                }
                Ok(())
            }

            Self::PoolShare {
                pool,
                share,
                total_supply,
                ..
            } => {
                // verify that `pool` is a valid `ID`.
                pool.validate()?;
                if *share == Self::zero_amount()
                    || *total_supply == Self::zero_amount()
                    || *share > *total_supply
                {
                    return Err(
                        AssetError::InvalidShare(share.clone(), total_supply.clone()).into(),
                    );
                }
                Ok(())
            }

            _ => Ok(()),
        }
    }

    /// Attempt to serialize self into a Bincode‐encoded byte vector.
    ///
    /// Uses the standard Bincode configuration to produce a compact,
    /// little‐endian binary representation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use num_bigint::BigUint;
    /// # use zescrow_core::{Asset, Chain};
    ///
    /// let asset = Asset::native(Chain::Ethereum, BigUint::from(1_000u64).into());
    /// let bytes = asset.to_bytes().expect("serialize");
    /// assert!(!bytes.is_empty());
    /// ```
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| AssetError::Serialization(e.to_string()).into())
    }

    /// Attempt to decode Self from a Bincode‐encoded byte slice `src`.
    ///
    /// Expects `src` to match the format produced by [`Self::to_bytes`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use zescrow_core::{Asset, Chain, BigNumber};
    /// # use zescrow_core::identity::ID;
    ///
    /// let original = Asset::token(Chain::Ethereum, ID::from(vec![4, 5, 6]), BigNumber::from(1_000u64), 18);
    /// let bytes = original.to_bytes().unwrap();
    /// let decoded = Asset::from_bytes(&bytes).unwrap();
    /// assert_eq!(format!("{:?}", decoded), format!("{:?}", original));
    /// ```
    pub fn from_bytes(src: &[u8]) -> Result<Self> {
        bincode::decode_from_slice(src, bincode::config::standard())
            .map_err(|e| AssetError::Parsing(e.to_string()).into())
            .map(|(asset, _)| asset)
    }

    /// Returns a human-readable representation of the asset
    /// (e.g., "1.2345 USDC", "12.3456% of Pool(0xdeadbeef)").
    pub fn to_human(&self) -> Result<String> {
        self.validate()?;

        match self {
            Self::Native { chain, amount } => {
                let s = Self::format_amount(amount, 18)?;
                Ok(format!("{s} {}", chain.as_ref()))
            }
            Self::Token {
                contract,
                amount,
                decimals,
                ..
            } => {
                let s = Self::format_amount(amount, *decimals)?;
                Ok(format!("{s} @{contract}"))
            }
            Self::Nft {
                contract, token_id, ..
            } => Ok(format!("NFT {token_id}@{contract}")),
            Self::MultiToken {
                amount,
                token_id,
                contract,
                ..
            } => {
                let s = Self::format_amount(amount, 0)?;
                Ok(format!("{s}x{token_id}@{contract}"))
            }
            Self::PoolShare {
                share,
                total_supply,
                pool,
                ..
            } => {
                let share = share
                    .0
                    .to_f64()
                    .ok_or_else(|| AssetError::Parsing("share too large to format".into()))?;
                let total_supply = total_supply.0.to_f64().ok_or_else(|| {
                    AssetError::Parsing("total supply too large to format".into())
                })?;
                let percentage = share / total_supply * 100.0;
                Ok(format!("{percentage:.4}% of {pool}"))
            }
        }
    }

    /// Format a BigNumber `amount` using `decimals` as fixed‐point precision.
    fn format_amount(amount: &BigNumber, decimals: u8) -> Result<String> {
        let raw = &amount.0;
        let scale = BigUint::from(10u8).pow(decimals as u32);
        let (whole, rem) = raw.div_rem(&scale);

        let whole_str = whole.to_str_radix(10);
        if decimals == 0 {
            return Ok(whole_str);
        }

        let mut rem_str = rem.to_str_radix(10);
        let width = decimals as usize;
        if rem_str.len() < width {
            rem_str = format!("{rem_str:0>width$}");
        }

        Ok(format!("{whole_str}.{rem_str}"))
    }

    /// Returns the underlying raw quantity for this asset:
    /// - `Native`, `Token`, `MultiToken`: the `amount` field.
    /// - `PoolShare`: the `share` field.
    /// - `Nft`: implicitly `1`.
    pub fn amount(&self) -> BigNumber {
        match self {
            Asset::Native { amount, .. }
            | Asset::Token { amount, .. }
            | Asset::MultiToken { amount, .. } => amount.clone(),

            Asset::PoolShare { share, .. } => share.clone(),
            Asset::Nft { .. } => BigUint::from(1u64).into(),
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

    fn zero_amount() -> BigNumber {
        BigUint::from(0u64).into()
    }
}

#[cfg(feature = "json")]
impl std::fmt::Display for Asset {
    /// Compact representation for logging.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{json}")
    }
}

#[cfg(feature = "json")]
impl FromStr for Asset {
    type Err = EscrowError;

    /// Parse an `Asset` from its JSON representation.
    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str::<Self>(s).map_err(|e| AssetError::Parsing(e.to_string()).into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn to_bignum(num: u64) -> BigNumber {
        BigNumber::from(num)
    }

    #[test]
    fn native() {
        let coin = Asset::native(Chain::Ethereum, to_bignum(1));
        assert!(coin.validate().is_ok());
        let zero_coin = Asset::native(Chain::Ethereum, to_bignum(0));
        assert!(zero_coin.validate().is_err());
    }

    #[test]
    fn token() {
        let token = Asset::token(Chain::Solana, ID::from(vec![4, 5, 6]), to_bignum(1000), 9);
        assert!(token.validate().is_ok());
        // empty contract ID
        let token2 = Asset::token(Chain::Solana, ID::from(Vec::new()), to_bignum(100), 9);
        assert!(token2.validate().is_err());
        let zero_token = Asset::token(Chain::Ethereum, ID::from(vec![1, 2, 3]), to_bignum(0), 6);
        assert!(zero_token.validate().is_err());
    }

    #[test]
    fn nft() {
        let nft = Asset::nft(Chain::Ethereum, ID::from(vec![7, 8, 9]), "zescrowNFT");
        assert!(nft.validate().is_ok());
        // empty token ID
        let bad_nft = Asset::nft(Chain::Ethereum, ID::from(vec![1]), "");
        assert!(bad_nft.validate().is_err());
        // empty contract ID
        let bad_nft = Asset::nft(Chain::Ethereum, ID::from(Vec::new()), "zescrowNFT");
        assert!(bad_nft.validate().is_err());
    }

    #[test]
    fn multi_token() {
        let asset = Asset::multi_token(
            Chain::Ethereum,
            ID::from(vec![1]),
            "zescrowToken",
            to_bignum(500),
        );
        assert!(asset.validate().is_ok());
        // zero amount
        let bad_asset = Asset::multi_token(
            Chain::Ethereum,
            ID::from(vec![1]),
            "zescrowToken",
            to_bignum(0),
        );
        assert!(bad_asset.validate().is_err());
        // empty token ID
        let bad_asset = Asset::multi_token(
            Chain::Ethereum,
            ID::from(Vec::new()),
            "zescrowToken",
            to_bignum(10),
        );
        assert!(bad_asset.validate().is_err());
        // empty contract ID
        let bad_asset = Asset::multi_token(Chain::Ethereum, ID::from(vec![1]), "", to_bignum(10));
        assert!(bad_asset.validate().is_err());
    }

    #[test]
    fn pool_share() {
        let share = to_bignum(50);
        let total_supply = to_bignum(100);
        let pool_share =
            Asset::pool_share(Chain::Solana, ID::from(vec![1]), share, total_supply, 0);
        assert!(pool_share.validate().is_ok());

        let share = to_bignum(0);
        let total_supply = to_bignum(100);
        let zero_share =
            Asset::pool_share(Chain::Solana, ID::from(vec![1]), share, total_supply, 0);
        assert!(zero_share.validate().is_err());

        let share = to_bignum(10);
        let total_supply = to_bignum(0);
        let zero_supply =
            Asset::pool_share(Chain::Solana, ID::from(vec![1]), share, total_supply, 0);
        assert!(zero_supply.validate().is_err());

        // empty pool ID
        let share = to_bignum(50);
        let total_supply = to_bignum(100);
        let bad_asset =
            Asset::pool_share(Chain::Solana, ID::from(Vec::new()), share, total_supply, 0);
        assert!(bad_asset.validate().is_err());

        // share greater than total_supply
        let share = to_bignum(150);
        let total_supply = to_bignum(100);
        let bad_asset = Asset::pool_share(Chain::Solana, ID::from(vec![1]), share, total_supply, 0);
        assert!(bad_asset.validate().is_err());
    }
}
