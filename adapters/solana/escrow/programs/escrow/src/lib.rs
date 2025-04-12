use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("6A9rvdALJ3JxaEVaz3AYfSoRPLdQgvzHUj5WApaEbGZR");

/// Expiry in slots added to the current slot.
pub const ESCROW_EXPIRY: u64 = 5000;

#[program]
pub mod escrow {
    use super::*;

    /// Initializes an escrow account,
    /// transferring `amount` lamports from depositor and locking in escrow.
    pub fn create_escrow(ctx: Context<CreateEscrow>, amount: u64) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        escrow.depositor = ctx.accounts.depositor.key();
        escrow.beneficiary = ctx.accounts.beneficiary.key();
        escrow.amount = amount;
        escrow.expiry = Clock::get()?.slot + ESCROW_EXPIRY;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.depositor.to_account_info(),
                    to: ctx.accounts.escrow.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    /// Unlocks the escrowed funds, transferring to the beneficiary if not expired.
    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;
        require!(Clock::get()?.slot <= escrow.expiry, EscrowError::Expired);

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.escrow.to_account_info(),
                    to: ctx.accounts.beneficiary.to_account_info(),
                },
            ),
            escrow.amount,
        )?;

        Ok(())
    }

    /// Unlocks escrowed funds, transferring to the depositor after expiry.
    pub fn refund_escrow(ctx: Context<RefundEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;
        require!(
            Clock::get()?.slot > escrow.expiry,
            EscrowError::NotYetExpired
        );

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.escrow.to_account_info(),
                    to: ctx.accounts.depositor.to_account_info(),
                },
            ),
            escrow.amount,
        )?;

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
    /// CHECK: The depositor account is derived from the escrow.
    #[account(mut)]
    depositor: AccountInfo<'info>,
    /// CHECK: Checked via address constraint to match escrow's beneficiary.
    #[account(mut, address = escrow.beneficiary)]
    beneficiary: AccountInfo<'info>,
    #[account(mut, close = depositor, has_one = depositor, has_one = beneficiary)]
    escrow: Account<'info, EscrowAccount>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefundEscrow<'info> {
    /// CHECK: Checked via address constraint to match escrow's depositor.
    #[account(mut, address = escrow.depositor)]
    depositor: AccountInfo<'info>,
    #[account(mut, close = depositor, has_one = depositor)]
    escrow: Account<'info, EscrowAccount>,
    system_program: Program<'info, System>,
}

#[error_code]
pub enum EscrowError {
    #[msg("Escrow has already expired.")]
    Expired,
    #[msg("Escrow has not yet expired.")]
    NotYetExpired,
}
