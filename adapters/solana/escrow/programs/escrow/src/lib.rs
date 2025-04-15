use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::system_program;

declare_id!("8uMq5t5rot6EqrnmubFsVth4ccSwgrDh4SsKvSDY4GQT");

/// Expiry in slots added to the current slot.
pub const ESCROW_EXPIRY: u64 = 5000;

#[program]
pub mod escrow {
    use super::*;

    /// Initializes a PDA escrow account funded by the depositor.
    pub fn create_escrow(ctx: Context<CreateEscrow>, amount: u64) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        escrow.depositor = ctx.accounts.depositor.key();
        escrow.beneficiary = ctx.accounts.beneficiary.key();
        escrow.amount = amount;
        escrow.expiry = Clock::get()?.slot + ESCROW_EXPIRY;
        escrow.bump = ctx.bumps.escrow;

        // Transfer lamports from depositor to the escrow PDA
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

    /// Releases escrow to beneficiary if not expired.
    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;

        require!(Clock::get()?.slot <= escrow.expiry, EscrowError::Expired);

        // Transfer lamports from PDA to beneficiary
        invoke_signed(
            &system_instruction::transfer(
                &ctx.accounts.escrow.key(),
                &ctx.accounts.beneficiary.key(),
                escrow.amount,
            ),
            &[
                ctx.accounts.escrow.to_account_info(),
                ctx.accounts.beneficiary.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"escrow",
                ctx.accounts.depositor.key.as_ref(),
                ctx.accounts.beneficiary.key.as_ref(),
                &[escrow.bump],
            ]],
        )?;

        Ok(())
    }

    /// Refunds escrow to depositor if expired.
    pub fn refund_escrow(ctx: Context<RefundEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;

        require!(
            Clock::get()?.slot > escrow.expiry,
            EscrowError::NotYetExpired
        );

        // Transfer lamports from PDA to depositor
        invoke_signed(
            &system_instruction::transfer(
                &ctx.accounts.escrow.key(),
                &ctx.accounts.depositor.key(),
                escrow.amount,
            ),
            &[
                ctx.accounts.escrow.to_account_info(),
                ctx.accounts.depositor.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"escrow",
                ctx.accounts.depositor.key.as_ref(),
                ctx.accounts.beneficiary.key.as_ref(),
                &[escrow.bump],
            ]],
        )?;

        Ok(())
    }
}

#[account]
pub struct EscrowAccount {
    pub depositor: Pubkey,
    pub beneficiary: Pubkey,
    pub amount: u64,
    pub expiry: u64,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    pub beneficiary: SystemAccount<'info>,

    #[account(
        init,
        payer = depositor,
        space = 8 + 32 + 32 + 8 + 8 + 1,
        seeds = [b"escrow", depositor.key().as_ref(), beneficiary.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {
    /// Used to validate PDA
    pub depositor: SystemAccount<'info>,

    #[account(mut)]
    pub beneficiary: SystemAccount<'info>,

    #[account(
        mut,
        close = depositor,
        seeds = [b"escrow", depositor.key().as_ref(), beneficiary.key().as_ref()],
        bump = escrow.bump,
        has_one = depositor @ EscrowError::InvalidDepositor,
        has_one = beneficiary @ EscrowError::InvalidBeneficiary
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefundEscrow<'info> {
    #[account(mut)]
    pub depositor: SystemAccount<'info>,

    pub beneficiary: SystemAccount<'info>,

    #[account(
        mut,
        close = depositor,
        seeds = [b"escrow", depositor.key().as_ref(), beneficiary.key().as_ref()],
        bump = escrow.bump,
        has_one = depositor @ EscrowError::InvalidDepositor,
        has_one = beneficiary @ EscrowError::InvalidBeneficiary
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum EscrowError {
    #[msg("Escrow has already expired.")]
    Expired,
    #[msg("Escrow has not yet expired.")]
    NotYetExpired,
    #[msg("Invalid depositor account.")]
    InvalidDepositor,
    #[msg("Invalid beneficiary account.")]
    InvalidBeneficiary,
}
