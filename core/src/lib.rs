//! Pure, chain-agnostic escrow core library.

pub mod asset;
pub mod condition;
pub mod error;
pub mod escrow;
pub mod identity;
pub mod interface;

pub use asset::Asset;
pub use condition::Condition;
pub use error::EscrowError;
pub use escrow::Escrow;
pub use identity::Party;
pub use interface::{Chain, ChainConfig, ChainMetadata, EscrowMetadata, EscrowParams, EscrowState};

pub type Result<T> = std::result::Result<T, EscrowError>;
