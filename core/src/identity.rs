//! Chain-agnostic identity types for escrow participants.

use core::str::FromStr as _;

use base64::prelude::*;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::error::IdentityError;
use crate::{EscrowError, Result};

/// A participant in the escrow protocol, wrapping a chain-agnostic [`ID`].
///
/// A `Party` represents an on-chain account or public-key identity.  
/// Internally it holds an [`ID`], which may have been encoded as hex, Base58, Base64,
/// or raw bytes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Party {
    /// The participant’s on-chain identity.
    pub identity: ID,
}

/// Supported encoding formats for on-chain identities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "encoding", content = "value", rename_all = "lowercase")]
pub enum ID {
    /// Hex-encoded string.
    Hex(String),
    /// Base58-encoded string.
    Base58(String),
    /// Base64-encoded string.
    Base64(String),
    /// Raw bytes.
    #[serde(with = "serde_bytes")]
    Bytes(Vec<u8>),
}

impl Party {
    /// Parses a `Party` from a string-encoded identity.
    ///
    /// The input is a string-encoded id in any of the supported formats:
    /// - **Hex** (with or without `0x` prefix),
    /// - **Base58**,
    /// - **Base64**,
    /// - or direct raw bytes (`ID::Bytes(Vec<u8>)`).
    ///
    /// # Errors
    ///
    /// Returns `Err(EscrowError::Identity(_))` if the input is empty or cannot be
    /// decoded into a valid byte sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zescrow_core::Party;
    /// let party = Party::new("0xdeadbeef").unwrap();
    /// assert_eq!(party.to_string(), "deadbeef");
    /// ```
    pub fn new<S: AsRef<str>>(id_str: S) -> Result<Self> {
        let identity = ID::from_str(id_str.as_ref())?;
        Ok(Self { identity })
    }

    /// Verifies that the underlying [`ID`] can be decoded into raw bytes.
    ///
    /// # Errors
    ///
    /// - `Err(EscrowError::Identity(_))` if decoding fails.
    pub fn verify_identity(&self) -> Result<()> {
        self.identity.validate()
    }
}

impl std::str::FromStr for Party {
    type Err = EscrowError;

    /// Parses an instance of `Self` from a string, alias for [`Self::new`].
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Party::new(s)
    }
}

impl std::fmt::Display for Party {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identity)
    }
}

impl ID {
    const HEX: &'static str = "hex";
    const BASE58: &'static str = "base58";
    const BASE64: &'static str = "base64";
    const BYTES: &'static str = "bytes";

    /// Verifies that self can be decoded into raw bytes, and that it's not empty.
    ///
    /// # Errors
    ///
    /// - `Err(EscrowError::Identity(_))` if decoding fails, or is empty.
    pub fn validate(&self) -> Result<()> {
        // self.to_bytes()?
        //     .is_empty()
        //     .then(|| ())
        //     .ok_or_else(|| IdentityError::EmptyIdentity.into())
        let id_bytes = self.to_bytes()?;
        if id_bytes.is_empty() {
            return Err(IdentityError::EmptyIdentity.into());
        }
        Ok(())
    }

    /// Decode this `ID` into its raw byte representation.
    ///
    /// Depending on the variant:
    /// - **Hex**: decodes the lowercase hex string (e.g. `"0xdeadbeef"`) into bytes.
    /// - **Base58**: decodes the Base58 string into bytes.
    /// - **Base64**: decodes the Base64 string into bytes.
    /// - **Bytes**: clones and returns the inner `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// An `IdentityError` corresponding to the failing ID type.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let decoded = match self {
            Self::Hex(s) => hex::decode(s).map_err(IdentityError::Hex),
            Self::Base58(s) => bs58::decode(s).into_vec().map_err(IdentityError::Base58),
            Self::Base64(s) => BASE64_STANDARD.decode(s).map_err(IdentityError::Base64),
            Self::Bytes(b) => Ok(b.clone()),
        }?;
        Ok(decoded)
    }

    /// Returns the hex string representation of the identity.
    ///
    /// # Errors
    ///
    /// Returns an `EscrowError::Identity` if the underlying bytes cannot be obtained.
    pub fn to_hex(&self) -> Result<String> {
        let bytes = self.to_bytes()?;
        Ok(hex::encode(bytes))
    }

    /// Returns the Base58 string representation of the identity.
    pub fn to_base58(&self) -> Result<String> {
        let bytes = self.to_bytes()?;
        Ok(bs58::encode(bytes).into_string())
    }

    /// Returns the Base64 string representation of the identity.
    pub fn to_base64(&self) -> Result<String> {
        let bytes = self.to_bytes()?;
        Ok(BASE64_STANDARD.encode(bytes))
    }

    /// Returns the encoding variant as a `&'static str`.
    pub fn encoding(&self) -> &'static str {
        match self {
            Self::Hex(_) => Self::HEX,
            Self::Base58(_) => Self::BASE58,
            Self::Base64(_) => Self::BASE64,
            Self::Bytes(_) => Self::BYTES,
        }
    }
}

impl std::fmt::Display for ID {
    /// Returns the canonical string representation of this `ID`.
    ///
    /// - **Hex**: lowercase hex string without prefix.
    /// - **Base58**: canonical Base58 string.
    /// - **Base64**: standard Base64 string.
    /// - **Bytes**: standard Base64 string of bytes.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex(s) => write!(f, "{}", s),
            Self::Base58(s) => write!(f, "{}", s),
            Self::Base64(s) => write!(f, "{}", s),
            Self::Bytes(b) => write!(f, "{}", BASE64_STANDARD.encode(b)),
        }
    }
}

impl std::str::FromStr for ID {
    type Err = EscrowError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(IdentityError::EmptyIdentity.into());
        }
        let raw = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
            .unwrap_or(trimmed);

        // try hex decoding
        if let Ok(bytes) = hex::decode(raw) {
            return Ok(ID::Hex(hex::encode(bytes)));
        }
        // try base58 decoding
        if let Ok(bytes) = bs58::decode(raw).into_vec() {
            return Ok(ID::Base58(bs58::encode(bytes).into_string()));
        }
        // try base64 decoding
        if let Ok(bytes) = BASE64_STANDARD.decode(raw) {
            return Ok(ID::Base64(BASE64_STANDARD.encode(bytes)));
        }

        Err(IdentityError::UnsupportedFormat.into())
    }
}

impl From<Vec<u8>> for ID {
    fn from(bytes: Vec<u8>) -> Self {
        ID::Bytes(bytes)
    }
}

impl From<&[u8]> for ID {
    fn from(bytes: &[u8]) -> Self {
        ID::Bytes(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn hex_identity() {
        let id_str = "deadbeef";
        let id = ID::from_str(id_str).unwrap();
        assert_eq!(id, ID::Hex("deadbeef".into()));
        assert_eq!(id.to_bytes().unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(id.to_hex().unwrap(), id_str);
        assert_eq!(id.encoding(), "hex");
    }

    #[test]
    fn hex_with_prefix() {
        let id_str = "0XDEADBEEF";
        let id = ID::from_str(id_str).unwrap();
        assert_eq!(id, ID::Hex("deadbeef".into()));
    }

    #[test]
    fn base58_identity() {
        let raw = vec![1, 2, 3, 4];
        let b58_str = bs58::encode(&raw).into_string();
        let id = ID::from_str(&b58_str).unwrap();
        assert_eq!(id, ID::Base58(b58_str.clone()));
        assert_eq!(id.to_base58().unwrap(), b58_str);
        assert_eq!(id.encoding(), "base58");
    }

    #[test]
    fn base64_identity() {
        let raw = vec![1, 2, 3, 4];
        let b64 = BASE64_STANDARD.encode(&raw);
        let id = ID::from_str(&b64).unwrap();
        assert_eq!(id, ID::Base64(b64.clone()));
        assert_eq!(id.to_base64().unwrap(), b64);
        assert_eq!(id.encoding(), "base64");
    }

    #[test]
    fn bytes_identity() {
        let raw = vec![9, 8, 7];
        let id: ID = raw.clone().into();
        assert_eq!(id, ID::Bytes(raw.clone()));
        assert_eq!(id.to_bytes().unwrap(), raw);
        assert_eq!(id.to_string(), BASE64_STANDARD.encode(&raw));
        assert_eq!(id.encoding(), "bytes");
    }

    #[test]
    fn verify_identity() {
        let party = Party::new("0xdeadbeef").unwrap();
        assert_eq!(party.to_string(), "deadbeef");
        assert!(party.verify_identity().is_ok());
    }

    #[test]
    fn invalid_identity() {
        assert!(ID::from_str("not a valid ID").is_err());
    }
}
