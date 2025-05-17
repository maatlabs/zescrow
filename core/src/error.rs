use thiserror::Error;

/// Escrow-related errors.
#[derive(Debug, Error)]
pub enum EscrowError {
    /// Condition verification failed.
    #[error("condition error: {0}")]
    Condition(#[from] ConditionError),

    /// Attempted an invalid state transition.
    #[error("invalid state transition")]
    InvalidState,

    /// Identity parsing or validation error.
    #[error("identity error: {0}")]
    Identity(#[from] IdentityError),

    /// Asset parsing or validation error.
    #[error("asset error: {0}")]
    Asset(#[from] AssetError),

    /// Unsupported chain identifier.
    #[error("unsupported chain")]
    UnsupportedChain,

    /// Attempted a chain-specific operation and failed.
    #[error("invalid chain operation: {0}")]
    InvalidChainOp(String),

    /// JSON (de)serialization errors.
    #[error("serde_json (de)serialization error")]
    SerdeJson(#[from] serde_json::Error),

    /// I/O errors.
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}

/// Errors that can occur during cryptographic condition verification.
#[derive(Debug, Error)]
pub enum ConditionError {
    /// SHA-256(preimage) did not match the stored digest.
    #[error("preimage hash mismatch")]
    PreimageMismatch,

    /// Ed25519/Secp256k1 public key or signature was malformed,
    /// or verification failed.
    #[error("Pubkey or signature error: {0}")]
    PubkeyOrSigVerification(#[from] ed25519_dalek::SignatureError),

    /// Fewer than `threshold` subconditions passed verification.
    #[error("threshold condition not met: {valid} of {threshold} valid")]
    ThresholdNotMet { threshold: usize, valid: usize },
}

/// Errors that might occur while parsing into an `ID`.
#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("invalid hex: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("invalid base58: {0}")]
    Base58(#[from] bs58::decode::Error),

    #[error("invalid base64: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("cannot parse identity from empty string")]
    EmptyIdentity,

    #[error("unsupported identity format")]
    UnsupportedFormat,
}

/// Errors when parsing or working with `Asset`.
#[derive(Debug, Error)]
pub enum AssetError {
    #[error("could not parse asset: {0}")]
    Parsing(String),

    #[error("amount must be non-zero")]
    ZeroAmount,

    #[error("error casting amount from u128 to u64")]
    AmountOverflow,

    #[error("share must be non-zero and <= total supply (share={0}, total={1})")]
    InvalidShare(u128, u128),

    #[error("invalid decimals: {0}")]
    InvalidDecimals(u8),

    #[error("human formatting overflow: amount={0}, decimals={1}")]
    FormatOverflow(u128, u8),

    #[error("unsupported asset string format")]
    UnsupportedFormat,

    #[error("parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}
