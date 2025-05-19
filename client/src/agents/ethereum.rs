use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;

use ethers::abi::Abi;
use ethers::contract::{Contract, ContractFactory};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::types::{Address, H256, U256};
use ethers::utils::hex;
use serde_json::Value;
use zescrow_core::{ChainConfig, ChainMetadata, EscrowMetadata, EscrowParams, EscrowState};

use crate::error::{AgentError, ClientError, Result};
use crate::Agent;

const ESCROW_JSON: &str =
    include_str!("../../../adapters/ethereum/artifacts/contracts/Escrow.sol/Escrow.json");

/// Escrow agent for interacting with the Ethereum network
pub struct EthereumAgent {
    // Ethereum JSON-RPC provider
    provider: Provider<Http>,
    // Wallet for signing transactions
    wallet: LocalWallet,
}

impl EthereumAgent {
    pub fn new(config: &ChainConfig) -> Result<Self> {
        let ChainConfig::Ethereum {
            rpc_url,
            private_key,
            ..
        } = config
        else {
            return Err(ClientError::ConfigMismatch);
        };

        Ok(Self {
            provider: Provider::<Http>::try_from(rpc_url)?,
            wallet: private_key.parse()?,
        })
    }
}

#[async_trait::async_trait]
impl Agent for EthereumAgent {
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        let client = Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            self.wallet.clone(),
        ));

        let artifact: Value = serde_json::from_str(ESCROW_JSON)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;
        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| ClientError::Serialization("Missing ABI".into()))?
            .to_string();
        let bytecode_hex = artifact
            .get("bytecode")
            .and_then(Value::as_str)
            .ok_or_else(|| ClientError::Serialization("Missing bytecode".into()))?;

        let abi = serde_json::from_str::<Abi>(&abi_json)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;
        let bytecode = hex::decode(&bytecode_hex[2..])
            .map_err(|e| ClientError::Serialization(e.to_string()))?;

        let factory = ContractFactory::new(abi.clone(), bytecode.into(), client.clone());

        // Escrow params
        let recipient = Address::from_str(&params.recipient.to_string())?;
        let finish_after = params.finish_after.unwrap_or_default() as u64;
        let cancel_after = params.cancel_after.unwrap_or_default() as u64;
        let has_conditions = params.has_conditions;
        let verifier_addr = params
            .chain_config
            .eth_verifier_contract()
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        let verifier = Address::from_str(&verifier_addr)?;

        let deployer = factory
            .deploy((
                recipient,
                finish_after,
                cancel_after,
                has_conditions,
                verifier,
            ))
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        let escrow_addr = deployer.address();
        let escrow = Contract::new(escrow_addr, abi.clone(), client.clone());
        escrow
            .method::<_, H256>("deposit", ())
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .value(U256::from(params.asset.amount()))
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        Ok(EscrowMetadata {
            chain_config: params.chain_config.clone(),
            asset: params.asset.clone(),
            sender: params.sender.clone(),
            recipient: params.recipient.clone(),
            has_conditions,
            chain_data: ChainMetadata::Ethereum {
                escrow_address: escrow_addr.to_string(),
            },
            state: EscrowState::Funded,
        })
    }

    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        // TODO: set up RISC Zero prover API call
        let proof_bytes: Vec<u8> = vec![];

        let client = Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            self.wallet.clone(),
        ));

        let artifact: Value = serde_json::from_str(ESCROW_JSON)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;
        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| ClientError::Serialization("Missing ABI".into()))?
            .to_string();
        let abi = serde_json::from_str::<Abi>(&abi_json)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;

        let escrow_addr = metadata
            .chain_data
            .get_eth_contract_address()
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        let escrow_addr = Address::from_str(&escrow_addr)?;

        let escrow = Contract::new(escrow_addr, abi, client);

        escrow
            .method::<_, H256>("finishEscrow", proof_bytes)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        Ok(())
    }

    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let client = Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            self.wallet.clone(),
        ));

        let artifact: Value = serde_json::from_str(ESCROW_JSON)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;
        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| ClientError::Serialization("Missing ABI".into()))?
            .to_string();
        let abi = serde_json::from_str::<Abi>(&abi_json)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;

        let escrow_addr = metadata
            .chain_data
            .get_eth_contract_address()
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        let escrow_addr = Address::from_str(&escrow_addr)?;

        let escrow = Contract::new(escrow_addr, abi, client);

        escrow
            .method::<_, H256>("cancelEscrow", ())
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        Ok(())
    }
}
