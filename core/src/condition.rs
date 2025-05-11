//! Escrow conditions and deterministic verification logic.

use ed25519_dalek::{Signature as Ed25519Sig, Verifier, VerifyingKey as Ed25519Pub};
use k256::ecdsa::{Signature as Secp256k1Sig, VerifyingKey as Secp256k1Pub};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::{EscrowError, Result};

/// Deterministic crypto conditions and fulfillments.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Condition {
    /// XRPL-style hashlock: SHA-256(preimage) == hash.
    Preimage { hash: [u8; 32], preimage: Vec<u8> },

    /// Ed25519 signature over a message.
    Ed25519 {
        public_key: [u8; 32],
        signature: Vec<u8>,
        message: Vec<u8>,
    },

    /// Secp256k1 signature over a message.
    Secp256k1 {
        public_key: Vec<u8>,
        signature: Vec<u8>,
        message: Vec<u8>,
    },

    /// Threshold SHA-256: at least `threshold` of `subconditions` must hold.
    Threshold {
        threshold: usize,
        subconditions: Vec<Condition>,
    },
}

impl Condition {
    /// Verify that specified fulfillments satisfy conditions.
    pub fn verify(&self) -> Result<()> {
        match self {
            Self::Preimage { hash, preimage } => {
                let computed = Sha256::digest(preimage);
                if computed.as_slice().ct_eq(hash).unwrap_u8() == 1 {
                    Ok(())
                } else {
                    Err(EscrowError::ConditionViolation)
                }
            }
            Self::Ed25519 {
                public_key,
                signature,
                message,
            } => {
                let pk = Ed25519Pub::from_bytes(public_key)
                    .map_err(|_| EscrowError::ConditionViolation)?;
                let sig = Ed25519Sig::from_slice(signature)
                    .map_err(|_| EscrowError::ConditionViolation)?;
                pk.verify(message, &sig)
                    .map_err(|_| EscrowError::ConditionViolation)
            }
            Self::Secp256k1 {
                public_key,
                signature,
                message,
            } => {
                let vk = Secp256k1Pub::from_sec1_bytes(public_key)
                    .map_err(|_| EscrowError::ConditionViolation)?;
                let sig = Secp256k1Sig::from_der(signature)
                    .map_err(|_| EscrowError::ConditionViolation)?;
                vk.verify(message, &sig)
                    .map_err(|_| EscrowError::ConditionViolation)
            }
            Self::Threshold {
                threshold,
                subconditions,
            } => {
                let mut valid = 0usize;
                for cond in subconditions.iter() {
                    if cond.verify().is_ok() {
                        valid += 1;
                    }
                }
                if valid >= *threshold {
                    Ok(())
                } else {
                    Err(EscrowError::ConditionViolation)
                }
            }
        }
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", json)
    }
}
