use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ClientError;

/// Escrow initialization inputs.
#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowParams {
    pub chain: Chain,
    pub depositor: String,
    pub beneficiary: String,
    pub amount: u64,
    // expiry block/slot
    pub expiry: u64,
}

/// Result of escrow creation, release, or refund.
#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowMetadata {
    pub chain: Chain,
    pub depositor: String,
    pub beneficiary: String,
    pub amount: u64,
    // expiry block/slot
    pub expiry: u64,
    pub tx_id: String,
    #[serde(flatten)]
    pub chain_data: ChainMetadata,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Ethereum,
    Solana,
}

impl FromStr for Chain {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" => Ok(Self::Ethereum),
            "solana" => Ok(Self::Solana),
            _ => Err(ClientError::UnsupportedChain(s.into())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChainMetadata {
    Ethereum {
        contract_address: String,
        block_number: u64,
    },
    Solana {
        program_id: String,
        pda: String,
        bump: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChainConfig {
    Ethereum {
        rpc_url: String,
        private_key: String,
        contract_address: String,
    },
    Solana {
        rpc_url: String,
        keypair_path: String,
        program_id: String,
    },
}
