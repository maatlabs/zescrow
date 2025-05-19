use std::{convert::TryFrom, sync::Arc};

use ethers::types::Address;
use ethers::{
    abi::Abi,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
};
use ethers::{contract::Contract, signers::LocalWallet};
use zescrow_core::interface::ChainConfig;
use zescrow_core::{EscrowMetadata, EscrowParams};

use crate::error::{ClientError, Result};
use crate::Agent;

const ESCROW_ABI_JSON: &str =
    include_str!("../../../adapters/ethereum/artifacts/contracts/Escrow.sol/Escrow.json");

/// Escrow agent for interacting with the Ethereum network
pub struct EthereumAgent {
    // Ethereum JSON-RPC provider
    provider: Provider<Http>,
    // Wallet for signing transactions
    wallet: LocalWallet,
    // On-chain escrow smart contract address
    contract_address: Address,
}

impl EthereumAgent {
    pub fn new(config: &ChainConfig) -> Result<Self> {
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

#[async_trait::async_trait]
impl Agent for EthereumAgent {
    async fn create_escrow(&self, _params: &EscrowParams) -> Result<EscrowMetadata> {
        todo!()
    }

    async fn finish_escrow(&self, _metadata: &EscrowMetadata) -> Result<()> {
        todo!()
    }

    async fn cancel_escrow(&self, _metadata: &EscrowMetadata) -> Result<()> {
        todo!()
    }
}
