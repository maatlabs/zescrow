use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    #[error("Chain configuration mismatch")]
    ConfigMismatch,

    #[error("Solana keypair: {0}")]
    Keypair(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("URL parse error")]
    UrlParse(#[from] url::ParseError),

    #[error("Address parse error")]
    AddressParse(#[from] rustc_hex::FromHexError),

    #[error("Invalid chain operation")]
    InvalidChainOperation,

    #[error("Solana RPC client error")]
    SolanaRpcClient(#[from] Box<solana_client::client_error::ClientError>),

    #[error("Anchor program error")]
    Anchorlang(#[from] anchor_lang::prelude::ProgramError),

    #[error("Chain agent error: {0}")]
    AgentError(#[from] AgentError),

    #[error("Transaction dropped")]
    TxDropped,

    #[error("Asset amount overflow")]
    AssetOverflow,

    #[error("{0}")]
    MissingEvent(String),

    #[error("Zescrow core: {0}")]
    ZescrowCore(String),

    #[error("RISC Zero prover/verifier: {0}")]
    ZkProver(String),
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Ethereum agent: {0}")]
    Ethereum(String),
    #[error("Solana agent: {0}")]
    Solana(String),
}

impl From<solana_client::client_error::ClientError> for ClientError {
    fn from(value: solana_client::client_error::ClientError) -> Self {
        Self::SolanaRpcClient(Box::new(value))
    }
}

impl From<ethers::providers::ProviderError> for ClientError {
    fn from(value: ethers::providers::ProviderError) -> Self {
        Self::BlockchainError(value.to_string())
    }
}

impl From<ethers::signers::WalletError> for ClientError {
    fn from(value: ethers::signers::WalletError) -> Self {
        Self::BlockchainError(value.to_string())
    }
}

impl From<solana_sdk::pubkey::ParsePubkeyError> for ClientError {
    fn from(value: solana_sdk::pubkey::ParsePubkeyError) -> Self {
        Self::BlockchainError(value.to_string())
    }
}

impl From<zescrow_core::error::EscrowError> for ClientError {
    fn from(value: zescrow_core::error::EscrowError) -> Self {
        Self::ZescrowCore(value.to_string())
    }
}
