//! Mock cryptographic primitives for testing escrow state transitions
//! without actual cryptography. NOT FOR PRODUCTION USE.

use crate::EscrowError;

/// A mock Pedersen commitment scheme using deterministic "commitments"
/// based on simple arithmetic for test reproducibility.
#[derive(Debug, Clone, Copy, Default)]
pub struct MockPedersen;

impl MockPedersen {
    /// Generates a "commitment" that's just (value || !value) in little-endian bytes.
    /// Returns (commitment, opening).
    pub fn commit(value: u128) -> ([u8; 32], [u8; 32]) {
        let mut commitment = [0u8; 32];
        let mut opening = [0u8; 32];

        let not_value = !value;
        commitment[..16].copy_from_slice(&value.to_le_bytes());
        commitment[16..].copy_from_slice(&not_value.to_le_bytes());

        // Opening is just the value repeated twice.
        opening[..16].copy_from_slice(&value.to_le_bytes());
        opening[16..].copy_from_slice(&value.to_le_bytes());

        (commitment, opening)
    }

    /// "Verifies" a commitment by checking the simple arithmetic structure.
    pub fn verify(commitment: [u8; 32], value: u128, opening: [u8; 32]) -> Result<(), EscrowError> {
        // Validate opening structure
        let opening_value = u128::from_le_bytes(opening[..16].try_into().unwrap());
        if opening_value != value || opening[16..] != opening[..16] {
            return Err(EscrowError::InvalidCommitment);
        }

        let (expected_commit, _) = Self::commit(value);
        if commitment != expected_commit {
            return Err(EscrowError::InvalidCommitment);
        }

        Ok(())
    }
}

/// Mock BLS signature verification that treats any signature with
/// the first byte set to 0xFF as valid.
#[derive(Debug, Clone, Copy, Default)]
pub struct MockBlsSignature;

impl MockBlsSignature {
    /// "Verifies" a signature by checking the first byte is 0xFF.
    pub fn verify(sig: &[u8; 48]) -> Result<(), EscrowError> {
        if sig[0] == 0xFF {
            Ok(())
        } else {
            Err(EscrowError::InvalidSignature)
        }
    }

    /// "Verifies" multiple signatures (all must be "valid").
    pub fn aggregate_verify(sigs: &[[u8; 48]]) -> Result<(), EscrowError> {
        for sig in sigs {
            Self::verify(sig)?;
        }
        Ok(())
    }
}

/// Mock ZK proof system that accepts any proof starting with "ZKESCROW".
#[derive(Debug, Clone, Copy, Default)]
pub struct MockZkProof;

impl MockZkProof {
    pub const PROOF: &[u8; 8] = b"ZKESCROW";

    /// Generates a mock proof with header + state bytes.
    pub fn generate(state: &[u8]) -> Vec<u8> {
        let mut proof = Self::PROOF.to_vec();
        proof.extend_from_slice(state);
        proof
    }

    /// Verifies proof header and state consistency.
    pub fn verify(proof: &[u8], expected_state: &[u8]) -> Result<(), EscrowError> {
        if !proof.starts_with(Self::PROOF) {
            return Err(EscrowError::InvalidProof);
        }

        let proof_state = &proof[8..];
        if proof_state != expected_state {
            return Err(EscrowError::InvalidProof);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_pedersen_commitment() {
        let (commit, open) = MockPedersen::commit(100);
        assert!(MockPedersen::verify(commit, 100, open).is_ok());

        let bad_open = [0u8; 32];
        assert!(MockPedersen::verify(commit, 100, bad_open).is_err());
    }

    #[test]
    fn test_mock_bls_signature() {
        let valid_sig = {
            let mut sig = [0u8; 48];
            sig[0] = 0xFF;
            sig
        };
        assert!(MockBlsSignature::verify(&valid_sig).is_ok());

        let invalid_sig = [0x00u8; 48];
        assert!(MockBlsSignature::verify(&invalid_sig).is_err());
    }

    #[test]
    fn test_mock_zero_knowledge_proof() {
        let state = b"state";
        let proof = MockZkProof::generate(state);
        assert!(MockZkProof::verify(&proof, state).is_ok());

        let bad_proof = b"BADPROOF".to_vec();
        assert!(MockZkProof::verify(&bad_proof, state).is_err());
    }
}
