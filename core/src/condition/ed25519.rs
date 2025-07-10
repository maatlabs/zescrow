use bincode::{Decode, Encode};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
#[cfg(feature = "json")]
use hex::serde as hex_serde;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Ed25519 condition: signature over an arbitrary message.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Ed25519 {
    /// Public key bytes
    #[cfg_attr(feature = "json", serde(with = "hex_serde"))]
    pub public_key: [u8; 32],

    /// Signature bytes
    #[cfg_attr(feature = "json", serde(with = "hex_serde"))]
    pub signature: Vec<u8>,

    /// Original message bytes
    #[cfg_attr(feature = "json", serde(with = "hex_serde"))]
    pub message: Vec<u8>,
}

impl Ed25519 {
    /// Verify that `signature` is a valid Ed25519 signature of `message` under `public_key`.
    pub fn verify(&self) -> Result<(), Error> {
        let pk = VerifyingKey::from_bytes(&self.public_key)
            .map_err(|e| Error::InvalidPublicKey(e.to_string()))?;
        let sig = Signature::from_slice(&self.signature).map_err(Error::InvalidSignature)?;
        pk.verify(&self.message, &sig)
            .map_err(|_| Error::VerificationFailed)
    }
}

/// Errors from ed25519 signature verification.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error decoding public key
    #[error("public key decoding error: {0}")]
    InvalidPublicKey(String),

    /// Error decoding signature
    #[error("signature decoding error: {0}")]
    InvalidSignature(#[from] ed25519_dalek::SignatureError),

    /// Error verifying signature
    #[error("signature verification failed")]
    VerificationFailed,
}
