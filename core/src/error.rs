/// Escrow-related errors.
#[derive(Debug, thiserror::Error)]
pub enum EscrowError {
    /// Occurs when the specified escrow conditions are not satisfied.
    #[error("Condition not satisfied")]
    ConditionViolation,
    /// Occurs upon invalid or inappropriate escrow state transitions.
    #[error("Invalid state transition")]
    InvalidState,
    /// Triggered when escrow execution has surpassed allowed time constraints.
    #[error("Timeout expired")]
    Expired,
    /// Occurs when a party calls for a refund while timeout is not expired.
    #[error("Escrow not yet expired for refund")]
    NotExpired,
}
