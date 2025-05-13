//! Chain-agnostic identity types for escrow participants.

use base64::prelude::*;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::error::IdentityError;
use crate::{EscrowError, Result};

/// Supported encoding formats for on-chain identities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "encoding", content = "value")]
pub enum ID {
    Hex(String),
    Base58(String),
    Base64(String),
    #[serde(with = "serde_bytes")]
    Bytes(Vec<u8>),
}

impl ID {
    pub fn to_hex(&self) -> Result<String> {
        Ok(hex::encode(self.to_bytes()?))
    }

    pub fn to_base58(&self) -> Result<String> {
        Ok(bs58::encode(self.to_bytes()?).into_string())
    }

    pub fn to_base64(&self) -> Result<String> {
        Ok(BASE64_STANDARD.encode(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(match self {
            Self::Hex(s) => hex::decode(s).map_err(IdentityError::Hex)?,
            Self::Base58(s) => bs58::decode(s).into_vec().map_err(IdentityError::Base58)?,
            Self::Base64(s) => BASE64_STANDARD.decode(s).map_err(IdentityError::Base64)?,
            Self::Bytes(b) => b.clone(),
        })
    }
}

impl std::fmt::Display for ID {
    /// Show each variant as its "natural" string form:
    /// - Hex: with `0x` prefix so it's still obvious
    /// - Base58: raw
    /// - Base64: raw
    /// - Bytes: interpret arbitrary bytes as base64, since that round-trips safely
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex(s) => write!(f, "0x{}", s),
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

/// A party in the escrow protocol,
/// wrapping a chain-agnostic `ID`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Party {
    /// The participant's on-chain identity (address or public key)
    pub identity: ID,
}

impl Party {
    /// Construct from any `ID`.
    pub fn new(identity: ID) -> Self {
        Self { identity }
    }
}

impl std::fmt::Display for Party {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Party({})", self.identity)
    }
}

impl std::str::FromStr for Party {
    type Err = EscrowError;

    /// Create a `Party` by parsing a typed string (e.g., `"0xdeadbeef"`).
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::new(ID::from_str(s)?))
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
