//! Core types for assets and parties participating in an escrow.

use serde::{Deserialize, Serialize};

/// Represents an asset locked in escrow.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Asset {
    /// Fungible asset with a unique identifier and specific amount.
    Fungible { id: [u8; 32], amount: u64 },
    /// Non-fungible asset identified uniquely by an ID.
    NonFungible { id: [u8; 32] },
}

/// Represents a party (participant) in an escrow transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Party {
    /// Cryptographic hash representing the party's identity.
    pub identity_hash: [u8; 32],
}
