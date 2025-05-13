//! Pure, chain-agnostic escrow core library.
//!
//! Exposes:
//! - `Asset` & `Party` for funds and participants
//! - `Condition` for cryptographic release logic
//! - `Escrow` & `EscrowState` with time locks and state transitions
//! - `EscrowError` for all error cases

pub mod asset;
pub mod condition;
pub mod error;
pub mod escrow;
pub mod identity;
pub mod interface;

pub use asset::Asset;
pub use condition::Condition;
pub use error::EscrowError;
pub use escrow::{Escrow, EscrowState};
pub use identity::Party;
pub use interface::{Chain, ChainMetadata, EscrowMetadata};

pub type Result<T> = std::result::Result<T, EscrowError>;
