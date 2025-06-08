//! Escrow state machine with optional cryptographic conditions.
//!
//! This module provides the `Escrow` type, representing an escrow instance with its
//! asset, parties, state, and optional conditions.

use serde::{Deserialize, Serialize};

use crate::interface::ESCROW_CONDITIONS_PATH;
use crate::{Asset, Condition, EscrowError, EscrowMetadata, EscrowState, Party, Result};

/// Full escrow context, representing the locked asset, participants, and settlement rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    /// Asset locked in escrow.
    pub asset: Asset,
    /// Intended recipient of the asset.
    pub recipient: Party,
    /// The party who funded (deposited) the asset.
    pub sender: Party,
    /// Optional cryptographic condition that must be fulfilled before release.
    pub condition: Option<Condition>,
    /// Current state in the escrow lifecycle.
    pub state: EscrowState,
}

impl Escrow {
    /// Constructs an `Escrow` from on-chain metadata and, if required,
    /// loads the cryptographic condition from a JSON file.
    ///
    /// If `metadata.has_conditions` is `true`, reads the file at
    /// `ESCROW_CONDITIONS_PATH` and parses it as a `Condition`.
    ///
    /// # Errors
    ///
    /// - I/O error when reading the condition file.
    /// - JSON parsing error when decoding the condition.
    pub fn from_metadata(metadata: EscrowMetadata) -> Result<Self> {
        let EscrowMetadata {
            asset,
            sender,
            recipient,
            state,
            has_conditions,
            ..
        } = metadata;

        let condition = if has_conditions {
            let content = std::fs::read_to_string(ESCROW_CONDITIONS_PATH)?;
            let cond: Condition = serde_json::from_str(&content)?;
            Some(cond)
        } else {
            None
        };

        Ok(Self {
            asset,
            recipient,
            sender,
            condition,
            state,
        })
    }

    /// Attempts to execute (release) the escrow, performing all necessary checks.
    ///
    /// - Ensures current `state` is `Funded`.
    /// - Verifies `sender` and `recipient` identities.
    /// - Validates the `asset` parameters.
    /// - If a cryptographic `condition` is present, verifies it.
    ///
    /// On success, transitions to `EscrowState::Released`.
    ///
    /// # Errors
    ///
    /// Returns `EscrowError::InvalidState` if not in `Funded` state, or
    /// propagates identity, asset, or condition errors.
    pub fn execute(&mut self) -> Result<EscrowState> {
        if self.state != EscrowState::Funded {
            return Err(EscrowError::InvalidState);
        }

        self.sender.verify_identity()?;
        self.recipient.verify_identity()?;
        self.asset.validate()?;
        if let Some(condition) = &self.condition {
            condition.verify()?;
        }

        self.state = EscrowState::Released;
        Ok(self.state)
    }
}

impl std::fmt::Display for Escrow {
    /// Compact JSON representation of the `Escrow` for logging.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", json)
    }
}

#[cfg(test)]
mod tests {

    use core::str::FromStr as _;

    use num_bigint::BigUint;
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
            amount: BigUint::from(1000u64),
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
            amount: BigUint::from(0u64), // invalid zero amount
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
