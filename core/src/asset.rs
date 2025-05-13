use serde::{Deserialize, Serialize};

/// A fungible or non-fungible asset in escrow.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Asset {
    /// Fungible: identified by `id` (e.g. token mint) and amount.
    Fungible { id: String, amount: u64 },
    /// Non-fungible: unique `id`.
    NonFungible { id: String },
}
