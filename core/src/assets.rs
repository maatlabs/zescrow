use merlin::Transcript;
use rand_core::OsRng;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

use crate::EscrowError;

pub trait Asset {
    /// Generate commitment for the asset value.
    fn commit(&self, rng: &mut OsRng) -> Result<[u8; 32], EscrowError>;

    /// Generate ZK proof for the committed value.
    fn generate_proof(&self, transcript: &mut Transcript) -> Result<Vec<u8>, EscrowError>;

    /// Verify proof against public commitment.
    fn verify_proof(
        &self,
        commitment: &[u8; 32],
        proof: &[u8],
        transcript: &mut Transcript,
    ) -> Result<(), EscrowError>;
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum AssetType {
    Fungible {
        commitment: [u8; 32],
        /// Hash(amount || randomness) using Blake2s
        proof: Option<[u8; 64]>,
    },
    NonFungible {
        id_commitment: [u8; 32],
        /// Proof of valid ID encoding
        proof: Option<[u8; 80]>,
    },
}
