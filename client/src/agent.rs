use zescrow_core::{EscrowMetadata, EscrowParams};

use crate::Result;

pub mod ethereum;
pub mod solana;

/// Core interface for blockchain-specific escrow operations.
///
/// Implementators must provide chain-specific logic for:
/// - Creating escrow contracts/accounts
/// - Releasing funds to beneficiaries
/// - Refunding expired escrows
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// Create a new escrow with specified parameters
    ///
    /// # Arguments
    /// * `params` - Escrow creation parameters including assets and parties
    ///
    /// # Returns
    /// Metadata containing chain-specific identifiers and transaction details
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata>;

    /// Release escrowed funds to beneficiary
    ///
    /// # Arguments
    /// * `metadata` - Escrow metadata from creation
    ///
    /// # Preconditions
    /// - Escrow must be in funded state
    /// - Current block/slot must be before expiry
    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;

    /// Refund escrowed funds to depositor
    ///
    /// # Arguments
    /// * `metadata` - Escrow metadata from creation
    ///
    /// # Preconditions
    /// - Escrow must have expired
    /// - No prior release/refund executed
    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()>;
}

/// Converts a RISC Zero image ID from [u32; 8] format to [u8; 32] format.
///
/// This conversion is necessary because RISC Zero generates image IDs as 8 32-bit integers,
/// but the on-chain Solana verifier expects a 32-byte array.
///
/// # Arguments
///
/// * `input` - The RISC Zero image ID as [u32; 8]
///
/// # Returns
///
/// * `[u8; 32]` - The converted image ID as a 32-byte array
pub fn convert_array(input: [u32; 8]) -> [u8; 32] {
    let bytes: Vec<u8> = input.iter().flat_map(|&x| x.to_le_bytes()).collect();
    bytes.try_into().expect("Failed to convert array")
}
