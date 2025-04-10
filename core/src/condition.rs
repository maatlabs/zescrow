//! Escrow conditions and deterministic verification logic.

use serde::{Deserialize, Serialize};

use crate::{EscrowError, Result};

/// Escrow conditions must be deterministically verifiable,
/// representing logic that governs the release of escrowed assets.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Condition {
    /// Requires a minimum number (`threshold`) of valid signatures from approved signers.
    MultiSig {
        threshold: usize,
        signers: Vec<[u8; 32]>,
        signatures: Vec<[u8; 32]>,
    },
    /// Requires execution within a certain block height (time constraint).
    TimeLock { expiry_block: u64 },
}

impl Condition {
    /// Verifies if the escrow condition is met given external inputs.
    ///
    /// # Arguments
    /// * `current_block` - Current block height (required for TimeLock)
    pub fn verify(&self, current_block: Option<u64>) -> Result<()> {
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
            Self::TimeLock { expiry_block } => match current_block {
                Some(block) if block <= *expiry_block => Ok(()),
                _ => Err(EscrowError::Expired),
            },
        }
    }
}
