use bincode::{Decode, Encode};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use super::Condition;

/// N-of-M threshold condition.
///
/// Satisfied when at least `threshold` of the `subconditions` verify
/// successfully. Subconditions can be any [`Condition`] variant,
/// including nested thresholds.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Threshold {
    /// Minimum number of valid subconditions required.
    pub threshold: usize,

    /// Subconditions to evaluate.
    pub subconditions: Vec<Condition>,
}

impl Threshold {
    /// Verifies that at least `threshold` subconditions are satisfied.
    ///
    /// Returns `Ok(())` if the threshold is met, or `Err` with details
    /// about how many conditions passed versus required.
    ///
    /// A threshold of zero is always satisfied, regardless of subconditions.
    pub fn verify(&self) -> Result<(), Error> {
        (self.threshold == 0)
            .then_some(())
            .map(Ok)
            .unwrap_or_else(|| self.verify_threshold())
    }

    /// Counts satisfied subconditions and checks against threshold.
    fn verify_threshold(&self) -> Result<(), Error> {
        let satisfied = self.count_satisfied();

        (satisfied >= self.threshold)
            .then_some(())
            .ok_or(Error::ThresholdNotMet {
                required: self.threshold,
                satisfied,
            })
    }

    /// Counts the number of subconditions that verify successfully.
    fn count_satisfied(&self) -> usize {
        self.subconditions
            .iter()
            .filter_map(|c| c.verify().ok())
            .count()
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
