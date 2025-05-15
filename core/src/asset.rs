use serde::{Deserialize, Serialize};

use crate::error::{AssetError, IdentityError};
use crate::identity::ID;
use crate::{Chain, EscrowError, Result};

/// All the "kinds" of assets we might escrow on any chain.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "asset_type", rename_all = "snake_case")]
pub enum Asset {
    /// Native coin of a chain (e.g., ETH, SOL).
    Native {
        chain: Chain,
        /// Amount in the smallest unit (e.g., wei, lamports).
        amount: u128,
    },
    /// Contract/program-based fungible token (e.g., ERC-20, SPL).
    Token {
        chain: Chain,
        contract: ID,
        amount: u128,
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
        amount: u128,
    },
    /// Liquidity-pool share, staking derivative, etc.
    PoolShare {
        chain: Chain,
        pool: ID,
        share: u128,
        /// Total supply of pool tokens.
        total_supply: u128,
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
    fn format_amount(amount: &u128, decimals: &u8) -> Result<String> {
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

// TODO: Proper handling of Asset construction from JSON
impl std::str::FromStr for Asset {
    type Err = EscrowError;

    fn from_str(s: &str) -> Result<Self> {
        let parts = s.split(':').collect::<Vec<_>>();
        match parts.as_slice() {
            ["native", chain, amount] => {
                let amount = amount.parse()?;
                let chain = chain.parse()?;

                Ok(Self::Native { chain, amount })
            }

            ["token", chain, contract, amount, decimals] => {
                let contract =
                    ID::from_str(contract).map_err(|_| IdentityError::UnsupportedFormat)?;
                let amount = amount.parse()?;
                let decimals = decimals.parse()?;
                let chain = chain.parse()?;

                Ok(Self::Token {
                    chain,
                    contract,
                    amount,
                    decimals,
                })
            }

            ["nft", chain, contract, token_id] => {
                let contract =
                    ID::from_str(contract).map_err(|_| IdentityError::UnsupportedFormat)?;
                let chain = chain.parse()?;

                Ok(Self::Nft {
                    chain,
                    contract,
                    token_id: token_id.to_string(),
                })
            }

            ["multi", chain, contract, token_id, amount] => {
                let contract =
                    ID::from_str(contract).map_err(|_| IdentityError::UnsupportedFormat)?;
                let amount = amount.parse()?;
                let chain = chain.parse()?;

                Ok(Self::MultiToken {
                    chain,
                    contract,
                    token_id: token_id.to_string(),
                    amount,
                })
            }

            ["pool", chain, pool, share, total_supply, decimals] => {
                let pool = ID::from_str(pool).map_err(|_| IdentityError::UnsupportedFormat)?;
                let share = share.parse()?;
                let total_supply = total_supply.parse()?;
                let chain = chain.parse().map_err(|_| AssetError::UnsupportedFormat)?;
                let decimals = decimals.parse()?;

                Ok(Self::PoolShare {
                    chain,
                    pool,
                    share,
                    total_supply,
                    decimals,
                })
            }

            _ => Err(AssetError::UnsupportedFormat.into()),
        }
    }
}
