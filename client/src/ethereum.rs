use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;

use ethers::abi::Abi;
use ethers::contract::{Contract, EthEvent};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, H256, U256};
use serde_json::Value;
use tracing::{debug, info};
use zescrow_core::{ChainConfig, EscrowMetadata, EscrowParams, ExecutionState};

use crate::error::{AgentError, ClientError};
use crate::{Agent, Result};

// Escrow contract ABI for encoding/decoding calls and events
const ESCROW_JSON: &str =
    include_str!("../../agent/ethereum/artifacts/contracts/Escrow.sol/Escrow.json");

// On-chain escrow operations.
const CREATE_ESCROW: &str = "createEscrow";
const FINISH_ESCROW: &str = "finishEscrow";
const CANCEL_ESCROW: &str = "cancelEscrow";

/// The `EscrowCreated` event
#[derive(Clone, Debug, EthEvent)]
#[ethevent(
    name = "EscrowCreated",
    abi = "EscrowCreated(uint256,address,address,uint256,uint256,uint256)"
)]
struct EscrowCreatedEvent {
    #[ethevent(indexed)]
    escrow_id: U256,
    #[ethevent(indexed)]
    sender: Address,
    #[ethevent(indexed)]
    recipient: Address,

    amount: U256,
    finish_after: U256,
    cancel_after: U256,
}

/// Escrow agent for interacting with the Ethereum network
pub struct EthereumAgent {
    /// Ethereum JSON-RPC provider
    pub provider: Provider<Http>,
    /// Contract instance as sender
    escrow_as_sender: Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    /// Contract instance as recipient
    escrow_as_recipient: Option<Contract<SignerMiddleware<Provider<Http>, LocalWallet>>>,
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

        let artifact: Value =
            serde_json::from_str(ESCROW_JSON).map_err(|e| AgentError::Ethereum(e.to_string()))?;
        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| AgentError::Ethereum("Missing ABI".into()))?
            .to_string();
        let abi = serde_json::from_str::<Abi>(&abi_json)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        let escrow_addr = Address::from_str(&agent_id)?;

        let escrow_as_sender = {
            let sender = sender_private_id
                .parse::<LocalWallet>()?
                .with_chain_id(chain_id);
            let sender = Arc::new(SignerMiddleware::new(provider.clone(), sender));
            Contract::new(escrow_addr, abi.clone(), sender)
        };
        let escrow_as_recipient = recipient.map(|wallet| {
            let recipient = Arc::new(SignerMiddleware::new(
                provider.clone(),
                wallet.with_chain_id(chain_id),
            ));
            Contract::new(escrow_addr, abi, recipient)
        });

        Ok(Self {
            provider,
            escrow_as_sender,
            escrow_as_recipient,
        })
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
        info!("Sending {CREATE_ESCROW} transaction with amount {}", amount);
        let call = self
            .escrow_as_sender
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
        let block_hash = receipt
            .block_hash
            .ok_or(ClientError::MissingEvent("no block hash".to_string()))?;
        let events = self
            .escrow_as_sender
            .event::<EscrowCreatedEvent>()
            .at_block_hash(block_hash)
            .query()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;

        let event = events
            .into_iter()
            .next()
            .ok_or(ClientError::MissingEvent("missing escrow_id".to_string()))?;

        let escrow_id = event.escrow_id;
        // escrowId > 0
        if escrow_id.is_zero() {
            return Err(ClientError::MissingEvent("zero escrow_id".into()));
        }
        let escrow_id = escrow_id.as_u64();
        info!(
            "{CREATE_ESCROW} transaction confirmed for escrow with ID {}",
            escrow_id
        );

        Ok(EscrowMetadata {
            params: params.clone(),
            state: ExecutionState::Funded,
            escrow_id: Some(escrow_id),
        })
    }

    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let id = metadata
            .escrow_id
            .ok_or(AgentError::Ethereum("Missing escrow_id".to_string()))?;
        let escrow_id: U256 = id.into();

        let escrow_as_recipient = self
            .escrow_as_recipient
            .as_ref()
            .ok_or(AgentError::Ethereum("Missing recipient key".to_string()))?;

        info!("Sending {FINISH_ESCROW} transaction for escrow with ID {id}");
        let call = escrow_as_recipient
            .method::<_, ()>(FINISH_ESCROW, escrow_id)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        call.send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        info!("{FINISH_ESCROW} transaction confirmed for escrow with ID {id}");

        Ok(())
    }

    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let id = metadata
            .escrow_id
            .ok_or(AgentError::Ethereum("Missing escrow_id".to_string()))?;
        let escrow_id: U256 = id.into();

        info!("Sending {CANCEL_ESCROW} transaction for ID {id}");
        self.escrow_as_sender
            .method::<_, ()>(CANCEL_ESCROW, escrow_id)
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .send()
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?
            .await
            .map_err(|e| AgentError::Ethereum(e.to_string()))?;
        info!("{CANCEL_ESCROW} transaction confirmed for ID {id}");

        Ok(())
    }
}
