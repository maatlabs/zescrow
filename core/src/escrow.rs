//! This module provides the `Escrow` type, representing an escrow instance with its
//! asset, parties, state, and optional conditions.

use bincode::{Decode, Encode};
#[cfg(feature = "json")]
use {
    crate::interface::{EscrowMetadata, ESCROW_CONDITIONS_PATH},
    serde::{Deserialize, Serialize},
    serde_json,
};

use crate::{Asset, Condition, EscrowError, ExecutionState, Party, Result};

/// Full escrow context, representing the locked asset, participants, and settlement rules.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode)]
pub struct Escrow {
    /// Asset locked in escrow.
    pub asset: Asset,
    /// Intended recipient of the asset.
    pub recipient: Party,
    /// The party who funded (deposited) the asset.
    pub sender: Party,
    /// Optional cryptographic condition that must be fulfilled before release.
    pub condition: Option<Condition>,
    /// State of escrow execution in the prover (zkVM).
    pub state: ExecutionState,
}

impl Escrow {
    /// Creates a new [Escrow].
    pub fn new(
        sender: Party,
        recipient: Party,
        asset: Asset,
        condition: Option<Condition>,
    ) -> Self {
        Self {
            asset,
            recipient,
            sender,
            condition,
            state: ExecutionState::Initialized,
        }
    }

    /// Attempts to execute (release) the escrow, performing all necessary checks.
    ///
    /// - Ensures current `state` is `Funded`.
    /// - Verifies `sender` and `recipient` identities.
    /// - Validates the `asset` parameters.
    /// - If a cryptographic `condition` is present, verifies it.
    ///
    /// On success, transitions to `ExecutionState::ConditionsMet`.
    ///
    /// # Errors
    ///
    /// Returns `EscrowError::InvalidState` if not in `Funded` state, or
    /// propagates identity, asset, or condition errors.
    pub fn execute(&mut self) -> Result<ExecutionState> {
        if self.state != ExecutionState::Funded {
            return Err(EscrowError::InvalidState);
        }

        self.sender.verify_identity()?;
        self.recipient.verify_identity()?;
        self.asset.validate()?;

        if let Some(condition) = &self.condition {
            condition.verify()?;
        }

        self.state = ExecutionState::ConditionsMet;
        Ok(self.state)
    }

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
    #[cfg(feature = "json")]
    pub fn from_metadata(metadata: EscrowMetadata) -> Result<Self> {
        let condition = if metadata.params.has_conditions {
            let content = std::fs::read_to_string(ESCROW_CONDITIONS_PATH)?;
            let cond: Condition = serde_json::from_str(&content)?;
            Some(cond)
        } else {
            None
        };

        Ok(Self {
            asset: metadata.params.asset,
            recipient: metadata.params.recipient,
            sender: metadata.params.sender,
            condition,
            state: metadata.state,
        })
    }
}

#[cfg(feature = "json")]
impl std::fmt::Display for Escrow {
    /// Compact JSON representation of the `Escrow` for logging.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{json}")
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest as _, Sha256};

    use super::*;
    use crate::{BigNumber, ID};

    #[test]
    fn execute_escrow() {
        let sender = Party::new("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let recipient = Party::new("0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8").unwrap();

        let asset = Asset::token(
            ID::from("0xdeadbeef".as_bytes()),
            BigNumber::from(1_000u64),
            BigNumber::from(2_000u64),
            18,
        );

        let preimage = b"secret".to_vec();
        let hash = Sha256::digest(&preimage);
        let condition = Condition::hashlock(hash.into(), preimage);

        let mut escrow = Escrow::new(sender.clone(), recipient.clone(), asset, Some(condition));
        escrow.state = ExecutionState::Funded;
        assert_eq!(escrow.execute().unwrap(), ExecutionState::ConditionsMet);
        assert_eq!(escrow.state, ExecutionState::ConditionsMet);
        // Ensure re-execution is not allowed
        assert!(escrow.execute().is_err());

        let invalid_asset = Asset::token(
            ID::from("0xdeadbeef".as_bytes()),
            BigNumber::from(0u64), // zero amount
            BigNumber::from(2_000u64),
            18,
        );

        let mut invalid_escrow = Escrow::new(sender, recipient, invalid_asset, None);
        invalid_escrow.state = ExecutionState::Funded;
        assert!(invalid_escrow.execute().is_err(),);
    }
}
