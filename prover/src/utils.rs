use serde::{Deserialize, Serialize};
use serde_hex::{SerHexOpt, Strict};
use sha2::{Digest, Sha256};
use zescrow_core::condition::Condition;
use zescrow_core::escrow::{Escrow, EscrowState};
use zescrow_core::identity::{Asset, Party};

// TODO: 1. handle all `Condition`s
//       2. create a proper `AssetId` with methods
pub fn map_escrow_metadata(meta: EscrowMetadata) -> Escrow {
    let condition = meta.condition.map(|preimage| {
        let hash: [u8; 32] = Sha256::digest(&preimage)
            .as_slice()
            .try_into()
            .expect("hash length mismatch");
        Condition::Preimage {
            hash,
            preimage: preimage.to_vec(),
        }
    });

    // derive asset ID from chain_data for now:
    let asset_id = match &meta.chain_data {
        ChainMetadata::Ethereum {
            contract_address, ..
        } => contract_address.clone(),
        ChainMetadata::Solana { program_id, .. } => program_id.clone(),
    };

    Escrow {
        asset: Asset::Fungible {
            id: asset_id,
            amount: meta.amount,
        },
        recipient: Party {
            identity_hash: meta.recipient,
        },
        sender: Party {
            identity_hash: meta.sender,
        },
        condition,
        created_block: meta.created_block,
        state: EscrowState::Released,
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
    /// Optional cryptographic (e.g., SHA-256 preimage) condition
    #[serde(with = "SerHexOpt::<Strict>")]
    pub condition: Option<[u8; 32]>,
    /// Block height when this escrow was created.
    pub created_block: u64,
    /// Chain-specific metadata for smart contracts/programs
    #[serde(flatten)]
    pub chain_data: ChainMetadata,
}
