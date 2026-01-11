//! Ethereum blockchain agent implementation.
//!
//! Provides [`EthereumAgent`] for interacting with the Zescrow Ethereum
//! smart contract. Supports creating, finishing, and canceling escrows
//! on Ethereum and EVM-compatible chains.

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

use crate::error::ClientError;
use crate::{Agent, Result};

/// Escrow contract ABI embedded at compile time.
const ESCROW_JSON: &str = include_str!("../abi/Escrow.json");

// Contract method names.
const CREATE_ESCROW: &str = "createEscrow";
const FINISH_ESCROW: &str = "finishEscrow";
const CANCEL_ESCROW: &str = "cancelEscrow";

/// The `EscrowCreated` event emitted when a new escrow is created.
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

/// Ethereum blockchain agent for escrow operations.
///
/// Manages interactions with the Zescrow Ethereum smart contract,
/// including transaction signing and event parsing.
pub struct EthereumAgent {
    /// Ethereum JSON-RPC provider.
    pub provider: Provider<Http>,
    /// Contract instance signed by the sender.
    escrow_as_sender: Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    /// Contract instance signed by the recipient (optional, for finish operations).
    escrow_as_recipient: Option<Contract<SignerMiddleware<Provider<Http>, LocalWallet>>>,
}

impl EthereumAgent {
    /// Creates a new Ethereum agent from chain configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Chain configuration containing RPC URL and sender key
    /// * `recipient` - Optional recipient wallet for finish operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - RPC connection fails
    /// - Contract ABI parsing fails
    /// - Wallet parsing fails
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
            .map_err(|e| ClientError::ethereum("get_chainid", e))?
            .as_u64();
        debug!(%chain_id, "Connected to Ethereum");

        let abi = Self::load_contract_abi()?;
        let escrow_addr = Address::from_str(agent_id)?;

        let escrow_as_sender = Self::create_contract_instance(
            &provider,
            escrow_addr,
            abi.clone(),
            sender_private_id,
            chain_id,
        )?;

        let escrow_as_recipient = recipient.map(|wallet| {
            let signer = Arc::new(SignerMiddleware::new(
                provider.clone(),
                wallet.with_chain_id(chain_id),
            ));
            Contract::new(escrow_addr, abi, signer)
        });

        Ok(Self {
            provider,
            escrow_as_sender,
            escrow_as_recipient,
        })
    }

    /// Loads and parses the contract ABI from embedded JSON.
    fn load_contract_abi() -> Result<Abi> {
        let artifact: Value = serde_json::from_str(ESCROW_JSON)
            .map_err(|e| ClientError::ethereum("parse_artifact", e))?;

        let abi_json = artifact
            .get("abi")
            .ok_or_else(|| ClientError::ethereum("load_abi", "missing ABI field in artifact"))?
            .to_string();

        serde_json::from_str::<Abi>(&abi_json).map_err(|e| ClientError::ethereum("parse_abi", e))
    }

    /// Creates a contract instance with a signing middleware.
    fn create_contract_instance(
        provider: &Provider<Http>,
        address: Address,
        abi: Abi,
        private_key: &str,
        chain_id: u64,
    ) -> Result<Contract<SignerMiddleware<Provider<Http>, LocalWallet>>> {
        let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        let signer = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
        Ok(Contract::new(address, abi, signer))
    }

    /// Extracts the escrow ID from transaction events.
    async fn extract_escrow_id(&self, block_hash: H256) -> Result<u64> {
        let events = self
            .escrow_as_sender
            .event::<EscrowCreatedEvent>()
            .at_block_hash(block_hash)
            .query()
            .await
            .map_err(|e| ClientError::ethereum("query_events", e))?;

        let event = events
            .into_iter()
            .next()
            .ok_or_else(|| ClientError::MissingEvent("EscrowCreated event not found".into()))?;

        let escrow_id = event.escrow_id;
        if escrow_id.is_zero() {
            return Err(ClientError::MissingEvent("escrow_id is zero".into()));
        }

        Ok(escrow_id.as_u64())
    }

    /// Returns the recipient contract instance, or an error if not configured.
    fn recipient_contract(
        &self,
    ) -> Result<&Contract<SignerMiddleware<Provider<Http>, LocalWallet>>> {
        self.escrow_as_recipient
            .as_ref()
            .ok_or_else(|| ClientError::ethereum(FINISH_ESCROW, "recipient wallet not configured"))
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

        info!(
            "Sending {} transaction with amount {}",
            CREATE_ESCROW, amount
        );

        let call = self
            .escrow_as_sender
            .method::<_, H256>(CREATE_ESCROW, (recipient, finish_after, cancel_after))
            .map_err(|e| ClientError::ethereum(CREATE_ESCROW, e))?
            .value(amount);

        let pending_tx = call
            .send()
            .await
            .map_err(|e| ClientError::ethereum(CREATE_ESCROW, e))?;

        let receipt = pending_tx
            .await
            .map_err(|e| ClientError::ethereum(CREATE_ESCROW, e))?
            .ok_or_else(|| ClientError::tx_dropped("createEscrow transaction not confirmed"))?;

        info!(tx_hash = ?receipt.transaction_hash, "Transaction mined");

        let block_hash = receipt
            .block_hash
            .ok_or_else(|| ClientError::MissingEvent("no block hash in receipt".into()))?;

        let escrow_id = self.extract_escrow_id(block_hash).await?;
        info!("{} confirmed for escrow ID {}", CREATE_ESCROW, escrow_id);

        Ok(EscrowMetadata {
            params: params.clone(),
            state: ExecutionState::Funded,
            escrow_id: Some(escrow_id),
        })
    }

    async fn finish_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let id = metadata
            .escrow_id
            .ok_or_else(|| ClientError::ethereum(FINISH_ESCROW, "missing escrow_id"))?;

        let contract = self.recipient_contract()?;

        info!("Sending {} transaction for escrow ID {}", FINISH_ESCROW, id);

        contract
            .method::<_, ()>(FINISH_ESCROW, U256::from(id))
            .map_err(|e| ClientError::ethereum(FINISH_ESCROW, e))?
            .send()
            .await
            .map_err(|e| ClientError::ethereum(FINISH_ESCROW, e))?
            .await
            .map_err(|e| ClientError::ethereum(FINISH_ESCROW, e))?;

        info!("{} confirmed for escrow ID {}", FINISH_ESCROW, id);
        Ok(())
    }

    async fn cancel_escrow(&self, metadata: &EscrowMetadata) -> Result<()> {
        let id = metadata
            .escrow_id
            .ok_or_else(|| ClientError::ethereum(CANCEL_ESCROW, "missing escrow_id"))?;

        info!("Sending {} transaction for escrow ID {}", CANCEL_ESCROW, id);

        self.escrow_as_sender
            .method::<_, ()>(CANCEL_ESCROW, U256::from(id))
            .map_err(|e| ClientError::ethereum(CANCEL_ESCROW, e))?
            .send()
            .await
            .map_err(|e| ClientError::ethereum(CANCEL_ESCROW, e))?
            .await
            .map_err(|e| ClientError::ethereum(CANCEL_ESCROW, e))?;

        info!("{} confirmed for escrow ID {}", CANCEL_ESCROW, id);
        Ok(())
    }
}
