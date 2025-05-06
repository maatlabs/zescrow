//! Escrow-related errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EscrowError {
    /// Crypto condition (multi-sig or preimage) not met.
    #[error("Condition not satisfied")]
    ConditionViolation,

    /// Attempted an invalid state transition.
    #[error("Invalid state transition")]
    InvalidState,
}
