use core::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair};

use crate::error::{ClientError, Result};
use crate::interface::{ChainConfig, EscrowMetadata, EscrowParams};
use crate::Agent;

/// Escrow agent for interacting with the Solana network
pub struct SolanaAgent {
    // JSON-RPC client of a remote Solana node
    client: RpcClient,
    // Path to a Solana keypair
    // for signing transactions
    payer: Keypair,
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
            payer: read_keypair_file(&keypair_path)
                .map_err(|e| ClientError::Keypair(e.to_string()))?,
            program_id: Pubkey::from_str(&program_id)?,
        })
    }
}

#[async_trait::async_trait]
impl Agent for SolanaAgent {
    async fn create_escrow(&self, _params: &EscrowParams) -> Result<EscrowMetadata> {
        todo!()
    }

    async fn release_escrow(&self, _metadata: &EscrowMetadata) -> Result<()> {
        todo!()
    }

    async fn refund_escrow(&self, _metadata: &EscrowMetadata) -> Result<()> {
        todo!()
    }
}
