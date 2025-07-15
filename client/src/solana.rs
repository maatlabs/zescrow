use core::str::FromStr;
use std::path::PathBuf;

use anchor_client::solana_sdk::instruction::Instruction;
use anchor_lang::prelude::AccountMeta;
use anchor_lang::{system_program, InstructionData};
use escrow::{instruction as escrow_instruction, CreateEscrowArgs, ESCROW};
use num_traits::ToPrimitive;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use tracing::{debug, info, trace};
use zescrow_core::interface::ChainConfig;
use zescrow_core::{EscrowMetadata, EscrowParams, ExecutionState};

use super::Agent;
use crate::error::{AgentError, ClientError};
use crate::Result;

/// Escrow agent for interacting with the Solana network
pub struct SolanaAgent {
    // JSON-RPC client of a remote Solana node
    client: RpcClient,
    // Escrow creator keypair
    sender_keypair: Keypair,
    // Escrow beneficiary keypair
    recipient_keypair: Option<Keypair>,
    // On-chain escrow program ID
    escrow_program_id: Pubkey,
}

impl SolanaAgent {
    /// Create a new SolanaAgent, reading keypairs and program ID.
    pub async fn new(
        config: &ChainConfig,
        recipient_keypair_path: Option<PathBuf>,
    ) -> Result<Self> {
        let ChainConfig {
            rpc_url,
            sender_private_id,
            agent_id,
            ..
        } = config;

        let sender_keypair = read_keypair_file(sender_private_id)
            .map_err(|e| ClientError::Keypair(e.to_string()))?;
        debug!(sender = %sender_keypair.pubkey(), "Loaded sender keypair");

        // Load optional recipient keypair
        let recipient_keypair = if let Some(path) = recipient_keypair_path {
            let kp = read_keypair_file(path).map_err(|e| ClientError::Keypair(e.to_string()))?;
            debug!(recipient = %kp.pubkey(), "Loaded recipient keypair");
            Some(kp)
        } else {
            debug!("No recipient keypair provided");
            None
        };

        // Parse program ID
        let escrow_program_id =
            Pubkey::from_str(&agent_id).map_err(|e| AgentError::Solana(e.to_string()))?;
        info!("Using escrow program {}", escrow_program_id);

        Ok(Self {
            client: RpcClient::new(rpc_url),
            sender_keypair,
            recipient_keypair,
            escrow_program_id,
        })
    }
}

#[async_trait::async_trait]
impl Agent for SolanaAgent {
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        let sender = Pubkey::from_str(&params.sender.to_string())?;
        if sender != self.sender_keypair.pubkey() {
            return Err(ClientError::Keypair(
                "Sender keypair-pubkey mismatch".to_string(),
            ));
        }

        let recipient = Pubkey::from_str(&params.recipient.to_string())?;
        let amount = params
            .asset
            .amount()
            .0
            .to_u64()
            .ok_or(ClientError::AssetOverflow)?;
        trace!("Computed amount: {}", amount);

        let (escrow_account, _) = Pubkey::find_program_address(
            &[ESCROW, sender.as_ref(), recipient.as_ref()],
            &self.escrow_program_id,
        );
        info!(pda = %escrow_account, "Derived pda");

        let ix = Instruction {
            program_id: self.escrow_program_id,
            accounts: vec![
                AccountMeta::new(sender, true),
                AccountMeta::new_readonly(recipient, false),
                AccountMeta::new(escrow_account, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: InstructionData::data(&escrow_instruction::CreateEscrow {
                args: CreateEscrowArgs {
                    amount,
                    finish_after: params.finish_after,
                    cancel_after: params.cancel_after,
                },
            }),
        };
        debug!("CreateEscrow instruction built");

        let recent_hash = self.client.get_latest_blockhash()?;
        info!(blockhash = %recent_hash, "Fetched recent blockhash");
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&sender),
            &[&self.sender_keypair],
            recent_hash,
        );
        debug!("Signed CreateEscrow transaction");
        self.client.send_and_confirm_transaction(&tx)?;

        Ok(EscrowMetadata {
            params: params.clone(),
            state: ExecutionState::Funded,
            escrow_id: None,
        })
    }

    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let sender = Pubkey::from_str(&metadata.params.sender.to_string())?;

        let recipient = Pubkey::from_str(&metadata.params.recipient.to_string())?;
        let recipient_keypair = self
            .recipient_keypair
            .as_ref()
            .ok_or_else(|| ClientError::Keypair("Recipient keypair not provided".to_string()))?;
        if recipient != recipient_keypair.pubkey() {
            return Err(ClientError::Keypair(
                "Recipient keypair-pubkey mismatch".to_string(),
            ));
        }

        let (escrow_account, _) = Pubkey::find_program_address(
            &[ESCROW, sender.as_ref(), recipient.as_ref()],
            &self.escrow_program_id,
        );
        debug!("Using the address: {escrow_account} as the Escrow Account");

        let ix = Instruction {
            program_id: self.escrow_program_id,
            accounts: vec![
                AccountMeta::new(recipient, true),
                AccountMeta::new(escrow_account, false),
            ],
            data: InstructionData::data(&escrow_instruction::FinishEscrow {}),
        };
        debug!("FinishEscrow instruction built");

        let recent_hash = self.client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&recipient),
            &[recipient_keypair],
            recent_hash,
        );
        self.client.send_and_confirm_transaction(&tx)?;

        Ok(())
    }

    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let sender = Pubkey::from_str(&metadata.params.sender.to_string())?;
        let recipient = Pubkey::from_str(&metadata.params.recipient.to_string())?;

        let (escrow_account, _) = Pubkey::find_program_address(
            &[ESCROW, sender.as_ref(), recipient.as_ref()],
            &self.escrow_program_id,
        );

        let ix = Instruction {
            program_id: self.escrow_program_id,
            accounts: vec![
                AccountMeta::new(sender, true),
                AccountMeta::new(escrow_account, false),
            ],
            data: InstructionData::data(&escrow_instruction::CancelEscrow {}),
        };
        debug!("CancelEscrow instruction built");

        let recent_hash = self.client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&sender),
            &[&self.sender_keypair],
            recent_hash,
        );
        self.client.send_and_confirm_transaction(&tx)?;

        Ok(())
    }
}
