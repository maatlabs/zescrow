use core::str::FromStr;

use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::system_program;
use anchor_client::solana_sdk::sysvar::clock;
use anchor_lang::prelude::AccountMeta;
use anchor_lang::InstructionData;
use escrow::{instruction as escrow_instruction, ESCROW_PREFIX};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use solana_sdk::transaction::Transaction;

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
        let program_id = self.program_id;
        let beneficiary = Pubkey::from_str(&params.beneficiary)?;

        let (pda, bump) = Pubkey::find_program_address(
            &[
                ESCROW_PREFIX.as_bytes(),
                self.signer.pubkey().as_ref(),
                beneficiary.as_ref(),
            ],
            &program_id,
        );

        let ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(self.signer.pubkey(), true),
                AccountMeta::new_readonly(beneficiary, false),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(clock::id(), false),
            ],
            data: InstructionData::data(&escrow_instruction::CreateEscrow {
                amount: params.amount,
                expiry: params.expiry,
            }),
        };

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.signer.pubkey()));
        let blockhash = self.client.get_latest_blockhash()?;
        tx.sign(&[&self.signer], blockhash);

        self.client.send_and_confirm_transaction(&tx)?;

        Ok(EscrowMetadata {
            chain: Chain::Solana,
            depositor: self.signer.pubkey().to_string(),
            beneficiary: params.beneficiary.clone(),
            amount: params.amount,
            expiry: params.expiry,
            chain_data: ChainMetadata::Solana {
                program_id: program_id.to_string(),
                pda: pda.to_string(),
                bump,
            },
        })
    }

    async fn release_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let program_id = self.program_id;
        let pda = Pubkey::from_str(metadata.chain_data.get_pda()?)?;
        let depositor = Pubkey::from_str(&metadata.depositor)?;
        let beneficiary = Pubkey::from_str(&metadata.beneficiary)?;

        let ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new_readonly(depositor, false),
                AccountMeta::new(beneficiary, true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(clock::id(), false),
            ],
            data: InstructionData::data(&escrow_instruction::ReleaseEscrow {}),
        };

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.signer.pubkey()));
        let blockhash = self.client.get_latest_blockhash()?;
        tx.sign(&[&self.signer], blockhash);

        self.client.send_and_confirm_transaction(&tx)?;
        Ok(())
    }

    async fn refund_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let program_id = self.program_id;
        let pda = Pubkey::from_str(metadata.chain_data.get_pda()?)?;
        let depositor = Pubkey::from_str(&metadata.depositor)?;
        let beneficiary = Pubkey::from_str(&metadata.beneficiary)?;

        let ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(depositor, true),
                AccountMeta::new_readonly(beneficiary, false),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(clock::id(), false),
            ],
            data: InstructionData::data(&escrow_instruction::RefundEscrow {}),
        };

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.signer.pubkey()));
        let blockhash = self.client.get_latest_blockhash()?;
        tx.sign(&[&self.signer], blockhash);

        self.client.send_and_confirm_transaction(&tx)?;
        Ok(())
    }
}
