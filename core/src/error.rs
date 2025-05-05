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

    /// Tried to finish escrow before its `finish_after` block.
    #[error("Escrow not yet ready to finish")]
    NotReady,

    /// Tried to refund escrow before its `cancel_after` block.
    #[error("Escrow not yet expired for refund")]
    NotExpired,
}
