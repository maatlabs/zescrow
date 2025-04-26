use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::ClientError;

pub fn load_escrow_input_data<T>(path: &PathBuf) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let file = File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

pub fn save_escrow_metadata<T>(path: &PathBuf, data: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    let file = File::create(path)?;
    Ok(serde_json::to_writer_pretty(file, data)?)
}

pub fn load_chain_config(metadata: &EscrowMetadata) -> anyhow::Result<ChainConfig> {
    // Get original config path from environment (set during escrow creation)
    let config_path =
        std::env::var("CHAIN_CONFIG_PATH").context("Missing CHAIN_CONFIG_PATH env variable")?;
    let config = load_escrow_input_data::<ChainConfig>(&PathBuf::from(config_path))?;

    match (config, &metadata.chain_data) {
        (
            ChainConfig::Ethereum {
                rpc_url,
                private_key,
                ..
            },
            ChainMetadata::Ethereum {
                contract_address,
                block_number: _,
            },
        ) => Ok(ChainConfig::Ethereum {
            rpc_url,
            private_key,
            contract_address: contract_address.clone(),
        }),
        (
            ChainConfig::Solana {
                rpc_url,
                keypair_path,
                ..
            },
            ChainMetadata::Solana {
                program_id,
                pda: _,
                bump: _,
            },
        ) => Ok(ChainConfig::Solana {
            rpc_url,
            keypair_path,
            program_id: program_id.clone(),
        }),
        _ => Err(anyhow::anyhow!("Chain config and chain metadata mismatch")),
    }
}

/// Escrow input parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowParams {
    /// Target blockchain network
    pub chain: Chain,
    /// Depositor's blockchain address
    pub depositor: String,
    /// Beneficiary's blockchain address
    pub beneficiary: String,
    /// Escrow amount in native token units
    pub amount: u64,
    /// Expiration block/slot number
    pub expiry: u64,
}

/// Result of escrow creation, release, or refund.
#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowMetadata {
    /// Original blockchain network
    pub chain: Chain,
    /// Depositor's address
    pub depositor: String,
    /// Beneficiary's address
    pub beneficiary: String,
    /// Locked amount
    pub amount: u64,
    /// Expiration block/slot
    pub expiry: u64,
    /// Escrow transaction ID
    pub tx_id: String,
    /// Chain-specific metadata
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

/// Chain-specific network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChainConfig {
    /// Ethereum network configuration
    Ethereum {
        /// JSON-RPC endpoint URL
        rpc_url: String,
        /// Hex-encoded private key
        /// in wallet import format (WIF)
        private_key: String,
        /// Escrow smart contract address
        contract_address: String,
    },
    /// Solana network configuration
    Solana {
        /// JSON-RPC endpoint URL
        rpc_url: String,
        /// Path to payer keypair file
        keypair_path: String,
        /// Program ID for escrow program
        program_id: String,
    },
}
