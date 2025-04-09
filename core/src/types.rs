use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Escrow {
    pub id: [u8; 32],
    pub asset: Asset,
    pub beneficiary: Party,
    pub depositor: Party,
    pub condition: Condition,
    pub created_block: u64,
    pub expiry_block: u64,
    pub state: EscrowState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Asset {
    Fungible { amount: u64 },
    NonFungible { id: [u8; 32] },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Party {
    pub identity_hash: [u8; 32],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Condition {
    MultiSig {
        threshold: u8,
        signers: Vec<[u8; 32]>,
    },
    TimeLock {
        expiry_block: u64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EscrowState {
    Initialized,
    Funded,
    Completed,
    Expired,
}

#[derive(Debug, thiserror::Error)]
pub enum EscrowError {
    #[error("Condition not satisfied")]
    ConditionViolation,
    #[error("Invalid state transition")]
    InvalidState,
    #[error("Timeout expired")]
    Expired,
}
