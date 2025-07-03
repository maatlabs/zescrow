#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::system_program;
pub use groth_16_verifier::ID as GROTH16_VERIFIER_ID;
pub use system_program::ID as SYSTEM_PROGRAM_ID;
use verifier_router::cpi::accounts::Verify;
use verifier_router::program::VerifierRouter as VerifierRouterProgram;
pub use verifier_router::router::Proof;
use verifier_router::state::{VerifierEntry, VerifierRouter};
pub use verifier_router::ID as VERIFIER_ROUTER_ID;

declare_id!("5VwWRGjhF6xv51WgWb1E3iYhWNf63HauSST8AZ5B8zTJ");

/// PDA seed prefix for `escrow` program
pub const ESCROW: &str = "escrow";
/// PDA seed prefix for `verifier_router` program
pub const ROUTER: &str = "router";
/// PDA seed prefix for `groth_16_verifier` program
pub const VERIFIER: &str = "verifier";
// We assume only one verifier will be used
pub const SELECTOR: u32 = 1;

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
        escrow.has_conditions = args.has_conditions;

        emit!(EscrowEvent {
            sender: escrow.sender,
            recipient: escrow.recipient,
            amount: escrow.amount,
            action: EscrowState::Created
        });
        Ok(())
    }

    /// Finishes an escrow, enforcing time-lock and/or cryptographic condition.
    pub fn finish_escrow(ctx: Context<FinishEscrow>, args: FinishEscrowArgs) -> Result<()> {
        let escrow = &ctx.accounts.escrow_account;
        let current_slot = Clock::get()?.slot;

        if let Some(finish_after) = escrow.finish_after {
            require!(current_slot >= finish_after, EscrowError::NotReady);
        }

        if escrow.has_conditions {
            let proof_data = args.proof_data.ok_or(error!(EscrowError::ProofDataEmpty))?;

            let image_id = proof_data.image_id;
            let proof = proof_data.proof;
            let journal_digest = proof_data.journal_digest;

            let cpi_accounts = Verify {
                router: ctx.accounts.router_account.to_account_info(),
                verifier_entry: ctx.accounts.verifier_entry.to_account_info(),
                verifier_program: ctx.accounts.verifier_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(ctx.accounts.router.to_account_info(), cpi_accounts);

            verifier_router::cpi::verify(cpi_ctx, SELECTOR, proof, image_id, journal_digest)
                .map_err(|_| EscrowError::ConditionNotMet)?;
        }

        // // Transfer out lamports and close PDA
        // **ctx
        //     .accounts
        //     .escrow_account
        //     .to_account_info()
        //     .try_borrow_mut_lamports()? -= escrow.amount;
        // **ctx
        //     .accounts
        //     .recipient
        //     .to_account_info()
        //     .try_borrow_mut_lamports()? += escrow.amount;

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

        // Must be past cancel_after to reclaim
        require!(
            escrow
                .cancel_after
                .is_some_and(|cancel_after| current_slot >= cancel_after),
            EscrowError::NotExpired
        );

        // **ctx
        //     .accounts
        //     .escrow_account
        //     .to_account_info()
        //     .try_borrow_mut_lamports()? -= escrow.amount;
        // **ctx
        //     .accounts
        //     .sender
        //     .to_account_info()
        //     .try_borrow_mut_lamports()? += escrow.amount;

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
    /// Whether this escrow is subject to any cryptographic conditions
    pub has_conditions: bool,
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

    #[account(
        init,
        seeds = [ESCROW.as_bytes(), sender.key().as_ref(), recipient.key().as_ref()],
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
             + 1   // has_conditions
    )]
    pub escrow_account: Account<'info, Escrow>,

    /// System program for lamport transfers
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateEscrowArgs {
    pub amount: u64,
    pub finish_after: Option<u64>,
    pub cancel_after: Option<u64>,
    pub has_conditions: bool,
}

#[derive(Accounts)]
#[instruction(args: FinishEscrowArgs)]
pub struct FinishEscrow<'info> {
    /// Recipient claiming the funds
    #[account(mut)]
    pub recipient: Signer<'info>,

    /// PDA holding the escrow, closed to recipient on success
    #[account(
        mut,
        seeds = [ESCROW.as_bytes(), escrow_account.sender.as_ref(), recipient.key().as_ref()],
        bump,
        close = recipient
    )]
    pub escrow_account: Account<'info, Escrow>,

    /// The router program that will route to the correct verifier
    pub router: Program<'info, VerifierRouterProgram>,

    /// The router account that will be used for routing our proof
    pub router_account: Account<'info, VerifierRouter>,

    /// The PDA entry in the router that maps our selector to the actual verifier.
    // TODO: Try changing to unchecked account because verifier checks the fields.
    #[account(
        seeds = [
            VERIFIER.as_bytes(),
            &SELECTOR.to_le_bytes()
        ],
        bump,
        seeds::program = VERIFIER_ROUTER_ID
    )]
    pub verifier_entry: Account<'info, VerifierEntry>,

    /// The actual Groth16 verifier program that will verify the proof.
    /// CHECK: The verifier program checks are handled by the router program
    pub verifier_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Clone, PartialEq, Eq, AnchorDeserialize, AnchorSerialize)]
pub struct FinishEscrowArgs {
    pub proof_data: Option<ProofData>,
}

/// Values necessary for verification of RISC Zero ZK proofs.
#[derive(Clone, PartialEq, Eq, AnchorDeserialize, AnchorSerialize)]
pub struct ProofData {
    pub image_id: [u8; 32],
    pub proof: Proof,
    pub journal_digest: [u8; 32],
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    /// Original initializer reclaiming funds
    #[account(mut)]
    pub sender: Signer<'info>,

    /// PDA holding the escrow, closed back to sender
    #[account(
        mut,
        seeds = [ESCROW.as_bytes(), sender.key().as_ref(), escrow_account.recipient.as_ref()],
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
    #[msg("No proof data given.")]
    ProofDataEmpty,
    #[msg("Proof verification failed.")]
    ConditionNotMet,
    #[msg("Too early to cancel.")]
    NotExpired,
}
