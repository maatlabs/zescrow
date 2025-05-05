//! Pure, chain-agnostic escrow core library.
//!
//! Exposes:
//! - `Asset` & `Party` for funds and participants
//! - `Condition` for cryptographic release logic
//! - `Escrow` & `EscrowState` with time locks and state transitions
//! - `EscrowError` for all error cases

pub mod condition;
pub mod error;
pub mod escrow;
pub mod identity;

pub use condition::Condition;
pub use error::EscrowError;
pub use escrow::{Escrow, EscrowState};
pub use identity::{Asset, Party};

pub type Result<T> = std::result::Result<T, EscrowError>;
