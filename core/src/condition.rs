use bincode::{Decode, Encode};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::error::ConditionError;
use crate::Result;

/// Ed25519 signature over an arbitrary message.
pub mod ed25519;
/// XRPL-style hashlock: SHA-256(preimage) == hash.
pub mod hashlock;
/// Secp256k1 ECDSA signature over an arbitrary message.
pub mod secp256k1;
/// Threshold condition: at least `threshold` subconditions must hold.
pub mod threshold;

use ed25519::Ed25519;
use hashlock::Hashlock;
use secp256k1::Secp256k1;
use threshold::Threshold;

/// A cryptographic condition that can be deterministically verified.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(tag = "condition", content = "fulfillment", rename_all = "lowercase")
)]
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum Condition {
    /// XRPL-style hashlock: SHA-256(preimage) == hash.
    Hashlock(Hashlock),
    /// Ed25519 signature over an arbitrary message.
    Ed25519(Ed25519),
    /// Secp256k1 ECDSA signature over an arbitrary message.
    Secp256k1(Secp256k1),
    /// Threshold condition: at least `threshold` subconditions must hold.
    Threshold(Threshold),
}

impl Condition {
    /// Validate the provided witness data.
    ///
    /// # Errors
    ///
    /// Returns `EscrowError::Condition` under any of the following circumstances:
    /// - **Preimage**: SHA-256(preimage) does not match.
    /// - **Ed25519/Secp256k1**: public key parsing or signature parsing/verification fails.
    /// - **Threshold**: fewer than `threshold` subconditions succeed.
    #[inline]
    pub fn verify(&self) -> Result<()> {
        match self {
            Self::Hashlock(hashlock) => hashlock.verify().map_err(ConditionError::Hashlock)?,
            Self::Ed25519(ed25519) => ed25519.verify().map_err(ConditionError::Ed25519)?,
            Self::Secp256k1(secp256k1) => secp256k1.verify().map_err(ConditionError::Secp256k1)?,
            Self::Threshold(threshold) => threshold.verify().map_err(ConditionError::Threshold)?,
        }
        Ok(())
    }

    /// Construct a hashlock (preimage) condition.
    pub fn hashlock(hash: [u8; 32], preimage: Vec<u8>) -> Self {
        Self::Hashlock(Hashlock { hash, preimage })
    }

    /// Construct an Ed25519 signature condition.
    pub fn ed25519(public_key: [u8; 32], message: Vec<u8>, signature: Vec<u8>) -> Self {
        Self::Ed25519(Ed25519 {
            public_key,
            signature,
            message,
        })
    }

    /// Construct a Secp256k1 signature condition.
    pub fn secp256k1(public_key: Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> Self {
        Self::Secp256k1(Secp256k1 {
            public_key,
            signature,
            message,
        })
    }

    /// Construct a threshold condition.
    pub fn threshold(threshold: usize, subconditions: Vec<Self>) -> Self {
        Self::Threshold(Threshold {
            threshold,
            subconditions,
        })
    }
}

#[cfg(feature = "json")]
impl std::fmt::Display for Condition {
    /// Serialize the condition to compact JSON for logging or write formats.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{json}")
    }
}

#[cfg(test)]
mod tests {

    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn preimage() {
        let preimage = b"secret".to_vec();
        let hash = Sha256::digest(&preimage).into();
        let cond = Condition::hashlock(hash, preimage);
        assert!(cond.verify().is_ok());

        // invalid preimage
        let cond = Condition::hashlock(hash, b"wrong-secret".to_vec());
        assert!(cond.verify().is_err());
    }

    #[test]
    fn ed25519() {
        use ed25519_dalek::ed25519::signature::rand_core::OsRng;
        use ed25519_dalek::{Signer, SigningKey};

        let mut csprng = OsRng;
        let sk: SigningKey = SigningKey::generate(&mut csprng);

        let message = b"zkEscrow".to_vec();
        let signature = sk.sign(&message).to_bytes().to_vec();
        let public_key = sk.verifying_key().to_bytes();

        let cond = Condition::ed25519(public_key.clone(), message.clone(), signature.clone());
        assert!(cond.verify().is_ok());

        // tampered sig
        let mut signature = signature;
        signature[0] ^= 0xFF;

        let cond = Condition::ed25519(public_key, message, signature);
        assert!(cond.verify().is_err());
    }

    #[test]
    fn secp256k1() {
        use k256::ecdsa::signature::Signer;
        use k256::ecdsa::{Signature, SigningKey};
        use k256::elliptic_curve::rand_core::OsRng;

        let sk = SigningKey::random(&mut OsRng);
        let vk = sk.verifying_key();
        let message = b"zkEscrow".to_vec();
        let signature: Signature = sk.sign(&message);

        let sig_bytes = signature.to_der().as_bytes().to_vec();
        let pk_bytes = vk.to_encoded_point(false).as_bytes().to_vec();

        let cond = Condition::secp256k1(pk_bytes.clone(), message, sig_bytes.clone());
        assert!(cond.verify().is_ok());

        // tampered message
        let cond = Condition::secp256k1(pk_bytes, b"tampered".to_vec(), sig_bytes);
        assert!(cond.verify().is_err());
    }

    #[test]
    fn nonzero_threshold() {
        // two trivial subconditions: one succeeds, one fails
        let hash = Sha256::digest(b"zkEscrow").into();
        let correct = Condition::hashlock(hash, b"zkEscrow".to_vec());
        let wrong = Condition::hashlock(hash, b"wrong-preimage".to_vec());

        // threshold == 1 should pass
        let cond = Condition::threshold(1, vec![correct.clone(), wrong.clone()]);
        assert!(cond.verify().is_ok());

        // threshold == 2 should fail
        let cond = Condition::threshold(2, vec![correct, wrong]);
        assert!(cond.verify().is_err());

        // threshold == 1 and no subconditions should fail
        let cond = Condition::threshold(1, vec![]);
        assert!(cond.verify().is_err());
    }

    #[test]
    fn zero_threshold() {
        // threshold == 0 with empty subconditions should pass
        let cond = Condition::threshold(0, vec![]);
        assert!(cond.verify().is_ok());

        // threshold == 0 with subconditions should also pass
        let preimage = b"zkEscrow".to_vec();
        let hash = Sha256::digest(&preimage).into();
        let subcond = Condition::hashlock(hash, preimage);
        let cond = Condition::threshold(0, vec![subcond]);
        assert!(cond.verify().is_ok());
    }

    #[test]
    fn nested_thresholds() {
        let preimage = b"zkEscrow".to_vec();
        let hash = Sha256::digest(&preimage).into();
        let leaf = Condition::hashlock(hash, preimage);

        // inner threshold: need 1 of `leaf`
        let inner = Condition::threshold(1, vec![leaf.clone()]);
        // outer threshold: need 1 of `inner`
        let outer = Condition::threshold(1, vec![inner]);
        assert!(outer.verify().is_ok());

        // if `leaf` wrong, `inner` fails, and so does `outer`
        let wrong_leaf = Condition::hashlock(hash, b"wrong-preimage".to_vec());
        let inner2 = Condition::threshold(1, vec![wrong_leaf]);
        let outer2 = Condition::threshold(1, vec![inner2]);
        assert!(outer2.verify().is_err());
    }
}
