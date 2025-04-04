use std::collections::HashSet;

use bls12_381::G1Affine;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

use crate::{EscrowError, TimeSource, VerificationCtx};

pub trait Condition: Encode + Decode + TypeInfo {
    fn verify(&self, ctx: &VerificationCtx) -> bool;
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct MultiSig {
    pub threshold: u32,
}

impl Condition for MultiSig {
    fn verify(&self, ctx: &VerificationCtx) -> bool {
        let mut unique_sigs = HashSet::new();
        let mut valid_sigs = 0usize;
        let threshold = self.threshold as usize;

        for sig in &ctx.signatures {
            // check for duplicates
            if unique_sigs.contains(sig) {
                continue;
            }
            if verify_signature(sig).is_ok() {
                valid_sigs += 1;
                unique_sigs.insert(sig);

                if valid_sigs >= threshold {
                    return true;
                }
            }
        }
        valid_sigs >= threshold
    }
}

fn verify_signature(sig_bytes: &[u8; 48]) -> Result<G1Affine, EscrowError> {
    G1Affine::from_compressed(sig_bytes)
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
        let current = match ctx.time {
            TimeSource::BlockNumber(n) => n,
            TimeSource::Timestamp(t) => t,
        };
        current >= self.min_block && current <= self.max_block
    }
}
