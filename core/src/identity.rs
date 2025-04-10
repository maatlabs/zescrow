use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Asset {
    Fungible { id: [u8; 32], amount: u64 },
    NonFungible { id: [u8; 32] },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Party {
    pub identity_hash: [u8; 32],
}
