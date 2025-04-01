use scale::{Decode, Encode};
use scale_info::TypeInfo;

use crate::VerificationCtx;

pub trait Condition: Encode + Decode + TypeInfo {
    fn verify(&self, ctx: &VerificationCtx) -> bool;
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct TimeLock {
    pub min_block: u64,
    pub max_block: u64,
}

impl Condition for TimeLock {
    fn verify(&self, ctx: &VerificationCtx) -> bool {
        ctx.current_block >= self.min_block && ctx.current_block <= self.max_block
    }
}
