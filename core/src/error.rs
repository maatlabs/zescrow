use thiserror::Error;

/// Escrow-related errors.
#[derive(Debug, Error, PartialEq)]
pub enum EscrowError {
    /// Condition verification failed.
    #[error("condition error: {0}")]
    Condition(#[from] ConditionError),

    /// Legacy catch-all when a condition simply isn’t satisfied.
    // TODO: Deprecate this once all `verify` calls return `ConditionError`
    #[error("condition not satisfied")]
    ConditionViolation,

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

    /// Integer parsing error.
    #[error("parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

/// Errors that can occur during cryptographic condition verification.
#[derive(Debug, Error, PartialEq)]
pub enum ConditionError {
    /// SHA-256(preimage) did not match the stored digest.
    #[error("preimage hash mismatch")]
    PreimageMismatch,

    /// Ed25519 public key or signature was malformed,
    /// or verification failed.
    #[error("Ed25519 pubkey or signature verification failed")]
    Ed25519Verification,

    /// Secp256k1 public key or signature was malformed,
    /// or verification failed.
    #[error("Secp256k1 pubkey or signature verification failed")]
    Secp256k1Verification,

    /// Fewer than `threshold` subconditions passed verification.
    #[error("threshold condition not met: {valid} of {threshold} valid")]
    ThresholdNotMet { threshold: usize, valid: usize },
}

/// Errors that might occur while parsing into an `ID`.
#[derive(Debug, Error, PartialEq)]
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
#[derive(Debug, Error, PartialEq)]
pub enum AssetError {
    #[error("amount must be non-zero")]
    ZeroAmount,

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
