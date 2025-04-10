#[derive(Debug, thiserror::Error)]
pub enum EscrowError {
    #[error("Condition not satisfied")]
    ConditionViolation,
    #[error("Invalid state transition")]
    InvalidState,
    #[error("Timeout expired")]
    Expired,
}
