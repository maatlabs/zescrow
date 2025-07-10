use bincode::{Decode, Encode};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use super::Condition;

/// Threshold condition: at least `threshold` subconditions must hold.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Threshold {
    /// Minimum number of valid subconditions required.
    pub threshold: usize,

    /// Subconditions to evaluate.
    pub subconditions: Vec<Condition>,
}

impl Threshold {
    /// Verify subconditions
    pub fn verify(&self) -> Result<(), Error> {
        // zero threshold always satisfied
        if self.threshold == 0 {
            return Ok(());
        }

        let satisfied = self
            .subconditions
            .iter()
            .filter(|c| c.verify().is_ok())
            .count();

        if satisfied >= self.threshold {
            Ok(())
        } else {
            Err(Error::ThresholdNotMet {
                required: self.threshold,
                satisfied,
            })
        }
    }
}

/// Threshold conditions verification errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Fewer than the required number of subconditions were satisfied.
    #[error("needed at least {required} passes, but only {satisfied} succeeded")]
    ThresholdNotMet {
        /// Minimum number of valid subconditions required.
        required: usize,
        /// Number of verified subconditions.
        satisfied: usize,
    },
}
