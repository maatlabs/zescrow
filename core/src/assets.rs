use merlin::Transcript;

use crate::EscrowError;

pub trait Asset {
    fn commit_amount(&self) -> [u8; 32];
    fn verify_deposit(&self, transcript: &mut Transcript) -> Result<(), EscrowError>;
}
