use bincode::{Decode, Encode};
#[cfg(feature = "json")]
use hex::serde as hex_serde;
use k256::ecdsa::signature::Verifier;
use k256::ecdsa::{Signature, VerifyingKey};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Secp256k1 ECDSA signature condition.
///
/// Verifies that `signature` is a valid ECDSA signature of `message`
/// under `public_key`.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Secp256k1 {
    /// Compressed SEC1-encoded public key bytes.
    #[cfg_attr(feature = "json", serde(with = "hex_serde"))]
    pub public_key: Vec<u8>,

    /// DER-encoded signature bytes.
    #[cfg_attr(feature = "json", serde(with = "hex_serde"))]
    pub signature: Vec<u8>,

    /// Original message bytes.
    #[cfg_attr(feature = "json", serde(with = "hex_serde"))]
    pub message: Vec<u8>,
}

impl Secp256k1 {
    /// Verify that `signature` is a valid Secp256k1 signature of `message` under `public_key`.
    pub fn verify(&self) -> Result<(), Error> {
        let vk = VerifyingKey::from_sec1_bytes(&self.public_key)
            .map_err(|e| Error::InvalidPublicKey(e.to_string()))?;
        let sig = Signature::from_der(&self.signature).map_err(Error::InvalidSignature)?;
        vk.verify(&self.message, &sig)
            .map_err(|_| Error::VerificationFailed)
    }
}

/// Errors from Secp256k1 signature verification.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error decoding public key
    #[error("public key decoding error: {0}")]
    InvalidPublicKey(String),

    /// Error decoding signature
    #[error("signature decoding error: {0}")]
    InvalidSignature(#[from] k256::ecdsa::Error),

    /// Error verifying signature
    #[error("signature verification failed")]
    VerificationFailed,
}
