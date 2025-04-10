use serde::{Deserialize, Serialize};

use crate::error::EscrowError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Condition {
    MultiSig {
        threshold: usize,
        signers: Vec<[u8; 32]>,
        signatures: Vec<[u8; 32]>,
    },
    TimeLock {
        current_block: u64,
        expiry_block: u64,
    },
}

impl Condition {
    pub fn verify(&self) -> Result<(), EscrowError> {
        match self {
            Self::MultiSig {
                threshold,
                signers,
                signatures,
            } => {
                if signatures.len() >= *threshold && signatures.iter().all(|s| signers.contains(s))
                {
                    Ok(())
                } else {
                    Err(EscrowError::ConditionViolation)
                }
            }
            Self::TimeLock {
                current_block,
                expiry_block,
            } => {
                if current_block <= expiry_block {
                    Ok(())
                } else {
                    Err(EscrowError::ConditionViolation)
                }
            }
        }
    }
}
