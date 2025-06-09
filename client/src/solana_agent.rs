use core::str::FromStr;
use std::path::PathBuf;

use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::system_program;
use anchor_lang::prelude::AccountMeta;
use anchor_lang::InstructionData;
use escrow::{instruction as escrow_instruction, CreateEscrowArgs, FinishEscrowArgs, PREFIX};
use num_traits::ToPrimitive;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use zescrow_core::interface::ChainConfig;
use zescrow_core::{ChainMetadata, EscrowMetadata, EscrowParams, ExecutionState};

use crate::error::{AgentError, ClientError, Result};
use crate::Agent;

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
    pub async fn new(
        config: &ChainConfig,
        recipient_keypair_path: Option<PathBuf>,
    ) -> Result<Self> {
        let ChainConfig::Solana {
            rpc_url,
            sender_keypair_path,
            escrow_program_id,
            ..
        } = config
        else {
            return Err(ClientError::ConfigMismatch);
        };

        let sender_keypair = read_keypair_file(sender_keypair_path)
            .map_err(|e| ClientError::Keypair(e.to_string()))?;
        let recipient_keypair = match recipient_keypair_path {
            Some(path) => {
                let kp =
                    read_keypair_file(&path).map_err(|e| ClientError::Keypair(e.to_string()))?;
                Some(kp)
            }
            None => None,
        };

        Ok(Self {
            client: RpcClient::new(rpc_url),
            sender_keypair,
            recipient_keypair,
            escrow_program_id: Pubkey::from_str(escrow_program_id)?,
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
        let amount = params.asset.amount();
        let amount = amount.to_u64().ok_or(ClientError::AssetOverflow)?;
        let (pda, bump) = Pubkey::find_program_address(
            &[PREFIX.as_bytes(), sender.as_ref(), recipient.as_ref()],
            &self.escrow_program_id,
        );

        let ix = Instruction {
            program_id: self.escrow_program_id,
            accounts: vec![
                AccountMeta::new(sender, true),
                AccountMeta::new_readonly(recipient, false),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: InstructionData::data(&escrow_instruction::CreateEscrow {
                args: CreateEscrowArgs {
                    amount,
                    finish_after: params.finish_after,
                    cancel_after: params.cancel_after,
                    has_conditions: params.has_conditions,
                },
            }),
        };

        let recent_hash = self.client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&sender),
            &[&self.sender_keypair],
            recent_hash,
        );
        self.client.send_and_confirm_transaction(&tx)?;

        let EscrowParams {
            chain_config,
            asset,
            sender,
            recipient,
            has_conditions,
            ..
        } = params.clone();

        Ok(EscrowMetadata {
            chain_config,
            asset,
            sender,
            recipient,
            has_conditions,
            chain_data: ChainMetadata::Solana {
                escrow_program_id: self.escrow_program_id.to_string(),
                pda: pda.to_string(),
                bump,
            },
            state: ExecutionState::Funded,
        })
    }

    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let recipient = Pubkey::from_str(&metadata.recipient.to_string())?;
        let recipient_keypair = self
            .recipient_keypair
            .as_ref()
            .ok_or_else(|| ClientError::Keypair("Recipient keypair not provided".to_string()))?;
        if recipient != recipient_keypair.pubkey() {
            return Err(ClientError::Keypair(
                "Recipient keypair-pubkey mismatch".to_string(),
            ));
        }
        let pda = metadata.chain_data.get_pda()?;
        let pda = Pubkey::from_str(&pda)?;

        // TODO
        // ZK proof generation
        let proof: Vec<u8> = vec![];

        let verifier_program = metadata
            .chain_config
            .sol_verifier_program()
            .map_err(|e| AgentError::Solana(e.to_string()))?;
        let verifier_program = Pubkey::from_str(&verifier_program)?;

        let ix = Instruction {
            program_id: self.escrow_program_id,
            accounts: vec![
                AccountMeta::new(recipient, true),
                AccountMeta::new(pda, false),
                AccountMeta::new(verifier_program, false),
            ],
            data: InstructionData::data(&escrow_instruction::FinishEscrow {
                args: FinishEscrowArgs { proof },
            }),
        };

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
        let sender = Pubkey::from_str(&metadata.sender.to_string())?;
        let pda = metadata.chain_data.get_pda()?;
        let pda = Pubkey::from_str(&pda)?;

        let ix = Instruction {
            program_id: self.escrow_program_id,
            accounts: vec![AccountMeta::new(sender, true), AccountMeta::new(pda, false)],
            data: InstructionData::data(&escrow_instruction::CancelEscrow {}),
        };

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
