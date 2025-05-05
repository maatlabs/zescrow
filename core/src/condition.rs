//! Escrow conditions and deterministic verification logic.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{EscrowError, Result};

/// Deterministic crypto conditions for release.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Condition {
    /// Require ≥ `threshold` valid signatures from among `signers`.
    MultiSig {
        threshold: usize,
        signers: Vec<[u8; 32]>,
        signatures: Vec<[u8; 32]>,
    },
    /// Require a SHA-256 preimage match.
    Preimage {
        /// Expected SHA-256 hash
        hash: [u8; 32],
        /// Provided preimage bytes
        preimage: Vec<u8>,
    },
}

impl Condition {
    /// Verify the condition; returns `Err(EscrowError::ConditionViolation)` on failure.
    pub fn verify(&self) -> Result<()> {
        match self {
            Condition::MultiSig {
                threshold,
                signers,
                signatures,
            } => {
                let valid = signatures.len() >= *threshold
                    && signatures.iter().all(|sig| signers.contains(sig));
                if valid {
                    Ok(())
                } else {
                    Err(EscrowError::ConditionViolation)
                }
            }
            Condition::Preimage { hash, preimage } => {
                let computed = Sha256::digest(preimage);
                if computed.as_slice() == hash {
                    Ok(())
                } else {
                    Err(EscrowError::ConditionViolation)
                }
            }
        }
    }
}
