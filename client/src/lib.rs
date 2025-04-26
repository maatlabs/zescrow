use agents::{EthereumAgent, SolanaAgent};
use async_trait::async_trait;
use error::ClientError;
use interface::{Chain, ChainConfig, EscrowMetadata, EscrowParams};

pub mod agents;
pub mod error;
pub mod interface;

pub type Result<T> = std::result::Result<T, ClientError>;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata>;
    async fn release_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;
    async fn refund_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;
}

pub struct ZescrowClient {
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

#[async_trait]
impl Agent for EthereumAgent {
    async fn create_escrow(&self, _params: &EscrowParams) -> Result<EscrowMetadata> {
        todo!()
    }

    async fn release_escrow(&self, _metadata: &EscrowMetadata) -> Result<()> {
        todo!()
    }

    async fn refund_escrow(&self, _metadata: &EscrowMetadata) -> Result<()> {
        todo!()
    }
}

#[async_trait]
impl Agent for SolanaAgent {
    async fn create_escrow(&self, _params: &EscrowParams) -> Result<EscrowMetadata> {
        todo!()
    }

    async fn release_escrow(&self, _metadata: &EscrowMetadata) -> Result<()> {
        todo!()
    }

    async fn refund_escrow(&self, _metadata: &EscrowMetadata) -> Result<()> {
        todo!()
    }
}
