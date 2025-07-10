//! Escrow program with XRPL-style time-lock semantics.

#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::system_program;

declare_id!("8u5bT8xkx6X4qKuRnn7oeDdrE1v4jG1F749YzqP1Z7BQ");

/// Seed prefix for PDA derivation.
pub const ESCROW: &[u8] = b"escrow";

#[program]
pub mod escrow {
    use super::*;

    /// Creates a new escrow, enforcing XRPL-style guards:
    /// - At least one of `finish_after` or `cancel_after` must be set.  
    /// - If both set, `finish_after < cancel_after`.
    pub fn create_escrow(ctx: Context<CreateEscrow>, args: CreateEscrowArgs) -> Result<()> {
        // Must have at least one resolution path
        require!(
            args.finish_after.is_some() || args.cancel_after.is_some(),
            EscrowError::MustSpecifyPath
        );
        // If both set, enforce ordering
        if let (Some(finish), Some(cancel)) = (args.finish_after, args.cancel_after) {
            require!(finish < cancel, EscrowError::InvalidTimeOrder);
        }
        // Amount cannot be zero
        require!(args.amount > 0, EscrowError::InvalidAmount);

        // Transfer lamports into the PDA
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.sender.to_account_info(),
                to: ctx.accounts.escrow_account.to_account_info(),
            },
        );
        system_program::transfer(cpi_ctx, args.amount)?;

        let escrow = &mut ctx.accounts.escrow_account;

        escrow.sender = ctx.accounts.sender.key();
        escrow.recipient = ctx.accounts.recipient.key();
        escrow.amount = args.amount;
        escrow.finish_after = args.finish_after;
        escrow.cancel_after = args.cancel_after;
        escrow.bump = ctx.bumps.escrow_account;

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Created
        });

        Ok(())
    }

    /// Releases an escrow:
    /// - If `finish_after` is `Some(t)`, require current slot >= t.  
    /// - If `finish_after` is `None`, allow immediate release.  
    /// - Only callable by `recipient`.
    pub fn finish_escrow(ctx: Context<FinishEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow_account;
        let current_slot = Clock::get()?.slot;

        require!(
            ctx.accounts.recipient.key() == escrow.recipient,
            EscrowError::Unauthorized
        );

        if let Some(t) = escrow.finish_after {
            require!(current_slot >= t, EscrowError::NotReady);
        }

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Finished
        });

        Ok(())
    }

    /// Cancels an escrow:
    /// - Requires `cancel_after` to be `Some(t)`.  
    /// - Current slot >= t.  
    /// - Only callable by the original `sender`.
    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow_account;
        let current_slot = Clock::get()?.slot;

        require!(
            ctx.accounts.sender.key() == escrow.sender,
            EscrowError::Unauthorized
        );
        // Must have set a `cancel_after`
        require!(escrow.cancel_after.is_some(), EscrowError::CancelNotAllowed);
        let t = escrow.cancel_after.unwrap();
        require!(current_slot >= t, EscrowError::NotExpired);

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Cancelled
        });

        Ok(())
    }
}

/// Escrow account data, stored in a PDA.
#[account]
pub struct Escrow {
    /// Account that initialized the escrow
    pub sender: Pubkey,
    /// Intended beneficiary of the escrowed funds
    pub recipient: Pubkey,
    /// Amount of lamports locked
    pub amount: u64,
    /// Optional slot after which funds can be released
    pub finish_after: Option<u64>,
    /// Optional slot after which sender can reclaim funds
    pub cancel_after: Option<u64>,
    /// PDA bump seed for address validation.
    pub bump: u8,
}

/// Context for `create_escrow` transaction.
#[derive(Accounts)]
#[instruction(args: CreateEscrowArgs)]
pub struct CreateEscrow<'info> {
    /// Sender funding the escrow
    #[account(mut)]
    pub sender: Signer<'info>,

    /// Recipient of the escrow; must differ from sender to
    /// prevent self-escrow.
    ///
    /// CHECK: we enforce correctness via PDA seeds.
    #[account(
        constraint = recipient.key() != sender.key() @ EscrowError::InvalidRecipient
    )]
    pub recipient: UncheckedAccount<'info>,

    /// PDA holding the escrow.
    #[account(
        init,
        payer = sender,
        space = 8  + std::mem::size_of::<Escrow>(),
        seeds = [ESCROW, sender.key().as_ref(), recipient.key().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, Escrow>,

    /// System program for lamport transfers
    pub system_program: Program<'info, System>,
}

/// Arguments for `create_escrow` transaction.
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateEscrowArgs {
    /// Amount to escrow.
    pub amount: u64,
    /// Optional slot after which "release" is allowed.
    /// Must be `None` or less than `cancel_after` if both are set.
    pub finish_after: Option<u64>,
    /// Optional slot after which "cancel" is allowed.
    /// Must be `None` or greater than `finish_after` if both are set.
    pub cancel_after: Option<u64>,
}

/// Context for `finish_escrow`.
#[derive(Accounts)]
pub struct FinishEscrow<'info> {
    /// Recipient claiming the funds
    #[account(mut)]
    pub recipient: Signer<'info>,

    /// PDA holding the escrow, closed to recipient on success
    #[account(
        mut,
        seeds = [ESCROW, escrow_account.sender.as_ref(), recipient.key().as_ref()],
        bump = escrow_account.bump,
        close = recipient
    )]
    pub escrow_account: Account<'info, Escrow>,
}

/// Context for `cancel_escrow` transaction.
#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    /// Original initializer reclaiming funds
    #[account(mut)]
    pub sender: Signer<'info>,

    /// PDA holding the escrow, closed back to sender on success.
    #[account(
        mut,
        seeds = [ESCROW, sender.key().as_ref(), escrow_account.recipient.as_ref()],
        bump = escrow_account.bump,
        close = sender
    )]
    pub escrow_account: Account<'info, Escrow>,
}

/// Events emitted by the escrow program.
#[event]
pub struct EscrowEvent {
    /// Original depositor
    pub sender: Pubkey,
    /// Intended beneficiary
    pub recipient: Pubkey,
    /// Escrow amount; must be nonzero
    pub amount: u64,
    /// What stage of the escrow lifecycle was just executed
    pub action: EscrowState,
}

/// Escrow lifecycle actions.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum EscrowState {
    /// Escrow initialized, PDA funded, funds locked.
    Created,
    /// Escrow released to intended beneficiary (`recipient`).
    Finished,
    /// Escrow cancelled and original `sender` refunded.
    Cancelled,
}

/// Program-specific error codes.
#[error_code]
pub enum EscrowError {
    /// Specified amount is zero.
    #[msg("Amount must be greater than zero.")]
    InvalidAmount,

    /// Both `finish_after` or `cancel_after` are missing.
    #[msg("Must specify at least one of finish_after or cancel_after.")]
    MustSpecifyPath,

    /// Specified slot for `finish_after` exceeds that of `cancel_after`.
    #[msg("finish_after must be less than cancel_after.")]
    InvalidTimeOrder,

    /// Self-escrow is not allowed.
    #[msg("Sender and recipient must differ.")]
    InvalidRecipient,

    /// Only callable by designated `Signer`.
    #[msg("Unauthorized caller.")]
    Unauthorized,

    /// `finish_after` not yet reached.
    #[msg("Too early to finish.")]
    NotReady,

    /// `cancel_after` not specified; cannot cancel escrow.
    #[msg("Cancel not allowed (no cancel_after).")]
    CancelNotAllowed,

    /// `cancel_after` not yet reached.
    #[msg("Too early to cancel.")]
    NotExpired,
}
