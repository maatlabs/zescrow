#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::system_program;

declare_id!("8u5bT8xkx6X4qKuRnn7oeDdrE1v4jG1F749YzqP1Z7BQ");

/// Program-derived address seed prefix
pub const ESCROW: &str = "escrow";

#[program]
pub mod escrow {
    use super::*;

    /// Creates a new escrow, initializing a PDA and transferring lamports.
    pub fn create_escrow(ctx: Context<CreateEscrow>, args: CreateEscrowArgs) -> Result<()> {
        // fund PDA
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

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Created
        });

        Ok(())
    }

    /// Finishes an escrow, enforcing time-lock condition.
    pub fn finish_escrow(ctx: Context<FinishEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow_account;
        let current_slot = Clock::get()?.slot;

        // Must be past finish_after to release funds
        if let Some(finish_after) = escrow.finish_after {
            require!(current_slot >= finish_after, EscrowError::NotReady);
        }

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Finished
        });

        Ok(())
    }

    /// Cancels an escrow after expiration, returning funds to sender.
    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow_account;
        let current_slot = Clock::get()?.slot;

        // Must be past cancel_after to reclaim funds
        if let Some(cancel_after) = escrow.cancel_after {
            require!(current_slot >= cancel_after, EscrowError::NotExpired);
        }

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Cancelled
        });

        Ok(())
    }
}

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
}

#[derive(Accounts)]
#[instruction(args: CreateEscrowArgs)]
pub struct CreateEscrow<'info> {
    /// Sender funding the escrow
    #[account(mut)]
    pub sender: Signer<'info>,

    /// Recipient of the escrow.
    ///
    /// CHECK: we enforce correctness via PDA seeds
    pub recipient: UncheckedAccount<'info>,

    /// PDA holding the escrow.
    #[account(
        init,
        seeds = [
            ESCROW.as_bytes(),
            sender.key().as_ref(),
            recipient.key().as_ref()
        ],
        bump,
        payer = sender,
        space = 8  // discriminator
             + 32  // sender
             + 32  // recipient
             + 8   // amount
             + 1   // option tag for finish_after
             + 8   // finish_after
             + 1   // option tag for cancel_after
             + 8   // cancel_after
    )]
    pub escrow_account: Account<'info, Escrow>,

    /// System program for lamport transfers
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateEscrowArgs {
    pub amount: u64,
    /// Optional slot after which "release" is allowed.
    /// Must be `None` or less than `cancel_after` if both are set.
    pub finish_after: Option<u64>,
    /// Optional slot after which "cancel" is allowed.
    /// Must be `None` or greater than `finish_after` if both are set.
    pub cancel_after: Option<u64>,
}

#[derive(Accounts)]
pub struct FinishEscrow<'info> {
    /// Recipient claiming the funds
    #[account(mut)]
    pub recipient: Signer<'info>,

    /// PDA holding the escrow, closed to recipient on success
    #[account(
        mut,
        seeds = [
            ESCROW.as_bytes(),
            escrow_account.sender.as_ref(),
            recipient.key().as_ref()
        ],
        bump,
        close = recipient
    )]
    pub escrow_account: Account<'info, Escrow>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    /// Original initializer reclaiming funds
    #[account(mut)]
    pub sender: Signer<'info>,

    /// PDA holding the escrow, closed back to sender
    #[account(
        mut,
        seeds = [
            ESCROW.as_bytes(),
            sender.key().as_ref(),
            escrow_account.recipient.as_ref()
        ],
        bump,
        close = sender
    )]
    pub escrow_account: Account<'info, Escrow>,
}

/// Events emitted by the escrow program
#[event]
pub struct EscrowEvent {
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub action: EscrowState,
}

/// Escrow lifecycle actions
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum EscrowState {
    Created,
    Finished,
    Cancelled,
}

#[error_code]
pub enum EscrowError {
    #[msg("Too early to finish.")]
    NotReady,
    #[msg("Too early to cancel.")]
    NotExpired,
}
