//! Zescrow client library for cross-chain escrow operations.
//!
//! This crate provides the [`ZescrowClient`] for creating, finishing, and
//! canceling escrows across multiple blockchains, along with RISC Zero
//! zkVM integration for zero-knowledge proof generation.
//!
//! # Supported Chains
//!
//! - **Ethereum**: Via [`EthereumAgent`]
//! - **Solana**: Via [`SolanaAgent`]
//!
//! # Example
//!
//! ```ignore
//! use zescrow_client::{ZescrowClient, Recipient};
//! use zescrow_core::interface::ChainConfig;
//!
//! async fn create_escrow(config: &ChainConfig) -> anyhow::Result<()> {
//!     let client = ZescrowClient::builder(config).build().await?;
//!     // Use client.create_escrow(), client.finish_escrow(), etc.
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;

pub use error::ClientError;
pub use ethereum::EthereumAgent;
use ethers::signers::LocalWallet;
pub use solana::SolanaAgent;
use tracing::{debug, info};
use zescrow_core::interface::ChainConfig;
use zescrow_core::{Chain, EscrowMetadata, EscrowParams};

pub mod error;
pub mod ethereum;
pub mod prover;
pub mod solana;

/// Result type alias using [`ClientError`].
pub type Result<T> = std::result::Result<T, ClientError>;

/// Core interface for blockchain-specific escrow operations.
///
/// Implementors must provide chain-specific logic for:
/// - Creating escrow contracts/programs and/or accounts
/// - Releasing funds to beneficiaries
/// - Refunding expired escrows
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to support async execution.
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// Creates a new escrow with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Escrow creation parameters including assets, parties, and timelocks
    ///
    /// # Returns
    ///
    /// Metadata containing chain-specific identifiers and the escrow state.
    ///
    /// # Errors
    ///
    /// Returns an error if transaction submission or confirmation fails.
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata>;

    /// Releases escrowed funds to the beneficiary.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Escrow metadata from creation
    ///
    /// # Preconditions
    ///
    /// - Escrow must be in funded state
    /// - Current block/slot must be at or after `finish_after` (if set)
    /// - Caller must be the recipient
    ///
    /// # Errors
    ///
    /// Returns an error if the caller is not authorized or timelocks are not met.
    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;

    /// Refunds escrowed funds to the depositor.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Escrow metadata from creation
    ///
    /// # Preconditions
    ///
    /// - `cancel_after` must be set in the escrow
    /// - Current block/slot must be at or after `cancel_after`
    /// - Caller must be the sender
    ///
    /// # Errors
    ///
    /// Returns an error if cancellation is not allowed or timelocks are not met.
    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;
}

/// Unified client for cross-chain escrow management.
///
/// Wraps a chain-specific [`Agent`] to provide a consistent interface
/// for escrow operations across chains.
pub struct ZescrowClient {
    /// The underlying blockchain agent.
    pub agent: Box<dyn Agent>,
}

/// Builder for constructing [`ZescrowClient`] instances.
///
/// Use [`ZescrowClient::builder`] to create a new builder.
pub struct ZescrowClientBuilder {
    config: ChainConfig,
    recipient: Option<Recipient>,
}

/// Recipient key configuration for escrow operations.
///
/// Different chains use different key formats:
/// - Ethereum uses wallet private keys (hex-encoded)
/// - Solana uses keypair files (JSON)
#[derive(Debug, Clone)]
pub enum Recipient {
    /// Ethereum wallet for signing transactions.
    Ethereum(LocalWallet),
    /// Path to a Solana keypair JSON file.
    Solana(PathBuf),
}

impl ZescrowClient {
    /// Creates a new builder for constructing a client.
    ///
    /// # Arguments
    ///
    /// * `config` - Chain configuration specifying the target blockchain
    pub fn builder(config: &ChainConfig) -> ZescrowClientBuilder {
        ZescrowClientBuilder {
            config: config.clone(),
            recipient: None,
        }
    }

    /// Creates an escrow on-chain.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters defining the escrow terms
    ///
    /// # Returns
    ///
    /// Metadata for the created escrow, including chain-specific identifiers.
    pub async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        let metadata = self.agent.create_escrow(params).await?;
        debug!(?metadata, "Escrow created");
        Ok(metadata)
    }

    /// Releases an existing escrow to the recipient.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Escrow metadata from creation
    pub async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        self.agent.finish_escrow(metadata).await.inspect(|_| {
            debug!("Escrow released");
        })
    }

    /// Cancels an existing escrow and refunds the sender.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Escrow metadata from creation
    pub async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        self.agent.cancel_escrow(metadata).await.inspect(|_| {
            debug!("Escrow cancelled");
        })
    }
}

impl ZescrowClientBuilder {
    /// Sets the recipient key for finish operations.
    ///
    /// This is required when calling [`ZescrowClient::finish_escrow`].
    pub fn recipient(mut self, recipient: Recipient) -> Self {
        self.recipient = Some(recipient);
        self
    }

    /// Builds the client, instantiating the appropriate chain agent.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The recipient key type doesn't match the chain
    /// - Agent initialization fails
    pub async fn build(self) -> Result<ZescrowClient> {
        debug!("Building ZescrowClient with config: {:?}", self.config);

        let agent: Box<dyn Agent> = match &self.config.chain {
            Chain::Ethereum => {
                let wallet = self.ethereum_wallet()?;
                debug!(wallet_present = wallet.is_some(), "Selected EthereumAgent");
                Box::new(EthereumAgent::new(&self.config, wallet).await?)
            }
            Chain::Solana => {
                let keypair_path = self.solana_keypair()?;
                debug!(
                    keypair_present = keypair_path.is_some(),
                    "Selected SolanaAgent"
                );
                Box::new(SolanaAgent::new(&self.config, keypair_path).await?)
            }
        };

        info!("Agent initialized successfully");
        Ok(ZescrowClient { agent })
    }

    /// Extracts the Ethereum wallet from the recipient configuration.
    fn ethereum_wallet(&self) -> Result<Option<LocalWallet>> {
        match &self.recipient {
            Some(Recipient::Ethereum(w)) => Ok(Some(w.clone())),
            Some(Recipient::Solana(_)) => Err(ClientError::Keypair(
                "expected Ethereum wallet for Ethereum chain".into(),
            )),
            None => Ok(None),
        }
    }

    /// Extracts the Solana keypair path from the recipient configuration.
    fn solana_keypair(&self) -> Result<Option<PathBuf>> {
        match &self.recipient {
            Some(Recipient::Solana(path)) => Ok(Some(path.clone())),
            Some(Recipient::Ethereum(_)) => Err(ClientError::Keypair(
                "expected Solana keypair file for Solana chain".into(),
            )),
            None => Ok(None),
        }
    }
}

impl std::str::FromStr for Recipient {
    type Err = ClientError;

    /// Parses a recipient from a string.
    ///
    /// - Strings starting with `0x` are parsed as Ethereum private keys
    /// - Other strings are treated as paths to Solana keypair files
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.strip_prefix("0x")
            .map(|_| {
                s.parse::<LocalWallet>()
                    .map(Self::Ethereum)
                    .map_err(|e| ClientError::Keypair(e.to_string()))
            })
            .unwrap_or_else(|| Ok(Self::Solana(PathBuf::from(s))))
    }
}
