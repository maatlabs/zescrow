//! Escrow state machine with time locks and optional crypto conditions.

use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::identity::{Asset, Party};
use crate::{EscrowError, Result};

/// Where in the lifecycle an escrow is
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum EscrowState {
    Funded,
    Released,
    Expired,
}

/// Full escrow context
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Escrow {
    /// Asset locked in escrow.
    pub asset: Asset,
    /// Recipient of funds.
    pub recipient: Party,
    /// Who funded it.
    pub sender: Party,
    /// Optional cryptographic condition.
    pub condition: Option<Condition>,
    /// Block height when escrow was created.
    pub created_block: u64,
    /// Current state.
    pub state: EscrowState,
}

impl Escrow {
    /// Attempts to finish (release) by verifying the predefined conditions.
    pub fn execute(&mut self) -> Result<EscrowState> {
        if self.state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }
        if let Some(cond) = &self.condition {
            cond.verify()?;
        }
        self.state = EscrowState::Released;
        Ok(self.state)
    }
}
