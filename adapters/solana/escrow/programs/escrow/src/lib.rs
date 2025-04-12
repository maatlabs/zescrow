use anchor_lang::prelude::*;

declare_id!("6A9rvdALJ3JxaEVaz3AYfSoRPLdQgvzHUj5WApaEbGZR");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
