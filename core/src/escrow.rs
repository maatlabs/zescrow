//! Escrow state machine with time locks and optional crypto conditions.

use serde::{Deserialize, Serialize};

use crate::interface::ESCROW_CONDITIONS_PATH;
use crate::{Asset, Condition, EscrowError, EscrowMetadata, EscrowState, Party, Result};

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
    /// Current state.
    pub state: EscrowState,
}

impl Escrow {
    /// Validates and attempts to finish (release) escrow by
    /// verifying all predefined conditions.
    // TODO: Add more robust checks
    pub fn execute(&mut self) -> Result<EscrowState> {
        if self.state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }
        self.asset.validate()?;

        self.sender.verify_identity()?;
        self.recipient.verify_identity()?;

        if let Some(cond) = &self.condition {
            cond.verify()?;
        }

        self.state = EscrowState::Released;
        Ok(self.state)
    }

    pub fn from_metadata(metadata: EscrowMetadata) -> Self {
        let EscrowMetadata {
            asset,
            sender,
            recipient,
            state,
            has_conditions,
            ..
        } = metadata;

        let condition = if has_conditions {
            let conditions = std::fs::read_to_string(ESCROW_CONDITIONS_PATH)
                .expect("Failed to read escrow conditions JSON file.");
            let escrow_conditions: Condition =
                serde_json::from_str(&conditions).expect("Invalid escrow conditions JSON");
            Some(escrow_conditions)
        } else {
            None
        };

        Self {
            asset,
            recipient,
            sender,
            condition,
            state,
        }
    }
}

#[cfg(test)]
mod tests {

    use core::str::FromStr as _;

    use sha2::{Digest as _, Sha256};

    use super::*;
    use crate::identity::ID;
    use crate::Chain;

    #[test]
    fn execute_escrow() {
        let sender = Party::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let recipient = Party::from_str("0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8").unwrap();
        let asset = Asset::Token {
            chain: Chain::Ethereum,
            contract: ID::from_str("0xdeadbeef").unwrap(),
            amount: 1000,
            decimals: 18,
        };

        let condition = Condition::Preimage {
            hash: Sha256::digest(b"secret").into(),
            preimage: b"secret".to_vec(),
        };

        let mut escrow = Escrow {
            asset,
            recipient: recipient.clone(),
            sender: sender.clone(),
            condition: Some(condition),
            state: EscrowState::Funded,
        };

        assert_eq!(escrow.execute().unwrap(), EscrowState::Released);
        assert_eq!(escrow.state, EscrowState::Released);

        // Ensure re-execution is not allowed
        assert!(escrow.execute().is_err());

        // Asset validation failure test
        let invalid_asset = Asset::Token {
            chain: Chain::Ethereum,
            contract: ID::from_str("0xdeadbeef").unwrap(),
            amount: 0, // invalid zero amount
            decimals: 18,
        };

        let mut invalid_escrow = Escrow {
            asset: invalid_asset,
            recipient,
            sender,
            condition: None,
            state: EscrowState::Funded,
        };

        assert!(invalid_escrow.execute().is_err(),);
    }
}
