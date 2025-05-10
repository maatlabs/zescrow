use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::system_program;
use sha2::{Digest, Sha256};

declare_id!("8F9ByFr24Y7mAAbUvCcZ9w3GpD16LP6f2THw3Sygy3ct");

/// Program prefix for seed generation
pub const PREFIX: &str = "escrow";

#[program]
pub mod escrow {
    use super::*;

    /// Creates a new escrow, initializing a PDA and transferring lamports
    pub fn create_escrow(ctx: Context<CreateEscrow>, args: CreateEscrowArgs) -> Result<()> {
        let system_program_info = ctx.accounts.system_program.to_account_info().clone();
        let sender_info = ctx.accounts.sender.to_account_info().clone();
        let escrow_info = ctx.accounts.escrow_account.to_account_info().clone();

        // Transfer lamports from sender to the escrow PDA
        system_program::transfer(
            CpiContext::new(
                system_program_info,
                system_program::Transfer {
                    from: sender_info,
                    to: escrow_info,
                },
            ),
            args.amount,
        )?;

        let escrow = &mut ctx.accounts.escrow_account;

        escrow.sender = ctx.accounts.sender.key();
        escrow.recipient = ctx.accounts.recipient.key();
        escrow.amount = args.amount;
        escrow.finish_after = args.finish_after;
        escrow.cancel_after = args.cancel_after;
        escrow.condition = args.condition;

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Created
        });

        Ok(())
    }

    /// Finishes an escrow, enforcing time-lock and/or cryptographic condition
    pub fn finish_escrow(ctx: Context<FinishEscrow>, condition: Option<String>) -> Result<()> {
        let escrow = &ctx.accounts.escrow_account;
        let now = Clock::get()?.unix_timestamp;

        // Enforce finish_after if set
        if let Some(ts) = escrow.finish_after {
            require!(now >= ts, EscrowError::NotReady);
        }

        // Enforce crypto condition if set
        if let Some(cond) = &escrow.condition {
            let preimage = condition.ok_or(EscrowError::ConditionNotMet)?;
            let mut hasher = Sha256::new();
            hasher.update(preimage);
            let hash = hasher.finalize();
            require!(
                hash.as_slice() == cond.as_bytes(),
                EscrowError::ConditionNotMet
            );
        }

        // Transfer out lamports and close PDA
        **ctx
            .accounts
            .escrow_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= escrow.amount;
        **ctx
            .accounts
            .recipient
            .to_account_info()
            .try_borrow_mut_lamports()? += escrow.amount;

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Finished
        });

        Ok(())
    }

    /// Cancels an escrow after expiration, returning funds to sender
    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &ctx.accounts.escrow_account;
        let now = Clock::get()?.unix_timestamp;

        // Must be past cancel_after to reclaim
        require!(
            escrow.cancel_after.is_some_and(|ts| now >= ts),
            EscrowError::NotExpired
        );

        **ctx
            .accounts
            .escrow_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= escrow.amount;
        **ctx
            .accounts
            .sender
            .to_account_info()
            .try_borrow_mut_lamports()? += escrow.amount;

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
    /// Optional UNIX timestamp after which funds can be released
    pub finish_after: Option<i64>,
    /// Optional UNIX timestamp after which sender can reclaim funds
    pub cancel_after: Option<i64>,
    /// Optional cryptographic (e.g., SHA-256 preimage) condition
    pub condition: Option<String>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct CreateEscrow<'info> {
    /// Sender funding the escrow
    #[account(mut)]
    pub sender: Signer<'info>,

    /// Recipient of the escrow.
    ///
    /// CHECK: we enforce correctness via PDA seeds
    pub recipient: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [PREFIX.as_bytes(), sender.key().as_ref(), recipient.key().as_ref()],
        bump,
        payer = sender,
        space = 8  // discriminator
             + 32  // sender
             + 32  // recipient
             + 8   // amount
             + 1   // Option tag for finish_after
             + 8   // finish_after
             + 1   // Option tag for cancel_after
             + 8   // cancel_after
             + 1   // Option tag for condition
             + 32  // condition
    )]
    pub escrow_account: Account<'info, Escrow>,

    /// System program for lamport transfers
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateEscrowArgs {
    pub amount: u64,
    pub finish_after: Option<i64>,
    pub cancel_after: Option<i64>,
    pub condition: Option<String>,
}

#[derive(Accounts)]
pub struct FinishEscrow<'info> {
    /// Recipient claiming the funds
    #[account(mut)]
    pub recipient: Signer<'info>,

    /// PDA holding the escrow, closed to recipient on success
    #[account(
        mut,
        seeds = [PREFIX.as_bytes(), escrow_account.sender.as_ref(), recipient.key().as_ref()],
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
        seeds = [PREFIX.as_bytes(), sender.key().as_ref(), escrow_account.recipient.as_ref()],
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
    #[msg("Escrow not yet ready to finish.")]
    NotReady,
    #[msg("Provided fulfillment does not match the condition.")]
    ConditionNotMet,
    #[msg("Escrow has not yet expired for cancellation.")]
    NotExpired,
}
