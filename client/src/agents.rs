use core::str::FromStr;
use std::convert::TryFrom;

use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::types::Address;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair};

use crate::error::ClientError;
use crate::interface::ChainConfig;

pub struct EthereumAgent {
    pub provider: Provider<Http>,
    pub wallet: LocalWallet,
    pub contract_address: Address,
}

pub struct SolanaAgent {
    pub client: RpcClient,
    pub payer: Keypair,
    pub program_id: Pubkey,
}

impl EthereumAgent {
    pub fn new(config: ChainConfig) -> Result<Self, ClientError> {
        let ChainConfig::Ethereum {
            rpc_url,
            private_key,
            contract_address,
        } = config
        else {
            return Err(ClientError::ConfigMismatch);
        };

        Ok(Self {
            provider: Provider::<Http>::try_from(rpc_url)?,
            wallet: private_key.parse()?,
            contract_address: contract_address.parse()?,
        })
    }
}

impl SolanaAgent {
    pub fn new(config: ChainConfig) -> Result<Self, ClientError> {
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
