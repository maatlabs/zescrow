use anchor_lang::prelude::*;

declare_id!("6A9rvdALJ3JxaEVaz3AYfSoRPLdQgvzHUj5WApaEbGZR");

/// Expiry in slots added to the current slot.
pub const ESCROW_EXPIRY: u64 = 5000;

#[program]
pub mod escrow {
    use super::*;

    pub fn create_escrow(ctx: Context<CreateEscrow>, amount: u64) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        escrow.depositor = *ctx.accounts.depositor.key;
        escrow.beneficiary = *ctx.accounts.beneficiary.key;
        escrow.amount = amount;
        escrow.expiry = Clock::get()?.slot + ESCROW_EXPIRY;

        **escrow.to_account_info().try_borrow_mut_lamports()? += amount;
        **ctx.accounts.depositor.try_borrow_mut_lamports()? -= amount;

        Ok(())
    }

    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        require!(Clock::get()?.slot <= escrow.expiry, EscrowError::Expired);

        **ctx.accounts.beneficiary.try_borrow_mut_lamports()? += escrow.amount;
        **escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.amount;

        Ok(())
    }

    pub fn refund_escrow(ctx: Context<RefundEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        require!(
            Clock::get()?.slot > escrow.expiry,
            EscrowError::NotYetExpired
        );

        **ctx.accounts.depositor.try_borrow_mut_lamports()? += escrow.amount;
        **escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.amount;

        Ok(())
    }
}

#[account]
pub struct EscrowAccount {
    depositor: Pubkey,
    beneficiary: Pubkey,
    amount: u64,
    expiry: u64,
}

#[derive(Accounts)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    depositor: Signer<'info>,
    /// CHECK: The beneficiary account is stored in the escrow and
    /// validated during release.
    beneficiary: AccountInfo<'info>,
    #[account(init, payer = depositor, space = 8 + 32 + 32 + 8 + 8)]
    escrow: Account<'info, EscrowAccount>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {
    /// CHECK: Checked via address constraint (TODO) to match escrow's beneficiary.
    #[account(mut)]
    beneficiary: AccountInfo<'info>,
    #[account(mut)]
    escrow: Account<'info, EscrowAccount>,
}

#[derive(Accounts)]
pub struct RefundEscrow<'info> {
    /// CHECK: Checked via address constraint (TODO) to match escrow's depositor.
    #[account(mut)]
    depositor: AccountInfo<'info>,
    #[account(mut)]
    escrow: Account<'info, EscrowAccount>,
}

#[error_code]
pub enum EscrowError {
    #[msg("Escrow has already expired.")]
    Expired,
    #[msg("Escrow has not yet expired.")]
    NotYetExpired,
}
