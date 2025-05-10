//! Escrow conditions and deterministic verification logic.

use ed25519_dalek::{Signature as Ed25519Sig, Verifier, VerifyingKey as Ed25519Pub};
use k256::ecdsa::{
    signature::Verifier as SecpVerify, Signature as Secp256k1Sig, VerifyingKey as Secp256k1Pub,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::{EscrowError, Result};

/// Deterministic crypto condition fingerprint.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Condition {
    /// XRPL-style hashlock: SHA-256(preimage) == hash.
    Preimage { hash: String },

    /// Ed25519 signature over a message.
    Ed25519 {
        public_key: String,
        signature: String,
        message: String,
    },

    /// Secp256k1 signature over a message.
    Secp256k1 {
        public_key: String,
        signature: String,
        message: String,
    },

    /// Threshold SHA-256: at least `threshold` of `subconditions` must hold.
    Threshold {
        threshold: usize,
        subconditions: Vec<Condition>,
    },
}

/// The data proving a condition.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Fulfillment {
    Preimage { preimage: String },
    Ed25519 { signature: String, message: String },
    Secp256k1 { signature: String, message: String },
    Threshold { subfulfillments: Vec<Fulfillment> },
}

impl Condition {
    /// Verify the condition; returns `Err(EscrowError::ConditionViolation)` on failure.
    pub fn verify(&self) -> Result<()> {
        todo!()
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", json)
    }
}
