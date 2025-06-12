use thiserror::Error;

use crate::BigNumber;

/// Errors arising from on-chain `Escrow` operations and parameter validation.
#[derive(Debug, Error)]
pub enum EscrowError {
    /// A cryptographic condition failed to verify.
    #[error("condition error: {0}")]
    Condition(#[from] ConditionError),

    /// Attempted to transition an `Escrow` in an invalid state.
    #[error("invalid state transition")]
    InvalidState,

    /// An identity could not be parsed or validated.
    #[error("identity error: {0}")]
    Identity(#[from] IdentityError),

    /// An asset failed parsing, validation, or formatting.
    #[error("asset error: {0}")]
    Asset(#[from] AssetError),

    /// Attempted a chain-specific operation on the wrong network
    /// (e.g., getting Solana PDA on Ethereum).
    #[error("invalid chain operation: {0}")]
    InvalidChainOp(String),

    /// An I/O error occurred (e.g., reading or writing JSON files).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing or serialization error.
    #[cfg(feature = "json")]
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// The specified blockchain network is not supported.
    #[error("unsupported chain specified")]
    UnsupportedChain,
}

/// Errors related to cryptographic condition verification.
#[derive(Debug, Error)]
pub enum ConditionError {
    /// The provided preimage did not hash to the expected value.
    #[error("preimage mismatch")]
    PreimageMismatch,

    /// Public key parsing or signature verification failed.
    #[error("public key or signature verification failed: {0}")]
    PubkeyOrSigVerification(#[from] ed25519_dalek::SignatureError),

    /// Fewer than the required number of subconditions were satisfied.
    #[error("threshold not met: required {threshold}, valid {valid}")]
    ThresholdNotMet {
        /// Minimum number of valid subconditions required.
        threshold: usize,
        /// Number of verified subconditions.
        valid: usize,
    },
}

/// Errors related to identity parsing and validation.
#[derive(Debug, Error)]
pub enum IdentityError {
    /// The provided identity string was empty.
    #[error("empty identity string")]
    EmptyIdentity,

    /// Error decoding a hex-encoded identity.
    #[error("hex decoding error: {0}")]
    Hex(#[from] hex::FromHexError),

    /// Error decoding a Base58-encoded identity.
    #[error("Base58 decoding error: {0}")]
    Base58(#[from] bs58::decode::Error),

    /// Error decoding a Base64-encoded identity.
    #[error("Base64 decoding error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// The input string did not match any supported identity format (hex, Base58, Base64).
    #[error("unsupported identity format")]
    UnsupportedFormat,
}

/// Errors related to asset parsing, validation, or formatting.
#[derive(Debug, Error)]
pub enum AssetError {
    /// Failed to parse an asset from a string or JSON.
    #[error("could not serialize asset: {0}")]
    Serialization(String),

    /// Failed to parse an asset from bytes or JSON.
    #[error("could not parse asset: {0}")]
    Parsing(String),

    /// A fungible or multi-token amount was zero, which is not allowed.
    #[error("amount must be non-zero")]
    ZeroAmount,

    /// The provided token ID of an NFT or multi-token is invalid or empty.
    #[error("provided token ID is invalid or empty")]
    InvalidTokenId,

    /// A liquidity pool share was invalid (must be > 0 and <= total supply).
    #[error("share must be non-zero and <= total supply (share={0}, total={1})")]
    InvalidShare(BigNumber, BigNumber),

    /// The specified number of decimals was invalid.
    #[error("invalid decimals: {0}")]
    InvalidDecimals(u8),

    /// Fixed-point formatting overflow (e.g., amount or decimals too large).
    #[error("human formatting overflow: amount={0}, decimals={1}")]
    FormatOverflow(BigNumber, u8),

    /// The provided asset string did not match a supported format.
    #[error("unsupported asset string format")]
    UnsupportedFormat,

    /// Error parsing an integer (e.g., token ID or amount) from a string.
    #[error("integer parsing error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

impl EscrowError {
    /// A helper to bypass the unavailability of the `ToString` trait
    /// in the RISC Zero guest.
    pub fn to_str(&self) -> String {
        self.to_string()
    }
}
