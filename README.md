# Zescrow

Zescrow (for zero-knowledge escrow) is a trust-minimized, chain-agnostic implementation of an escrow program using the RISC Zero zkVM as the zero-knowledge prover/verifier.

> [!WARNING]

**_This project is not audited, and it's currently under active development. Until `v1.0`, please do not deploy in production_**.

## Goals

1. _Privacy-Preserving_: reveal only necessary transaction details to counterparties  
2. _Chain-Agnostic_: deploy same escrow logic across L1s/L2s via lightweight agents  
3. _Dispute Minimization_: cryptographic proof of condition fulfillment preempts conflicts  

## Core Features  

- ZK proving/verification of cryptographic conditions e.g., hash-lock, Ed25519 signature over a message, etc.
- Chain-agnostic ZK verification via RISC Zero receipts.
- Solana programs and EVM smart contracts in `/agent`.

## Architecture

### Project Structure

- `agent` (Contains chain-specific escrow programs/smart contracts)
- `client` (Contains the RISC Zero guest and host, as well as mechanisms for executing on-chain `agent` actions)
- `core` (Main library that exposes types and functionality for the `client`)
- `templates` (Contains `escrow_params.json`, `escrow_metadata.json`, `escrow_conditions.json`; these are files that specify escrow parameters/inputs, escrow transaction output, and optional cryptographic conditions for the "Finish" command, respectively.)

### High-Level Flow

1. Build and deploy a chain-specific agent (via the `/agent`).
2. Specify the parameters of the escrow (via the `/templates/escrow_params.json`).
3. `Create` an escrow transaction (via the `client`).
4. To release an escrow, execute the `Finish` command of the `client`.
5. To cancel/refund an escrow, execute the `Cancel` command of the `client`.

![Zescrow architecture diagram](/assets/zescrow-arch.png)

## Usage

### Prerequisites

1. Please ensure that [rustup] is installed. The [`rust-toolchain.toml`][rust-toolchain] file will be used by `cargo` to automatically install the correct version.

2. Install the [risc0-toolchain], since the prover of the `zescrow-client` requires it.

### End-to-End Demo

1. Clone the repository:

    ```sh
    git clone https://github.com/maatlabs/zescrow.git
    cd zescrow
    ```

2. To create an escrow end-to-end on Ethereum (and other EVM-compatible chains), please follow the [ethereum-demo][ethereum-demo].

3. To create an escrow end-to-end on Solana, please follow the [solana-demo][solana-demo].

## Contributing

Thank you for considering contributing to this project! All contributions large and small are actively accepted.

- To get started, please read the [contribution guidelines](https://github.com/maatlabs/zescrow/blob/main/CONTRIBUTING.md).

- Browse [Good First Issues](https://github.com/maatlabs/zescrow/labels/good%20first%20issue).

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this codebase by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[ethereum-demo]: demos/ethereum_demo.md
[risc0-toolchain]: https://dev.risczero.com/api/zkvm/quickstart#1-install-the-risc-zero-toolchain
[rust-toolchain]: rust-toolchain.toml
[rustup]: https://rustup.rs
[solana-demo]: demos/solana_demo.md
