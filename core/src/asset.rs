use serde::{Deserialize, Serialize};

use crate::error::AssetError;
use crate::identity::ID;
use crate::{Chain, EscrowError, Result};

/// All the "kinds" of assets we might escrow on any chain.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "asset_type", content = "asset", rename_all = "snake_case")]
pub enum Asset {
    /// Native coin of a chain (e.g., ETH, SOL).
    Native {
        chain: Chain,
        /// Amount in the smallest unit (e.g., wei, lamports).
        amount: u64,
    },
    /// Contract/program-based fungible token (e.g., ERC-20, SPL).
    Token {
        chain: Chain,
        contract: ID,
        amount: u64,
        /// Number of decimals.
        decimals: u8,
    },
    /// Standard non-fungible token (e.g., ERC-721, SPL NFT)
    Nft {
        chain: Chain,
        contract: ID,
        token_id: String,
    },
    /// Semi-fungible "mixed" token (e.g., ERC-1155).
    MultiToken {
        chain: Chain,
        contract: ID,
        token_id: String,
        amount: u64,
    },
    /// Liquidity-pool share, staking derivative, etc.
    PoolShare {
        chain: Chain,
        pool: ID,
        share: u64,
        /// Total supply of pool tokens.
        total_supply: u64,
        decimals: u8,
    },
}

impl Asset {
    /// Validate by enforcing zero-amount and share invariants.
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

    /// Returns a human-readable form of the asset (e.g., "1.2345 USDC", "12.3456% of Pool(0xdeadbeef)")
    pub fn to_human(&self) -> Result<String> {
        self.validate()?;

        match self {
            Self::Native { chain, amount } => {
                let s = Self::format_amount(amount, &18)?;
                Ok(format!("{} {}", s, chain.as_ref()))
            }
            Self::Token {
                contract,
                amount,
                decimals,
                ..
            } => {
                let s = Self::format_amount(amount, decimals)?;
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
            } => Ok(format!("{}x{}@{}", amount, token_id, contract)),
            Self::PoolShare {
                share,
                total_supply,
                pool,
                ..
            } => {
                let pct = (*share as f64) / (*total_supply as f64) * 100.0;
                Ok(format!("{:.4}% of {}", pct, pool))
            }
        }
    }

    // Format smallest-unit integer into a fixed-width decimal.
    fn format_amount(amount: &u64, decimals: &u8) -> Result<String> {
        let (amount, decimals) = (*amount, *decimals);
        // TODO: check for differences in cross-chain implementations
        const MAX_DECIMALS: u8 = 38;

        if decimals > MAX_DECIMALS {
            return Err(AssetError::InvalidDecimals(decimals).into());
        }
        let ten_pow = 10u64
            .checked_pow(decimals as u32)
            .ok_or(AssetError::FormatOverflow(amount, decimals))?;
        let whole = amount / ten_pow;
        let rem = amount % ten_pow;
        let rem_str = format!("{:0>width$}", rem, width = decimals as usize);
        Ok(format!("{}.{}", whole, rem_str))
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
            Asset::Nft { .. } => 1u64,
        }
    }

    /// Checks if asset is a native coin.
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native { amount, chain } => write!(f, "Native[{}:{}]", chain.as_ref(), amount),
            Self::Token {
                amount,
                contract,
                decimals,
                ..
            } => write!(
                f,
                "Token[{} units of {} ({} decimals)]",
                amount, contract, decimals
            ),
            Self::Nft {
                contract, token_id, ..
            } => write!(f, "NFT[{}#{}]", contract, token_id),
            Self::MultiToken {
                amount,
                token_id,
                contract,
                ..
            } => write!(f, "MultiToken[{}x{} at {}]", amount, token_id, contract),
            Self::PoolShare {
                pool,
                share,
                total_supply,
                ..
            } => write!(f, "PoolShare[{} of {} (tp {})]", share, pool, total_supply),
        }
    }
}

impl std::str::FromStr for Asset {
    type Err = EscrowError;

    /// Parses an `Asset` from its JSON representation.
    ///
    /// Expects a JSON object matching the `Asset` enum’s
    /// `asset_type`/`asset_data` tagging, e.g.:
    /// ```json
    /// {
    ///   "asset_type": "token",
    ///   "asset": {
    ///     "chain": "ethereum",
    ///     "contract": "0xdeadbeef…",
    ///     "amount": 1000,
    ///     "decimals": 18
    ///   }
    /// }
    /// ```
    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str::<Self>(s).map_err(|e| AssetError::Parsing(e.to_string()).into())
    }
}
