/// Condition data structures and
/// deterministic verification logic
pub mod condition;
/// Escrow business logic and verification steps
pub mod escrow;
/// Data representations of fungible/NFT assets,
/// and identities of parties
pub mod identity;

pub mod error;
use error::EscrowError;

pub type Result<T> = std::result::Result<T, EscrowError>;

/// Chain-specific adapters
#[derive(Debug)]
pub enum Adapters {
    Ethereum,
    Solana,
}
