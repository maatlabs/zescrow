pub mod assets;
pub mod conditions;
pub mod escrow;

use bls12_381::G1Affine;
use escrow::State;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

/// Represents a participant in escrow with ZK identity.
///
/// # Safety
/// - `identity_commitment` must be generated using collision-resistant hashing
/// - `bls_public_key` must use BLS12-381 compressed serialization
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct Party {
    /// Pedersen commitment to participant's identity
    pub id_commitment: [u8; 32],

    /// BLS12-381 compressed public key (48 bytes).
    /// None for anonymous participants
    pub bls_public_key: Option<[u8; 48]>,
}

impl Party {
    /// Validate public key format.
    pub fn validate(&self) -> Result<(), EscrowError> {
        if let Some(pubkey) = &self.bls_public_key {
            G1Affine::from_compressed(pubkey)
                .into_option()
                .ok_or(EscrowError::InvalidPublicKey)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct VerificationCtx {
    pub time: TimeSource,
    pub signatures: Vec<[u8; 48]>,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum TimeSource {
    BlockNumber(u64),
    Timestamp(u64),
}

#[derive(thiserror::Error, Debug, Encode, Decode)]
pub enum EscrowError {
    #[error("Condition verification failed")]
    ConditionFailure,
    #[error("Invalid BLS signature")]
    InvalidSignature,
    #[error("Invalid BLS public key")]
    InvalidPublicKey,
    #[error("Invalid state transition: expected {expected:?}, got {actual:?}")]
    StateTransitionViolation { expected: State, actual: State },
    #[error("Failed to generate random data")]
    EntropyGenerationFailed,
}
