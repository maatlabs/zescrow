# Zescrow

Zescrow (for zero-knowledge escrow) is a trust-minimized, chain-agnostic implementation of an escrow program using the RISC Zero zkVM as the zero-knowledge prover.

> [!WARNING]

**This project is not audited, and it's currently under active development. Until v1.0.0, please do not use in production**.

## Goals

1. **Privacy-Preserving** - Reveal only necessary transaction details to counterparties  
2. **Chain-Agnostic** - Deploy same escrow logic across L1s/L2s via lightweight adapters  
3. **Dispute Minimization** - Cryptographic proof of condition fulfillment preempts 90%+ conflicts  

## Core Features  

- ZK-proof of valid state transitions (initialized → funded → released/disputed)  
- Confidential amounts & participant identities via commitments  
- Chain-agnostic verification via RISC Zero zkVM proofs  
- Solana programs and EVM smart contracts in `/adapters`

## Architecture

### Project Structure

- `adapters` (Chain-specific escrow and ZK verifier programs/smart contracts)
- `client` (The CLI tool for creating, finishing, and/or cancelling escrows; interacts with `adapters`)
- `core` (The main library that exposes types and business logic for ZK computations)
- `prover` (The RISC Zero host/zkVM)
- `methods` (The RISC Zero guest)
- `templates` (Contains `escrow_params.json`, `escrow_metadata.json`, `escrow_conditions.json`; these are files that specify escrow parameters/inputs, escrow transaction output, and optional cryptographic conditions for "Release", respectively.)

### High-Level Flow

1. Build and deploy a chain-specific adapter (via the `/adapters`).
2. Specify the parameters of the escrow (via the `./templates/escrow_params.json`).
3. `Create` an escrow transaction (via the `client`).
4. To release an escrow with `has_conditions == false` (i.e., with no cryptographic conditions), execute the `Finish` command of the `client`.
To release an escrow that `has_conditions`, first run the `prover` to generate a valid `receipt` (zero-knowledge proof) and then execute `Finish` command of the `client`.
5. To cancel/refund an escrow, execute `Cancel` via the `client`.

![Zescrow architecture diagram](./assets/zescrow-arch.png)

Zooming in on the proof generation routine, here's the interaction between the `prover` and the on-chain `adapters`:

![Proof generation flow diagram](./assets/proof-gen-flow.png)

## Usage

### Prerequisites

1. Please ensure that [rustup] is installed. The [`rust-toolchain.toml`][rust-toolchain] file will be used by `cargo` to
automatically install the correct version.

2. If you intend to create escrows with cryptographic conditions then the [risc0-toolchain] must be installed, since the `zescrow-prover` requires it.

### End-to-End

1. Clone the repository:

```sh
git clone https://github.com/maatlabs/zescrow.git
```

2. To create an escrow end-to-end on Ethereum (and other EVM-compatible chains), please follow the [ethereum-demo][ethereum-demo]. To create an escrow end-to-end on Solana, please follow the [solana-demo][solana-demo].

## Contributing

Thank you for considering contributing to this project! All contributions large and small are actively accepted.

- To get started, please read the [contribution guidelines](https://github.com/maatlabs/zescrow/blob/main/CONTRIBUTING.md).

- Browse [Good First Issues](https://github.com/maatlabs/zescrow/labels/good%20first%20issue).

## License

Licensed under [Apache License, Version 2.0](./LICENSE).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this codebase by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.

[ethereum-demo]: demos/ethereum_demo.md
[risc0-toolchain]: https://dev.risczero.com/api/zkvm/quickstart#1-install-the-risc-zero-toolchain
[rust-toolchain]: rust-toolchain.toml
[rustup]: https://rustup.rs
[solana-demo]: demos/solana_demo.md
