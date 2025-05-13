use thiserror::Error;

/// Escrow-related errors.
#[derive(Debug, Error, PartialEq)]
pub enum EscrowError {
    /// Crypto condition (e.g., Condition::Threshold) not met.
    #[error("condition not satisfied")]
    ConditionViolation,

    /// Attempted an invalid state transition.
    #[error("invalid state transition")]
    InvalidState,

    #[error("invalid length for fixed-size array")]
    InvalidLength,

    #[error("invalid hex: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("invalid base58: {0}")]
    Base58(#[from] bs58::decode::Error),

    #[error("invalid base64: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("cannot parse identity from empty string")]
    EmptyIdentity,

    #[error(
        "unsupported identity format: expected one of `hex:…`, `base58:…`, `base64:…`, or JSON"
    )]
    UnsupportedFormat,
}
