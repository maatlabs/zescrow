use std::path::PathBuf;

use error::ClientError;
pub use ethereum::EthereumAgent;
use ethers::signers::LocalWallet;
pub use solana::SolanaAgent;
use tracing::{debug, info};
use zescrow_core::interface::ChainConfig;
use zescrow_core::{Chain, EscrowMetadata, EscrowParams};

pub mod error;
pub mod ethereum;
pub mod solana;
pub mod util;

pub type Result<T> = std::result::Result<T, ClientError>;

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

/// Unified client for cross-chain escrow management.
pub struct ZescrowClient {
    pub agent: Box<dyn Agent>,
}

/// Builder for `ZescrowClient`.
pub struct ZescrowClientBuilder {
    chain: Chain,
    config: ChainConfig,
    recipient: Option<Recipient>,
}

#[derive(Debug, Clone)]
pub enum Recipient {
    Ethereum(LocalWallet),
    Solana(PathBuf),
}

impl std::str::FromStr for Recipient {
    type Err = ClientError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.strip_prefix("0x").is_some() {
            let wallet = s
                .parse::<LocalWallet>()
                .map_err(|e| ClientError::Keypair(e.to_string()))?;
            Ok(Self::Ethereum(wallet))
        } else {
            Ok(Self::Solana(PathBuf::from(s)))
        }
    }
}

impl ZescrowClient {
    /// Begin constructing a new client.
    pub fn builder(chain: Chain, config: ChainConfig) -> ZescrowClientBuilder {
        ZescrowClientBuilder {
            chain,
            config,
            recipient: None,
        }
    }
}

impl ZescrowClientBuilder {
    /// Specify the recipient key (for finish operations).
    pub fn recipient(mut self, recipient: Recipient) -> Self {
        self.recipient = Some(recipient);
        self
    }

    /// Finish building the client, instantiating the appropriate agent.
    pub async fn build(self) -> Result<ZescrowClient> {
        debug!("Building ZescrowClient with config: {:?}", self.config);

        let agent: Box<dyn Agent> = match &self.chain {
            Chain::Ethereum => {
                let wallet = match &self.recipient {
                    Some(Recipient::Ethereum(w)) => Some(w.clone()),
                    Some(Recipient::Solana(_)) => {
                        return Err(ClientError::Keypair(
                            "Expected Ethereum key for Ethereum escrows".into(),
                        ));
                    }
                    None => None,
                };
                debug!(
                    "Selected EthereumAgent, wallet present={}",
                    wallet.is_some()
                );
                Box::new(EthereumAgent::new(&self.config, wallet).await?)
            }
            Chain::Solana => {
                let keypair = match &self.recipient {
                    Some(Recipient::Solana(path)) => Some(path.clone()),
                    Some(Recipient::Ethereum(_)) => {
                        return Err(ClientError::Keypair(
                            "Expected Solana keypair file for Solana escrows".into(),
                        ));
                    }
                    None => None,
                };
                debug!(
                    "Selected SolanaAgent, keypair present={}",
                    keypair.is_some()
                );
                Box::new(SolanaAgent::new(&self.config, keypair).await?)
            }
        };

        info!("Agent initialized successfully");
        Ok(ZescrowClient { agent })
    }
}

impl ZescrowClient {
    /// Create an escrow on-chain.
    pub async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        let metadata = self.agent.create_escrow(params).await?;
        debug!(?metadata, "Escrow created");
        Ok(metadata)
    }

    /// Release an existing escrow.
    pub async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let res = self.agent.finish_escrow(metadata).await;
        if res.is_ok() {
            debug!("Escrow released");
        }
        res
    }

    /// Cancel an existing escrow.
    pub async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let res = self.agent.cancel_escrow(metadata).await;
        if res.is_ok() {
            debug!("Escrow cancelled");
        }
        res
    }
}
