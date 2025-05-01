use anchor_lang::prelude::*;

declare_id!("8F9ByFr24Y7mAAbUvCcZ9w3GpD16LP6f2THw3Sygy3ct");

#[program]
pub mod zkescrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
