//! Core types for assets and parties.

use serde::{Deserialize, Serialize};

/// A fungible or non-fungible asset in escrow.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Asset {
    /// Fungible: identified by `id` (e.g. token mint) and amount.
    Fungible { id: String, amount: u64 },
    /// Non-fungible: unique `id`.
    NonFungible { id: String },
}

/// A party participating in escrow.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Party {
    /// Some identifier (e.g. pubkey hash, address).
    pub identity_hash: String,
}
