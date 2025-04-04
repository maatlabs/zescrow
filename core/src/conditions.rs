use scale::{Decode, Encode};
use scale_info::TypeInfo;

use crate::{EscrowError, VerificationCtx};

pub trait Condition: Encode + Decode + TypeInfo {
    fn verify(&self, ctx: &VerificationCtx) -> bool;
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct MultiSig {
    pub threshold: u32, // `usize`: Decode not satisfied,
}

impl Condition for MultiSig {
    fn verify(&self, ctx: &VerificationCtx) -> bool {
        let valid_sigs = ctx
            .signatures
            .iter()
            .filter(|sig| verify_signature(sig).is_ok())
            .count();
        valid_sigs >= self.threshold as usize
    }
}

fn verify_signature(sig_bytes: &[u8; 48]) -> Result<bls12_381::G1Affine, EscrowError> {
    bls12_381::G1Affine::from_compressed(sig_bytes)
        .into_option()
        .ok_or(EscrowError::InvalidSignature)
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
