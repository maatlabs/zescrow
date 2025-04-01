use scale::{Decode, Encode};
use scale_info::TypeInfo;
use sha2::{Digest, Sha256};

use crate::assets::Asset;
use crate::conditions::Condition;
use crate::{EscrowError, Party, VerificationCtx};

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct Escrow<C, A>
where
    C: Condition,
    A: Asset,
{
    pub id: [u8; 32],
    pub state: State,
    pub condition: C,
    pub asset: A,
    pub parties: Vec<Party>,
    pub created_block: u64,
}

#[derive(Debug, Clone, Copy, Encode, Decode, TypeInfo, PartialEq)]
pub enum State {
    Initialized,
    Funded,
    Completed,
    Disputed,
}

impl<C, A> Escrow<C, A>
where
    C: Condition,
    A: Asset,
{
    pub fn initialize(
        condition: C,
        asset: A,
        parties: Vec<Party>,
        created_block: u64,
    ) -> Result<Self, EscrowError> {
        let mut hasher = Sha256::new();

        hasher.update(condition.encode());
        hasher.update(asset.commit_amount());
        hasher.update(created_block.to_le_bytes());

        let id = hasher.finalize().into();

        Ok(Self {
            id,
            state: State::Initialized,
            condition,
            asset,
            parties,
            created_block,
        })
    }

    pub fn fund(&mut self) -> Result<(), EscrowError> {
        if self.state != State::Initialized {
            return Err(EscrowError::Uninitialized);
        }
        self.state = State::Funded;
        Ok(())
    }

    pub fn execute(&mut self, ctx: VerificationCtx, proof: &[u8]) -> Result<State, EscrowError> {
        if self.state != State::Funded {
            return Err(EscrowError::InsufficientFunds);
        }
        if !self.condition.verify(&ctx) {
            return Err(EscrowError::ConditionFailure);
        }
        self.verify_state_transition_proof(proof)?;
        self.state = State::Completed;
        Ok(self.state)
    }

    fn verify_state_transition_proof(&self, _proof: &[u8]) -> Result<(), EscrowError> {
        todo!()
    }
}
