#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(rustdoc::invalid_html_tags, rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

/// Chain-agnostic asset representations and utilities.
pub mod asset;

/// Wrapper around [BigUint] so we can implement [bincode] traits.
pub mod bignum;

/// Deterministic cryptographic conditions and fulfillment logic.
pub mod condition;

/// Error types used throughout the `zescrow_core` crate.
pub mod error;

/// Off-chain escrow context for executing/verifying cryptographic conditions in zero-knowledge.
pub mod escrow;

/// Chain-agnostic identity types (hex/Base58/Base64/raw bytes) for escrow parties.
pub mod identity;

/// Types for JSON (de)serialization, parameter/metadata schemas, and chain configurations.
pub mod interface;

/// Helpers for (de)serializing with [serde].
#[cfg(feature = "json")]
pub mod serde;

pub use asset::{Asset, AssetKind};
pub use bignum::BigNumber;
pub use condition::Condition;
pub use error::EscrowError;
pub use escrow::Escrow;
pub use identity::{Party, ID};
pub use interface::{Chain, ChainConfig, EscrowMetadata, EscrowParams, ExecutionState};

/// `Result` type for all core operations.
pub type Result<T> = std::result::Result<T, EscrowError>;
