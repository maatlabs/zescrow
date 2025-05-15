use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use zescrow_core::Chain;

const TEMPLATES_DIR: &str = "templates";

/// Reads chain-specific configuration given a target `chain`
/// (e.g., ethereum, solana).
pub fn load_chain_config(chain: &Chain) -> anyhow::Result<ChainConfig> {
    let config_path = format!("{}/{}_config.json", TEMPLATES_DIR, chain.as_ref());
    load_escrow_input_data(&config_path)
}

/// Reads JSON-encoded escrow params and chain-specific configs
/// from the given `path`.
pub fn load_escrow_input_data<P, T>(path: P) -> anyhow::Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            anyhow::bail!(
                "Chain config file {:?} not found.
                Please create a <CHAIN>_config.json in /templates",
                path
            );
        }
        Err(e) => return Err(e).context(format!("opening file {:?}", path)),
    };
    serde_json::from_reader(file).with_context(|| format!("parsing JSON from {:?}", path))
}

/// Writes JSON-encoded `data` to the given `path`,
/// creating parent directories as needed.
pub fn save_escrow_metadata<P, T>(path: P, data: &T) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {:?}", parent))?;
    }
    let file = File::create(path).with_context(|| format!("creating file {:?}", path))?;
    serde_json::to_writer_pretty(file, data)
        .with_context(|| format!("serializing to JSON to {:?}", path))
}

/// Chain-specific network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChainConfig {
    /// Ethereum network configuration
    Ethereum {
        /// JSON-RPC endpoint URL
        rpc_url: String,
        /// Private key
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
