//! Core types for assets and parties participating in an escrow.

use serde::{Deserialize, Serialize};

/// Represents an asset locked in escrow.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Asset {
    /// Fungible asset with a unique identifier (digest in hex string)
    /// and specific amount.
    Fungible { id: String, amount: u64 },
    /// Non-fungible asset identified uniquely by an ID
    /// (digest in hex string).
    NonFungible { id: String },
}

/// Represents a party (participant) in an escrow transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Party {
    /// Cryptographic hash (hex string)
    /// representing the party's identity.
    pub identity_hash: String,
}
