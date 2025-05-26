use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;

use ethers::abi::{Abi, RawLog};
use ethers::contract::{Contract, EthEvent, EthLogDecode};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::types::{Address, H256, U256};
use serde_json::Value;
use zescrow_core::{ChainConfig, ChainMetadata, EscrowMetadata, EscrowParams, EscrowState};

use crate::error::{AgentError, ClientError, Result};
use crate::Agent;

/// Factory ABI for encoding/decoding calls and events
const ESCROW_FACTORY_JSON: &str = include_str!(
    "../../adapters/ethereum/artifacts/contracts/EscrowFactory.sol/EscrowFactory.json"
);

/// ABI for the `EscrowCreated` event
#[derive(Clone, Debug, EthEvent)]
#[ethevent(
    name = "EscrowCreated",
    abi = "EscrowCreated(
    address indexed creator,
    address indexed escrowAddress,
    address indexed recipient,
    uint256 amount,
    uint256 finishAfter,
    uint256 cancelAfter,
    bool hasConditions)"
)]
struct EscrowCreatedEvent {
    creator: Address,
    escrow_address: Address,
    recipient: Address,
    amount: U256,
    finish_after: U256,
    cancel_after: U256,
    has_conditions: bool,
}

/// Escrow agent for interacting with the Ethereum network
pub struct EthereumAgent {
    // Ethereum JSON-RPC provider
    provider: Provider<Http>,
    // Escrow creator wallet
    sender_wallet: LocalWallet,
    // Escrow beneficiary wallet
    recipient_wallet: Option<LocalWallet>,
}

impl EthereumAgent {
    pub fn new(config: &ChainConfig, recipient_wallet: Option<LocalWallet>) -> Result<Self> {
        let ChainConfig::Ethereum {
            rpc_url,
            sender_private_key,
            ..
        } = config
        else {
            return Err(ClientError::ConfigMismatch);
        };

        Ok(Self {
            provider: Provider::<Http>::try_from(rpc_url)?,
            sender_wallet: sender_private_key.parse()?,
            recipient_wallet,
        })
    }
}

#[async_trait::async_trait]
impl Agent for EthereumAgent {
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        let client = Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            self.sender_wallet.clone(),
        ));

        let artifact: Value = serde_json::from_str(ESCROW_FACTORY_JSON)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;
        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| ClientError::Serialization("Missing ABI".into()))?
            .to_string();
        let abi = serde_json::from_str::<Abi>(&abi_json)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;
        let factory_addr_str = params.chain_config.eth_escrow_factory_contract()?;
        let factory_addr = Address::from_str(&factory_addr_str)?;

        let factory = Contract::new(factory_addr, abi.clone(), client.clone());

        // Escrow params
        let recipient = Address::from_str(&params.recipient.to_string())?;
        let finish_after = params.finish_after.unwrap_or_default();
        let cancel_after = params.cancel_after.unwrap_or_default();
        let has_conditions = params.has_conditions;
        let verifier_addr = params.chain_config.eth_verifier_contract()?;
        let verifier = Address::from_str(&verifier_addr)?;
        let amount = U256::from(params.asset.amount());

        // Send `createEscrow` transaction, funding with `amount`
        let call = factory
            .method::<_, H256>(
                "createEscrow",
                (
                    recipient,
                    finish_after,
                    cancel_after,
                    has_conditions,
                    verifier,
                ),
            )
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .value(amount);
        let pending_tx = call
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        // Await mined receipt
        let receipt = pending_tx
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .ok_or(ClientError::TxDropped)?;

        let mut escrow_addr = Address::zero();
        for log in receipt.logs.iter() {
            let raw_log = RawLog {
                topics: log.topics.clone(),
                data: log.data.to_vec(),
            };
            if let Ok(parsed) = <EscrowCreatedEvent as EthLogDecode>::decode_log(&raw_log) {
                escrow_addr = parsed.escrow_address;
                break;
            }
        }

        if escrow_addr == Address::zero() {
            return Err(ClientError::MissingEvent("EscrowCreated".into()));
        }

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
        let recipient_wallet = self
            .recipient_wallet
            .as_ref()
            .ok_or_else(|| ClientError::Keypair("Recipient wallet not provided".to_string()))?;
        let client = Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            recipient_wallet.to_owned(),
        ));

        let artifact: Value = serde_json::from_str(ESCROW_FACTORY_JSON)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;
        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| ClientError::Serialization("Missing ABI".into()))?
            .to_string();
        let abi = serde_json::from_str::<Abi>(&abi_json)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;

        let factory_addr_str = metadata.chain_config.eth_escrow_factory_contract()?;
        let factory_addr = Address::from_str(&factory_addr_str)?;
        let factory = Contract::new(factory_addr, abi, client);

        let escrow_addr = metadata.chain_data.get_eth_contract_address()?;
        let escrow_addr = Address::from_str(&escrow_addr)?;

        // TODO: set up RISC Zero prover API call
        let proof_data: Vec<u8> = vec![];

        factory
            .method::<_, H256>("finishEscrow", (escrow_addr, proof_data))
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        Ok(())
    }

    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let client = Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            self.sender_wallet.clone(),
        ));

        let artifact: Value = serde_json::from_str(ESCROW_FACTORY_JSON)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;
        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| ClientError::Serialization("Missing ABI".into()))?
            .to_string();
        let abi = serde_json::from_str::<Abi>(&abi_json)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;

        let factory_addr_str = metadata.chain_config.eth_escrow_factory_contract()?;
        let factory_addr = Address::from_str(&factory_addr_str)?;
        let factory = Contract::new(factory_addr, abi, client);

        let escrow_addr = metadata.chain_data.get_eth_contract_address()?;
        let escrow_addr = Address::from_str(&escrow_addr)?;

        factory
            .method::<_, H256>("cancelEscrow", escrow_addr)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        Ok(())
    }
}
