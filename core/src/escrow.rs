//! Escrow state machine with time locks and optional crypto conditions.

use serde::{Deserialize, Serialize};

use crate::{Asset, Condition, EscrowError, Party, Result};

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

#[cfg(test)]
mod tests {

    use core::str::FromStr as _;

    use super::*;
    use crate::utils::assert_err;

    #[test]
    fn end_to_end() {
        let sender = Party::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let recipient = Party::from_str("0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8").unwrap();

        // funded -> released (no condition)
        let mut escrow = Escrow {
            asset: Asset::Fungible {
                id: "test-token".to_string(),
                amount: 10,
            },
            recipient,
            sender,
            condition: None,
            created_block: 0,
            state: EscrowState::Funded,
        };
        assert_eq!(escrow.execute().unwrap(), EscrowState::Released);
        assert_eq!(escrow.state, EscrowState::Released);

        // executing again should result in invalid state
        assert_err(escrow.execute(), EscrowError::InvalidState);

        // expired escrow cannot be executed (re-released)
        escrow.state = EscrowState::Expired;
        assert_err(escrow.execute(), EscrowError::InvalidState);

        // fund with failing condition
        let mut bad_escrow = Escrow {
            condition: Some(Condition::Preimage {
                hash: [0u8; 32],
                preimage: vec![10],
            }),
            ..escrow.clone()
        };
        bad_escrow.state = EscrowState::Funded;
        assert_err(bad_escrow.execute(), EscrowError::ConditionViolation);
    }
}
