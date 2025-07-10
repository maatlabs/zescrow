use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;

use ethers::abi::{Abi, RawLog};
use ethers::contract::{Contract, EthEvent, EthLogDecode};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, H256, U256};
use serde_json::Value;
use tracing::{debug, info, trace};
use zescrow_core::{ChainConfig, EscrowMetadata, EscrowParams, ExecutionState};

use crate::error::{AgentError, ClientError};
use crate::{Agent, Result};

// Factory ABI for encoding/decoding calls and events
const ESCROW_FACTORY_JSON: &str =
    include_str!("../../agent/ethereum/artifacts/contracts/EscrowFactory.sol/EscrowFactory.json");

// On-chain escrow operations.
const CREATE_ESCROW: &str = "createEscrow";
const FINISH_ESCROW: &str = "finishEscrow";
const CANCEL_ESCROW: &str = "cancelEscrow";

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
    uint256 cancelAfter)"
)]
struct EscrowCreatedEvent {
    creator: Address,
    escrow_address: Address,
    recipient: Address,
    amount: U256,
    finish_after: U256,
    cancel_after: U256,
}

/// Escrow agent for interacting with the Ethereum network
pub struct EthereumAgent {
    /// Ethereum JSON-RPC provider
    pub provider: Provider<Http>,
    // Factory contract
    factory: Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl EthereumAgent {
    pub async fn new(config: &ChainConfig, recipient: Option<LocalWallet>) -> Result<Self> {
        let ChainConfig {
            rpc_url,
            sender_private_id,
            agent_id,
            ..
        } = config;

        let provider = Provider::<Http>::try_from(rpc_url)?;
        let chain_id = provider
            .get_chainid()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .as_u64();
        debug!(%chain_id, "Connected to Ethereum");

        let sender = sender_private_id
            .parse::<LocalWallet>()?
            .with_chain_id(chain_id);
        let recipient = recipient.map(|w| w.with_chain_id(chain_id));
        debug!(has_recipient = recipient.is_some(), "Wallets configured");

        let artifact: Value = serde_json::from_str(ESCROW_FACTORY_JSON)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| AgentError::Ethereum("Missing ABI section".into()))?
            .to_string();
        let abi = serde_json::from_str::<Abi>(&abi_json)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        let client = Arc::new(SignerMiddleware::new(provider.clone(), sender));
        let factory_addr = Address::from_str(&agent_id)?;
        let factory = Contract::new(factory_addr, abi, client);

        Ok(Self { provider, factory })
    }
}

#[async_trait::async_trait]
impl Agent for EthereumAgent {
    async fn create_escrow(&self, params: &EscrowParams) -> Result<EscrowMetadata> {
        let recipient = Address::from_str(&params.recipient.to_string())?;
        let finish_after = params.finish_after.unwrap_or_default();
        let cancel_after = params.cancel_after.unwrap_or_default();
        let amount = U256::from_dec_str(&params.asset.amount().to_string())
            .map_err(|_| ClientError::AssetOverflow)?;

        // Send `createEscrow` transaction, funding with `amount`
        info!("Sending {CREATE_ESCROW} with amount {}", amount);
        let call = self
            .factory
            .method::<_, H256>(CREATE_ESCROW, (recipient, finish_after, cancel_after))
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
        info!(tx_hash = ?receipt.transaction_hash, "Transaction mined");

        // Decode event
        let mut escrow_addr = Address::zero();
        for log in &receipt.logs {
            trace!(topics = ?log.topics, "Parsing log");
            if let Ok(ev) = <EscrowCreatedEvent as EthLogDecode>::decode_log(&RawLog {
                topics: log.topics.clone(),
                data: log.data.to_vec(),
            }) {
                escrow_addr = ev.escrow_address;
                break;
            }
        }

        if escrow_addr.is_zero() {
            return Err(ClientError::MissingEvent("EscrowCreated".into()));
        }
        info!("{CREATE_ESCROW} transaction confirmed");

        Ok(EscrowMetadata {
            params: params.clone(),
            state: ExecutionState::Funded,
        })
    }

    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let escrow_addr = Address::from_str(&metadata.params.chain_config.agent_id)?;

        info!("Sending {FINISH_ESCROW} transaction");
        let call = self
            .factory
            .method::<_, ()>(FINISH_ESCROW, escrow_addr)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        call.send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        info!("{FINISH_ESCROW} transaction confirmed");

        Ok(())
    }

    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let escrow_addr = Address::from_str(&metadata.params.chain_config.agent_id)?;

        info!("Sending {CANCEL_ESCROW} transaction");
        self.factory
            .method::<_, ()>(CANCEL_ESCROW, escrow_addr)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        info!("{CANCEL_ESCROW} transaction confirmed");

        Ok(())
    }
}
