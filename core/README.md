# Zescrow Core

Core library for Zescrow: zero-knowledge escrows via the RISC Zero zkVM.

## Modules

- `asset` — chain-agnostic asset types (coins, tokens, NFTs, LP shares)  
- `condition` — cryptographic conditions (hashlocks, signatures, threshold)
- `escrow` — off-chain escrow state machine with ZK proofs
- `identity` — identity parsing and format conversions (hex, Base58, Base64)  
- `interface` — schemas, I/O helpers  
- `error` — typed errors  
- `bignum` — wrapper around BigUint.
- `serde` — JSON (de)serialization helpers (`json` feature)

## Optional dependencies

| Feature | Dependencies                                 |
|---------|----------------------------------------------|
| `bincode` (default) | `bincode` (derive)                  |
| `json`      | `serde`, `serde_json`, `serde_bytes`, `serde_with` |

## Quickstart

Add the crate as a dependency in your `Cargo.toml` (enable the `json` feature if you want `serde` support):

```toml
[dependencies]
zescrow-core = { version = "0.1", features = ["json"] }
```

```rust
use zescrow_core::{
    Asset, Condition, Escrow, EscrowError, ExecutionState, ID, Party, Result
};
use num_bigint::BigUint;
use sha2::{Digest, Sha256};

fn execute_escrow() -> Result<()> {
    let sender = Party::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")?;
    let recipient = Party::from_str("0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8")?;

    let asset = Asset::Token {
        chain: Chain::Ethereum,
        contract: ID::from_str("0xdeadbeef")?,
        amount: BigUint::from(1_000u64).into(),
        decimals: 18,
    };

    let preimage = b"secret".to_vec();
    let hash = Sha256::digest(&preimage);
    let condition = Condition::preimage(hash.into(), preimage);

    let mut escrow = Escrow::new(sender, recipient, asset, Some(condition));
    escrow.state = ExecutionState::Funded; // Advance execution state

    let exec_state = escrow.execute()?;
    if exec_state != ExecutionState::ConditionsMet {
        return Err(EscrowError::InvalidState);
    }

    Ok(())
}
```

## Documentation

<https://docs.rs/zescrow-core>

## License

Licensed under either [Apache License, Version 2.0](../LICENSE-APACHE)  
or [MIT License](../LICENSE-MIT) at your option.
