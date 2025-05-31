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
pub use condition::Condition;
pub use error::EscrowError;
pub use escrow::Escrow;
pub use identity::Party;
pub use interface::{Chain, ChainConfig, ChainMetadata, EscrowMetadata, EscrowParams, EscrowState};

/// `Result` type for all core operations, using [`EscrowError`] as the error.
pub type Result<T> = std::result::Result<T, EscrowError>;
