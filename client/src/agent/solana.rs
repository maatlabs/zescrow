use core::str::FromStr;
use std::path::PathBuf;

use anchor_client::solana_sdk::instruction::Instruction;
use anchor_lang::prelude::AccountMeta;
use anchor_lang::solana_program::hash::hashv;
use anchor_lang::InstructionData;
use escrow::{
    instruction as escrow_instruction, CreateEscrowArgs, FinishEscrowArgs, Proof, ProofData,
    ESCROW, GROTH16, GROTH16_VERIFIER_ID, ROUTER, SELECTOR, SYSTEM_PROGRAM_ID, VERIFIER_ROUTER_ID,
};
use num_traits::ToPrimitive;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use tracing::{debug, info, instrument, trace};
use zescrow_core::interface::{
    load_escrow_data, ChainConfig, ProofData as ZKProof, PROOF_DATA_PATH,
};
use zescrow_core::{ChainMetadata, EscrowMetadata, EscrowParams, ExecutionState};

use super::{convert_array, Agent};
use crate::error::{AgentError, ClientError, Result};

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
    #[instrument(skip_all, fields(rpc_url = %config.rpc_url(), program_id = %config.sol_escrow_program()?))]
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
            Pubkey::from_str(escrow_program_id).map_err(|e| AgentError::Solana(e.to_string()))?;
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
    #[instrument(skip(self, params), fields(
        sender = %params.sender,
        recipient = %params.recipient,
        amount = %params.asset.amount(),
        finish_after = ?params.finish_after,
        cancel_after = ?params.cancel_after,
        has_conditions = %params.has_conditions
    ))]
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
            &[ESCROW.as_bytes(), sender.as_ref(), recipient.as_ref()],
            &self.escrow_program_id,
        );
        info!(pda = %escrow_account, "Derived pda");

        let ix = Instruction {
            program_id: self.escrow_program_id,
            accounts: vec![
                AccountMeta::new(sender, true),
                AccountMeta::new_readonly(recipient, false),
                AccountMeta::new(escrow_account, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
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
            chain_config: params.chain_config.clone(),
            asset: params.asset.clone(),
            sender: params.sender.clone(),
            recipient: params.recipient.clone(),
            has_conditions: params.has_conditions,
            chain_data: ChainMetadata::Solana {
                escrow_program_id: self.escrow_program_id.to_string(),
            },
            state: ExecutionState::Funded,
        })
    }

    #[instrument(skip(self, metadata))]
    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let sender = Pubkey::from_str(&metadata.sender.to_string())?;

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

        let (escrow_account, _) = Pubkey::find_program_address(
            &[ESCROW.as_bytes(), sender.as_ref(), recipient.as_ref()],
            &self.escrow_program_id,
        );
        debug!("Using the address: {escrow_account} as the Escrow Account");

        let (router_account, _) =
            Pubkey::find_program_address(&[ROUTER.as_bytes()], &VERIFIER_ROUTER_ID);
        debug!("Using the address: {router_account} as the Router Account");

        let (verifier_entry, _) = Pubkey::find_program_address(
            &[GROTH16.as_bytes(), &SELECTOR.to_le_bytes()],
            &VERIFIER_ROUTER_ID,
        );

        let proof_data = if metadata.has_conditions {
            // load proof data
            let data: ZKProof =
                load_escrow_data(PROOF_DATA_PATH).map_err(|e| AgentError::Solana(e.to_string()))?;

            let image_id = convert_array(data.image_id);
            let proof = Proof {
                pi_a: data.proof.pi_a,
                pi_b: data.proof.pi_b,
                pi_c: data.proof.pi_c,
            };
            let journal_digest = hashv(&[data.output.as_slice()]).to_bytes();

            Some(ProofData {
                image_id,
                proof,
                journal_digest,
            })
        } else {
            None
        };

        let ix = Instruction {
            program_id: self.escrow_program_id,
            accounts: vec![
                AccountMeta::new(recipient, true),
                AccountMeta::new(escrow_account, false),
                AccountMeta::new_readonly(VERIFIER_ROUTER_ID, false),
                AccountMeta::new_readonly(router_account, false),
                AccountMeta::new_readonly(verifier_entry, false),
                AccountMeta::new_readonly(GROTH16_VERIFIER_ID, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
            data: InstructionData::data(&escrow_instruction::FinishEscrow {
                args: FinishEscrowArgs { proof_data },
            }),
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

    #[instrument(skip(self, metadata))]
    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let sender = Pubkey::from_str(&metadata.sender.to_string())?;
        let recipient = Pubkey::from_str(&metadata.recipient.to_string())?;

        let (escrow_account, _) = Pubkey::find_program_address(
            &[ESCROW.as_bytes(), sender.as_ref(), recipient.as_ref()],
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
