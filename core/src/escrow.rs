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
    /// Earliest block at which `finish` is allowed.
    pub finish_after: Option<u64>,
    /// Earliest block at which `refund` is allowed.
    pub cancel_after: Option<u64>,
    /// Current state.
    pub state: EscrowState,
}

impl Escrow {
    /// Attempt to finish (release) the escrow.
    /// Checks `finish_after` and then any crypto `condition`.
    pub fn execute(&mut self, current_block: u64) -> Result<EscrowState> {
        if self.state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }
        if let Some(ts) = self.finish_after {
            if current_block < ts {
                return Err(EscrowError::NotReady);
            }
        }
        if let Some(cond) = &self.condition {
            cond.verify()?;
        }
        self.state = EscrowState::Released;
        Ok(self.state)
    }

    /// Refund the depositor after `cancel_after`.
    pub fn refund(&mut self, current_block: u64) -> Result<()> {
        if self.state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }
        if let Some(ts) = self.cancel_after {
            if current_block < ts {
                return Err(EscrowError::NotExpired);
            }
        }
        self.state = EscrowState::Expired;
        Ok(())
    }
}
