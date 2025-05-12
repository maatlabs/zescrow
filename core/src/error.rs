//! Escrow-related errors

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum EscrowError {
    /// Crypto condition (multi-sig or preimage) not met.
    #[error("condition not satisfied")]
    ConditionViolation,

    /// Attempted an invalid state transition.
    #[error("invalid state transition")]
    InvalidState,

    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("invalid length for fixed-size array")]
    InvalidLength,
}
