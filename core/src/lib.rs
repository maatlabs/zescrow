pub mod assets;
pub mod conditions;
pub mod escrow;

use scale::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct Party {
    pub commitment: [u8; 32],
    pub signature: Option<[u8; 48]>,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct VerificationCtx {
    pub current_block: u64,
    pub current_timestamp: u64,
    pub signatures: Vec<[u8; 48]>,
}

#[derive(thiserror::Error, Debug, Encode, Decode)]
pub enum EscrowError {
    #[error("Escrow state not initialized")]
    Uninitialized,
    #[error("Condition verification failed")]
    ConditionFailure,
    #[error("Invalid BLS signature")]
    InvalidSignature,
    #[error("Not enough funds in escrow account")]
    InsufficientFunds,
}
