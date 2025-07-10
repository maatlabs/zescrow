use bincode::{Decode, Encode};
#[cfg(feature = "json")]
use hex::serde as hex_serde;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

#[cfg(feature = "json")]
use crate::serde::utf8_serde;

/// A hashlock condition.
///
/// A `preimage` must be provided such that
/// `SHA256(preimage) == hash`.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Hashlock {
    /// The expected SHA-256 digest of the preimage.
    #[cfg_attr(feature = "json", serde(with = "hex_serde"))]
    pub hash: [u8; 32],

    /// Secret preimage as UTF-8 string.
    #[cfg_attr(feature = "json", serde(with = "utf8_serde"))]
    pub preimage: Vec<u8>,
}

impl Hashlock {
    /// Compute SHA256(preimage) and compare to `hash`.
    pub fn verify(&self) -> Result<(), Error> {
        let computed = Sha256::digest(&self.preimage);
        if computed.as_slice().ct_eq(&self.hash).unwrap_u8() == 1 {
            Ok(())
        } else {
            Err(Error::Mismatch)
        }
    }
}

/// Errors from hashlock verification.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The provided preimage did not hash to the expected value.
    #[error("SHA256(preimage) != hash")]
    Mismatch,
}
