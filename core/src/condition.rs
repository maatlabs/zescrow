//! Escrow conditions and deterministic verification logic.

use ed25519_dalek::{Signature as Ed25519Sig, Verifier, VerifyingKey as Ed25519Pub};
use k256::ecdsa::{Signature as Secp256k1Sig, VerifyingKey as Secp256k1Pub};
use serde::{Deserialize, Serialize};
use serde_with::hex::Hex;
use serde_with::serde_as;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::error::ConditionError;
use crate::Result;

/// Deterministic crypto conditions and fulfillments.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "condition_type", content = "data", rename_all = "lowercase")]
pub enum Condition {
    /// XRPL-style hashlock: SHA-256(preimage) == hash.
    Preimage {
        #[serde_as(as = "Hex")]
        hash: [u8; 32],
        #[serde_as(as = "Hex")]
        preimage: Vec<u8>,
    },
    /// Ed25519 signature over a message.
    Ed25519 {
        #[serde_as(as = "Hex")]
        public_key: [u8; 32],
        #[serde_as(as = "Hex")]
        signature: Vec<u8>,
        #[serde_as(as = "Hex")]
        message: Vec<u8>,
    },
    /// Secp256k1 signature over a message.
    Secp256k1 {
        #[serde_as(as = "Hex")]
        public_key: Vec<u8>,
        #[serde_as(as = "Hex")]
        signature: Vec<u8>,
        #[serde_as(as = "Hex")]
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
                    Err(ConditionError::PreimageMismatch.into())
                }
            }
            Self::Ed25519 {
                public_key,
                signature,
                message,
            } => {
                let pk = Ed25519Pub::from_bytes(public_key)
                    .map_err(ConditionError::PubkeyOrSigVerification)?;
                let sig = Ed25519Sig::from_slice(signature)
                    .map_err(ConditionError::PubkeyOrSigVerification)?;
                pk.verify(message, &sig)
                    .map_err(|e| ConditionError::PubkeyOrSigVerification(e).into())
            }
            Self::Secp256k1 {
                public_key,
                signature,
                message,
            } => {
                let vk = Secp256k1Pub::from_sec1_bytes(public_key)
                    .map_err(ConditionError::PubkeyOrSigVerification)?;
                let sig = Secp256k1Sig::from_der(signature)
                    .map_err(ConditionError::PubkeyOrSigVerification)?;
                vk.verify(message, &sig)
                    .map_err(|e| ConditionError::PubkeyOrSigVerification(e).into())
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
                    Err(ConditionError::ThresholdNotMet {
                        threshold: *threshold,
                        valid,
                    }
                    .into())
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn preimage() {
        let preimage = b"secret".to_vec();
        let hash = Sha256::digest(&preimage).into();
        let cond = Condition::Preimage { hash, preimage };
        assert!(cond.verify().is_ok());

        // invalid preimage
        let cond = Condition::Preimage {
            hash,
            preimage: b"wrong-secret".to_vec(),
        };
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

        let cond = Condition::Ed25519 {
            public_key: public_key.clone(),
            signature: signature.clone(),
            message: message.clone(),
        };
        assert!(cond.verify().is_ok());

        // tampered sig
        let mut signature = signature;
        signature[0] ^= 0xFF;
        let cond = Condition::Ed25519 {
            public_key,
            signature,
            message,
        };
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

        let cond = Condition::Secp256k1 {
            public_key: pk_bytes.clone(),
            signature: sig_bytes.clone(),
            message,
        };
        assert!(cond.verify().is_ok());

        // tampered message
        let cond = Condition::Secp256k1 {
            public_key: pk_bytes,
            signature: sig_bytes,
            message: b"tampered".to_vec(),
        };
        assert!(cond.verify().is_err());
    }

    #[test]
    fn nonzero_threshold() {
        // two trivial subconditions: one succeeds, one fails
        let hash = Sha256::digest(b"zkEscrow").into();
        let correct = Condition::Preimage {
            hash,
            preimage: b"zkEscrow".to_vec(),
        };
        let wrong = Condition::Preimage {
            hash,
            preimage: b"wrong-preimage".to_vec(),
        };

        // threshold == 1 should pass
        let cond = Condition::Threshold {
            threshold: 1,
            subconditions: vec![correct.clone(), wrong.clone()],
        };
        assert!(cond.verify().is_ok());

        // threshold == 2 should fail
        let cond = Condition::Threshold {
            threshold: 2,
            subconditions: vec![correct, wrong],
        };
        assert!(cond.verify().is_err());

        // threshold == 1 and no subconditions should fail
        let cond = Condition::Threshold {
            threshold: 1,
            subconditions: vec![],
        };
        assert!(cond.verify().is_err());
    }

    #[test]
    fn zero_threshold() {
        // threshold == 0 with empty subconditions should pass
        let cond = Condition::Threshold {
            threshold: 0,
            subconditions: vec![],
        };
        assert!(cond.verify().is_ok());

        // threshold == 0 with subconditions should also pass
        let preimage = b"zkEscrow".to_vec();
        let hash = Sha256::digest(&preimage).into();
        let subcond = Condition::Preimage { hash, preimage };
        let cond = Condition::Threshold {
            threshold: 0,
            subconditions: vec![subcond],
        };
        assert!(cond.verify().is_ok());
    }

    #[test]
    fn nested_thresholds() {
        let preimage = b"zkEscrow".to_vec();
        let hash = Sha256::digest(&preimage).into();
        let leaf = Condition::Preimage { hash, preimage };

        // inner threshold: need 1 of `leaf`
        let inner = Condition::Threshold {
            threshold: 1,
            subconditions: vec![leaf.clone()],
        };
        // outer threshold: need 1 of `inner`
        let outer = Condition::Threshold {
            threshold: 1,
            subconditions: vec![inner],
        };
        assert!(outer.verify().is_ok());

        // if `leaf` wrong, `inner` fails, and so does `outer`
        let wrong_leaf = Condition::Preimage {
            hash,
            preimage: b"wrong-preimage".to_vec(),
        };
        let inner2 = Condition::Threshold {
            threshold: 1,
            subconditions: vec![wrong_leaf],
        };
        let outer2 = Condition::Threshold {
            threshold: 1,
            subconditions: vec![inner2],
        };
        assert!(outer2.verify().is_err());
    }
}
