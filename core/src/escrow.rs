use serde::{Deserialize, Serialize};

use crate::{
    condition::Condition,
    error::EscrowError,
    identity::{Asset, Party},
};

/// Escrow state transitions:
///
/// ```text
/// Initialized → Funded → Completed
///             ↘      ↙
///             Disputed
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EscrowState {
    Initialized,
    Funded,
    Completed,
    Expired,
}

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

impl Escrow {
    pub fn execute(&mut self) -> Result<Self, EscrowError> {
        if self.state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }
        self.condition.verify()?;
        self.state = EscrowState::Completed;

        Ok(self.clone())
    }
}
