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
    Hex(String),
    Base58(String),
    Base64(String),
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
    pub fn new<S: AsRef<str>>(id_str: S) -> Result<Self> {
        let identity = ID::from_str(id_str.as_ref())?;
        Ok(Self { identity })
    }

    /// Verifies that the underlying [`ID`] is well-formed by attempting
    /// to decode it into raw bytes.
    ///
    /// This uses `ID::to_bytes()` internally, so you’ll get back exactly
    /// the same decoding errors (`Hex`, `Base58`, or `Base64` failures)
    /// as if you had called it yourself.
    ///
    /// # Errors
    ///
    /// - `Err(EscrowError::Identity(_))` if decoding fails.
    pub fn verify_identity(&self) -> Result<()> {
        let _ = self.identity.to_bytes()?;
        Ok(())
    }
}

impl std::fmt::Display for Party {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identity)
    }
}

impl std::str::FromStr for Party {
    type Err = EscrowError;

    /// Parses an instance of `Self` from a string, alias for [`Self::new`].
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl ID {
    /// Decode this `ID` into its raw byte representation.
    ///
    /// Depending on the variant:
    /// - **Hex**: decodes the lowercase hex string (e.g. `"0xdeadbeef"`) into bytes.
    /// - **Base58**: decodes the Base58 string into bytes.
    /// - **Base64**: decodes the Base64 string into bytes.
    /// - **Bytes**: simply clones and returns the inner `Vec<u8>`.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the decoded bytes.
    ///
    /// # Errors
    ///
    /// An `IdentityError` corresponding to the failing ID type.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(match self {
            Self::Hex(s) => hex::decode(s).map_err(IdentityError::Hex)?,
            Self::Base58(s) => bs58::decode(s).into_vec().map_err(IdentityError::Base58)?,
            Self::Base64(s) => BASE64_STANDARD.decode(s).map_err(IdentityError::Base64)?,
            Self::Bytes(b) => b.clone(),
        })
    }

    pub fn to_hex(&self) -> Result<String> {
        Ok(hex::encode(self.to_bytes()?))
    }

    pub fn to_base58(&self) -> Result<String> {
        Ok(bs58::encode(self.to_bytes()?).into_string())
    }

    pub fn to_base64(&self) -> Result<String> {
        Ok(BASE64_STANDARD.encode(self.to_bytes()?))
    }
}

impl std::fmt::Display for ID {
    /// Show each variant as its "natural" string form,
    /// interpreting arbitrary bytes as base64, since that round-trips safely.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex(s) => write!(f, "{}", s),
            Self::Base58(s) => f.write_str(s),
            Self::Base64(s) => f.write_str(s),
            Self::Bytes(b) => f.write_str(&BASE64_STANDARD.encode(b)),
        }
    }
}

impl std::str::FromStr for ID {
    type Err = EscrowError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(IdentityError::EmptyIdentity.into());
        }
        // strip optional "0x" for hex-encoded IDs
        let raw = s.strip_prefix("0x").unwrap_or(s);

        // try hex decoding
        if let Ok(bytes) = hex::decode(raw) {
            return Ok(Self::Hex(hex::encode(bytes)));
        }
        // try base58 decoding
        if let Ok(bytes) = bs58::decode(raw).into_vec() {
            return Ok(Self::Base58(bs58::encode(bytes).into_string()));
        }
        // try base64 decoding
        if let Ok(bytes) = BASE64_STANDARD.decode(raw) {
            return Ok(Self::Base64(BASE64_STANDARD.encode(bytes)));
        }

        Err(IdentityError::UnsupportedFormat.into())
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr as _;

    use super::*;

    #[test]
    fn parse_and_display_id() {
        // hex (with 0x prefix)
        let id_hex = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        let id_hex_no_prefix = "d8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        let id = id_hex.parse::<ID>().unwrap();
        assert_eq!(id.to_hex().unwrap(), id_hex_no_prefix.to_lowercase());
        assert_eq!(id.to_string(), id_hex.to_lowercase());

        // base58 (no prefix)
        let id_bs58 = "9Ah3Yf4Q82n4RHmoyR4kQc8acbu7UbHDy3coc1QqVvRF";
        let id = id_bs58.parse::<ID>().unwrap();
        assert_eq!(id.to_base58().unwrap(), id_bs58);
        assert_eq!(id.to_string(), id_bs58);

        // base64 (no prefix) case 1
        let id_bs64 = "YWJjMTIzIT8kKiYoKSctPUB+";
        let id = id_bs64.parse::<ID>().unwrap();
        assert_eq!(id.to_base64().unwrap(), id_bs64);
        assert_eq!(id.to_string(), id_bs64);

        // parse raw bytes as ID
        let raw = vec![1, 2, 3, 4];
        let raw_bs64 = BASE64_STANDARD.encode(&raw);
        let id = raw_bs64.parse::<ID>().unwrap();
        assert_eq!(id.to_bytes().unwrap(), raw);
        assert_eq!(id.to_string(), raw_bs64);

        // display party
        let party = Party::from_str("0xdeadbeef").unwrap();
        assert_eq!(party.to_string(), "Party(0xdeadbeef)");

        // unsupported format
        assert!("no-prefix".parse::<ID>().is_err());
    }
}
