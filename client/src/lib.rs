use agents::{EthereumAgent, SolanaAgent};
use async_trait::async_trait;
use error::Result;
use interface::{Chain, ChainConfig, EscrowMetadata, EscrowParams};

pub mod agents;
pub mod error;
pub mod interface;

/// Core interface for blockchain-specific escrow operations.
///
/// Implementators must provide chain-specific logic for:
/// - Creating escrow contracts/accounts
/// - Releasing funds to beneficiaries
/// - Refunding expired escrows
#[async_trait]
pub trait Agent: Send + Sync {
    /// Create a new escrow with specified parameters
    ///
    /// # Arguments
    /// * `params` - Escrow creation parameters including amounts and parties
    ///
    /// # Returns
    /// Metadata containing chain-specific identifiers and transaction details
    ///
    /// # Errors
    /// - Insufficient funds
    /// - Invalid addresses
    /// - Network errors
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata>;

    /// Release escrowed funds to beneficiary
    ///
    /// # Arguments
    /// * `metadata` - Escrow metadata from creation
    ///
    /// # Preconditions
    /// - Escrow must be in funded state
    /// - Current block/slot must be before expiry
    ///
    /// # Errors
    /// - Escrow not funded/created
    /// - Expiry reached
    /// - Authorization failures
    async fn release_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;

    /// Refund escrowed funds to depositor
    ///
    /// # Arguments
    /// * `metadata` - Escrow metadata from creation
    ///
    /// # Preconditions
    /// - Escrow must have expired
    /// - No prior release/refund executed
    ///
    /// # Errors
    /// - Early refund attempt
    /// - State inconsistencies
    async fn refund_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;
}

/// Unified client for cross-chain escrow management
pub struct ZescrowClient {
    /// Chain-specific escrow agent
    pub agent: Box<dyn Agent>,
}

impl ZescrowClient {
    pub fn new(chain: Chain, config: ChainConfig) -> Result<Self> {
        let agent: Box<dyn Agent> = match chain {
            Chain::Ethereum => Box::new(EthereumAgent::new(config)?),
            Chain::Solana => Box::new(SolanaAgent::new(config)?),
        };
        Ok(Self { agent })
    }

    pub async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        self.agent.create_escrow(params).await
    }

    pub async fn release_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        self.agent.release_escrow(metadata).await
    }

    pub async fn refund_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        self.agent.refund_escrow(metadata).await
    }
}
