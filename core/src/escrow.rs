//! Escrow lifecycle and state transitions.

use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::error::EscrowError;
use crate::identity::{Asset, Party};

/// Represents the current state of the escrow.
///
/// State transitions:
///
/// ```text
/// Initialized → Funded → Completed
///             ↘      ↙
///             Disputed (Expired)
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EscrowState {
    Initialized,
    Funded,
    Completed,
    Expired,
}

/// Core escrow struct encapsulating the full escrow context.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Escrow {
    pub id: String, // Digest in hex string
    pub asset: Asset,
    pub beneficiary: Party,
    pub depositor: Party,
    pub condition: Condition,
    pub created_block: u64,
    pub expiry_block: u64,
    pub state: EscrowState,
}

impl Escrow {
    /// Executes escrow verification logic and state transitions.
    ///
    /// # Arguments
    /// - `current_block`: The current block height from the specified chain.
    pub fn execute(&mut self, current_block: Option<u64>) -> Result<Self, EscrowError> {
        if self.state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }

        self.condition.verify(current_block)?;
        self.state = EscrowState::Completed;

        Ok(self.clone())
    }
}
