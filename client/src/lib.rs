use std::path::PathBuf;

use error::{ClientError, Result};
pub use ethereum_agent::EthereumAgent;
use ethers::signers::LocalWallet;
pub use solana_agent::SolanaAgent;
use zescrow_core::interface::ChainConfig;
use zescrow_core::{Chain, EscrowMetadata, EscrowParams};

pub mod error;
pub mod ethereum_agent;
pub mod solana_agent;

/// Core interface for blockchain-specific escrow operations.
///
/// Implementators must provide chain-specific logic for:
/// - Creating escrow contracts/accounts
/// - Releasing funds to beneficiaries
/// - Refunding expired escrows
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// Create a new escrow with specified parameters
    ///
    /// # Arguments
    /// * `params` - Escrow creation parameters including assets and parties
    ///
    /// # Returns
    /// Metadata containing chain-specific identifiers and transaction details
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata>;

    /// Release escrowed funds to beneficiary
    ///
    /// # Arguments
    /// * `metadata` - Escrow metadata from creation
    ///
    /// # Preconditions
    /// - Escrow must be in funded state
    /// - Current block/slot must be before expiry
    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;

    /// Refund escrowed funds to depositor
    ///
    /// # Arguments
    /// * `metadata` - Escrow metadata from creation
    ///
    /// # Preconditions
    /// - Escrow must have expired
    /// - No prior release/refund executed
    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;
}

/// Unified client for cross-chain escrow management
pub struct ZescrowClient {
    /// Chain-specific escrow agent
    pub agent: Box<dyn Agent>,
}

impl ZescrowClient {
    pub async fn new(
        chain: &Chain,
        config: &ChainConfig,
        recipient: Option<Recipient>,
    ) -> Result<Self> {
        let agent: Box<dyn Agent> = match chain {
            Chain::Ethereum => {
                let maybe_wallet = match recipient {
                    Some(Recipient::Ethereum(w)) => Some(w),
                    Some(Recipient::Solana(_)) => {
                        return Err(ClientError::Keypair(
                            "Expected Ethereum key for Ethereum escrows".into(),
                        ));
                    }
                    None => None,
                };
                Box::new(EthereumAgent::new(config, maybe_wallet).await?)
            }

            Chain::Solana => {
                let maybe_keypath = match recipient {
                    None => None,
                    Some(Recipient::Solana(keypath)) => Some(keypath),
                    Some(Recipient::Ethereum(_)) => {
                        return Err(ClientError::Keypair(
                            "Expected Solana keypair file for Solana escrows".into(),
                        ));
                    }
                };
                Box::new(SolanaAgent::new(config, maybe_keypath).await?)
            }
        };

        Ok(Self { agent })
    }

    pub async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        self.agent.create_escrow(params).await
    }

    pub async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        self.agent.finish_escrow(metadata).await
    }

    pub async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        self.agent.cancel_escrow(metadata).await
    }
}

#[derive(Debug, Clone)]
pub enum Recipient {
    Ethereum(LocalWallet),
    Solana(PathBuf),
}

impl std::str::FromStr for Recipient {
    type Err = ClientError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // If it looks like a hex key, parse as Ethereum
        if let Some(_hex) = s.strip_prefix("0x") {
            let wallet = s.parse::<LocalWallet>()?;
            return Ok(Self::Ethereum(wallet));
        }
        // Otherwise treat as Solana keypair path
        Ok(Self::Solana(PathBuf::from(s)))
    }
}
