use hex::FromHex;
use serde::{Deserialize, Serialize};
use zescrow_core::{Condition as CoreCondition, Escrow, EscrowError};

// TODO: create a proper `AssetId` with methods
pub fn to_escrow(data: EscrowMetadata) -> Result<Escrow, EscrowError> {
    let _condition = data.condition.map(map_conditions).transpose()?;

    // derive asset ID from chain_data for now:
    let _asset_id = match &data.chain_data {
        ChainMetadata::Ethereum {
            contract_address, ..
        } => contract_address.clone(),
        ChainMetadata::Solana { program_id, .. } => program_id.clone(),
    };

    // Ok(Escrow {
    //     asset: Asset::Fungible {
    //         id: asset_id,
    //         amount: data.amount,
    //     },
    //     recipient: Party {
    //         identity_hash: data.recipient,
    //     },
    //     sender: Party {
    //         identity_hash: data.sender,
    //     },
    //     condition,
    //     created_block: data.created_block,
    //     state: EscrowState::Released,
    // })
    todo!()
}

fn map_conditions(c: Condition) -> Result<CoreCondition, EscrowError> {
    Ok(match c {
        Condition::Preimage { hash, preimage } => {
            let hash_b = Vec::from_hex(&hash)?;
            let hash: [u8; 32] = hash_b.try_into().map_err(|_| EscrowError::InvalidLength)?;
            let preimage = Vec::from_hex(&preimage)?;
            CoreCondition::Preimage { hash, preimage }
        }
        Condition::Ed25519 {
            public_key,
            signature,
            message,
        } => {
            let public_key: [u8; 32] = Vec::from_hex(&public_key)?
                .try_into()
                .map_err(|_| EscrowError::InvalidLength)?;
            let message = Vec::from_hex(&message)?;
            let signature = Vec::from_hex(&signature)?;
            CoreCondition::Ed25519 {
                public_key,
                signature,
                message,
            }
        }
        Condition::Secp256k1 {
            public_key,
            signature,
            message,
        } => {
            let public_key = Vec::from_hex(&public_key)?;
            let message = Vec::from_hex(&message)?;
            let signature = Vec::from_hex(&signature)?;
            CoreCondition::Secp256k1 {
                public_key,
                signature,
                message,
            }
        }
        Condition::Threshold {
            threshold,
            subconditions,
        } => {
            let subs = subconditions
                .into_iter()
                .map(map_conditions)
                .collect::<Result<_, _>>()?;
            CoreCondition::Threshold {
                threshold,
                subconditions: subs,
            }
        }
    })
}

/// Metadata returned from escrow creation
#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowMetadata {
    /// Original target blockchain network
    pub chain: Chain,
    /// Escrow creator's blockchain address
    pub sender: String,
    /// Escrow beneficiary's blockchain address
    pub recipient: String,
    /// Locked amount in native token units
    pub amount: u64,
    /// Optional UNIX timestamp after which funds can be released
    pub finish_after: Option<i64>,
    /// Optional UNIX timestamp after which sender can reclaim funds
    pub cancel_after: Option<i64>,
    /// Block height when this escrow was created.
    pub created_block: u64,
    /// Optional cryptographic (e.g., SHA-256 preimage) condition
    #[serde(flatten)]
    pub condition: Option<Condition>,
    /// Chain-specific metadata for smart contracts/programs
    #[serde(flatten)]
    pub chain_data: ChainMetadata,
}

/// Types of crypto conditions that can be specified
/// during escrow creation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Condition {
    /// XRPL-style hashlock: SHA-256(preimage) == hash.
    Preimage { hash: String, preimage: String },

    /// Ed25519 signature over a message.
    Ed25519 {
        public_key: String,
        signature: String,
        message: String,
    },

    /// Secp256k1 signature over a message.
    Secp256k1 {
        public_key: String,
        signature: String,
        message: String,
    },

    /// Threshold SHA-256: at least `threshold` of `subconditions` must hold.
    Threshold {
        threshold: usize,
        subconditions: Vec<Condition>,
    },
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", json)
    }
}

/// Target blockchains
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    // Ethereum and other EVM-compatible chains
    Ethereum,
    Solana,
}

impl AsRef<str> for Chain {
    fn as_ref(&self) -> &str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Solana => "solana",
        }
    }
}

/// Chain-specific metadata for smart contracts/programs
#[derive(Debug, Clone, Serialize, Deserialize)]
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
