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

    #[error("identity error: {0}")]
    Identity(IdentityError),

    #[error("asset parsing error: {0}")]
    Asset(AssetError),

    #[error("unsupported chain")]
    UnsupportedChain,

    #[error("parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
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

impl From<IdentityError> for EscrowError {
    fn from(value: IdentityError) -> Self {
        Self::Identity(value)
    }
}

impl From<AssetError> for EscrowError {
    fn from(value: AssetError) -> Self {
        Self::Asset(value)
    }
}
