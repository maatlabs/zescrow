pub type Result<T> = std::result::Result<T, ClientError>;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
    #[error("Chain configuration mismatch")]
    ConfigMismatch,
    #[error("Keypair error: {0}")]
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
    #[error("RPC client error")]
    SolanaRpcClient(#[from] solana_client::client_error::ClientError),
    #[error("Anchor program error")]
    Anchorlang(#[from] anchor_lang::prelude::ProgramError),
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
