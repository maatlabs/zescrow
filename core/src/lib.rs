//! Zescrow Core Library
//!
//! # Overview
//!
//! `zescrow_core` is a pure, chain-agnostic library that provides the fundamental types
//! and logic for a zero-knowledge escrow system. This crate exposes:
//!
//! - **Asset** types (`asset.rs`): Chain-agnostic representations of native coins,
//!   fungible tokens, NFTs, multi-tokens, and liquidity pool shares, including validation
//!   and human-readable formatting.
//! - Cryptographic **Condition**s (`condition.rs`): Deterministic conditions (hashlocks,
//!   Ed25519/ECDSA signatures, threshold schemes) with fulfillment verification.
//! - **Escrow** state machine (`escrow.rs`): An off-chain escrow context for executing/verifying
//!   cryptographic conditions in zero-knowledge.
//! - Identity types (`identity.rs`): Chain-agnostic party identities with support for
//!   hex, Base58, Base64, or raw bytes, plus decoding and format conversions.
//! - Interface types (`interface.rs`): JSON (de)serialization helpers, parameter and
//!   metadata schemas (`EscrowParams`, `EscrowMetadata`), chain-specific configuration
//!   (`ChainConfig`, `ChainMetadata`), and utility functions for loading/saving JSON files.
//! - Error handling (`error.rs`): Comprehensive, well-structured error types covering
//!   identity parsing, asset validation, condition verification, parameter checks, and
//!   chain-specific operations.
//!
#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Chain-agnostic asset representations and utilities.
pub mod asset;

/// Deterministic cryptographic conditions and fulfillment logic.
pub mod condition;

/// Error types used throughout the `zescrow_core` crate.
pub mod error;

/// off-chain escrow context for executing/verifying cryptographic conditions in zero-knowledge.
pub mod escrow;

/// Chain-agnostic identity types (hex/Base58/Base64/raw bytes) for escrow parties.
pub mod identity;

/// Types for JSON (de)serialization, parameter/metadata schemas, and chain configurations.
pub mod interface;

pub use asset::Asset;
use bincode::de::{BorrowDecoder, Decoder};
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use bincode::{BorrowDecode, Decode, Encode};
pub use condition::Condition;
pub use error::EscrowError;
pub use escrow::Escrow;
pub use identity::Party;
pub use interface::{
    Chain, ChainConfig, ChainMetadata, EscrowMetadata, EscrowParams, ExecutionState,
};
use num_bigint::BigUint;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// `Result` type for all core operations, using [`EscrowError`] as the error.
pub type Result<T> = std::result::Result<T, EscrowError>;

/// Serde helper to (de)serialize BigUint as strings.
#[cfg(feature = "json")]
mod biguint_serde {
    use num_bigint::BigUint;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &BigUint, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&value.to_str_radix(10))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<BigUint, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        s.parse::<BigUint>().map_err(de::Error::custom)
    }
}

/// Serde helper to (de)serialize Vec<u8> as UTF-8 strings.
#[cfg(feature = "json")]
mod utf8_serde {
    use std::str;

    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = str::from_utf8(bytes).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(s.into_bytes())
    }
}

/// Wrapper around BigUint so we can implement bincode traits.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(transparent))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BigNumber(#[cfg_attr(feature = "json", serde(with = "biguint_serde"))] pub BigUint);

impl From<BigUint> for BigNumber {
    fn from(v: BigUint) -> Self {
        BigNumber(v)
    }
}

impl From<BigNumber> for BigUint {
    fn from(v: BigNumber) -> BigUint {
        v.0
    }
}

impl Encode for BigNumber {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> std::result::Result<(), EncodeError> {
        let s = self.0.to_str_radix(10);
        s.encode(encoder)
    }
}

impl<Context> Decode<Context> for BigNumber {
    fn decode<D: Decoder>(decoder: &mut D) -> std::result::Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        BigUint::parse_bytes(s.as_bytes(), 10)
            .map(BigNumber)
            .ok_or_else(|| DecodeError::OtherString("BigUint parse error".into()))
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for BigNumber {
    fn borrow_decode<D: BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, DecodeError> {
        Self::decode(decoder)
    }
}

impl std::fmt::Display for BigNumber {
    /// Print the inner BigUint as a decimal string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_str_radix(10))
    }
}

#[cfg(all(test, feature = "json"))]
mod serde_helpers_tests {
    use num_bigint::BigUint;
    use serde::{Deserialize, Serialize};

    use super::{biguint_serde, utf8_serde};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct BigUintWrapper(#[serde(with = "biguint_serde")] BigUint);

    #[test]
    fn biguint_valid_serde() {
        let orig = BigUint::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        let wrapped = BigUintWrapper(orig.clone());
        let ser = serde_json::to_string(&wrapped).unwrap();
        assert_eq!(ser, "\"123456789012345678901234567890\"");
        let de: BigUintWrapper = serde_json::from_str(&ser).unwrap();
        assert_eq!(de, wrapped);
    }

    #[test]
    fn biguint_invalid_serde() {
        let bad = "\"not_a_number\"";
        assert!(serde_json::from_str::<BigUintWrapper>(bad).is_err());
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Utf8Wrapper(#[serde(with = "utf8_serde")] Vec<u8>);

    #[test]
    fn utf8_valid_serde() {
        let orig = b"hello-zescrow".to_vec();
        let wrapped = Utf8Wrapper(orig.clone());
        let ser = serde_json::to_string(&wrapped).unwrap();
        assert_eq!(ser, "\"hello-zescrow\"");
        let de: Utf8Wrapper = serde_json::from_str(&ser).unwrap();
        assert_eq!(de, wrapped);
    }

    #[test]
    fn utf8_invalid_serde() {
        let bad = Utf8Wrapper(vec![0xff, 0xfe]);
        assert!(serde_json::to_string(&bad).is_err());
    }
}
