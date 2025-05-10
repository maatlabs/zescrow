use core::str::FromStr;

use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::system_program;
use anchor_lang::prelude::AccountMeta;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::InstructionData;
use escrow::{instruction as escrow_instruction, CreateEscrowArgs, PREFIX};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::{read_keypair_file, Keypair};
use solana_sdk::transaction::Transaction;
use solana_sdk::{pubkey::Pubkey, sysvar::Sysvar as _};

use crate::error::{ClientError, Result};
use crate::interface::{Chain, ChainConfig, ChainMetadata, EscrowMetadata, EscrowParams};
use crate::Agent;

/// Escrow agent for interacting with the Solana network
pub struct SolanaAgent {
    // JSON-RPC client of a remote Solana node
    client: RpcClient,
    // Path to a Solana keypair
    // for signing transactions
    signer: Keypair,
    // On-chain escrow program ID
    program_id: Pubkey,
}

impl SolanaAgent {
    pub fn new(config: ChainConfig) -> Result<Self> {
        let ChainConfig::Solana {
            rpc_url,
            keypair_path,
            program_id,
        } = config
        else {
            return Err(ClientError::ConfigMismatch);
        };

        Ok(Self {
            client: RpcClient::new(rpc_url),
            signer: read_keypair_file(&keypair_path)
                .map_err(|e| ClientError::Keypair(e.to_string()))?,
            program_id: Pubkey::from_str(&program_id)?,
        })
    }
}

#[async_trait::async_trait]
impl Agent for SolanaAgent {
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        let sender = Pubkey::from_str(&params.sender)?;
        let recipient = Pubkey::from_str(&params.recipient)?;
        let (pda, bump) = Pubkey::find_program_address(
            &[PREFIX.as_bytes(), sender.as_ref(), recipient.as_ref()],
            &self.program_id,
        );

        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(sender, true),
                AccountMeta::new_readonly(recipient, false),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: InstructionData::data(&escrow_instruction::CreateEscrow {
                args: CreateEscrowArgs {
                    amount: params.amount,
                    finish_after: params.finish_after,
                    cancel_after: params.cancel_after,
                    condition: params.condition.clone().map(|c| c.to_string()),
                },
            }),
        };

        let recent_hash = self.client.get_latest_blockhash()?;
        let tx =
            Transaction::new_signed_with_payer(&[ix], Some(&sender), &[&self.signer], recent_hash);
        self.client.send_and_confirm_transaction(&tx)?;

        Ok(EscrowMetadata {
            chain: Chain::Solana,
            sender: params.sender.clone(),
            recipient: params.recipient.clone(),
            amount: params.amount,
            finish_after: params.finish_after,
            cancel_after: params.cancel_after,
            condition: params.condition.clone(),
            created_block: Clock::get()?.slot,
            chain_data: ChainMetadata::Solana {
                program_id: self.program_id.to_string(),
                pda: pda.to_string(),
                bump,
            },
        })
    }

    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let recipient = Pubkey::from_str(&metadata.recipient)?;
        let pda = Pubkey::from_str(metadata.chain_data.get_pda()?)?;

        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(recipient, true),
                AccountMeta::new(pda, false),
            ],
            data: InstructionData::data(&escrow_instruction::FinishEscrow {
                condition: metadata.condition.clone().map(|c| c.to_string()),
            }),
        };

        let recent_hash = self.client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&recipient),
            &[&self.signer],
            recent_hash,
        );
        self.client.send_and_confirm_transaction(&tx)?;
        Ok(())
    }

    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let sender = Pubkey::from_str(&metadata.sender)?;
        let pda = Pubkey::from_str(metadata.chain_data.get_pda()?)?;

        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![AccountMeta::new(sender, true), AccountMeta::new(pda, false)],
            data: InstructionData::data(&escrow_instruction::CancelEscrow {}),
        };

        let recent_hash = self.client.get_latest_blockhash()?;
        let tx =
            Transaction::new_signed_with_payer(&[ix], Some(&sender), &[&self.signer], recent_hash);
        self.client.send_and_confirm_transaction(&tx)?;
        Ok(())
    }
}
