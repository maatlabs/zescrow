# Zescrow

Zescrow (for zero-knowledge escrow) is a trust-minimized generic implementation of an escrow program.

## Goals

1. **Privacy-Preserving** - Reveal only necessary transaction details to counterparties  
2. **Chain-Agnostic** - Deploy same escrow logic across L1s/L2s via lightweight adapters  
3. **Dispute Minimization** - Cryptographic proof of condition satisfaction preempts 90%+ conflicts  

## Core Features  

- ZK-proof of valid state transitions (initialized → funded → released/disputed)  
- Confidential amounts & participant identities via commitments  
- Chain-agnostic verification via RISC Zero zkVM proofs  
- Solana/Ethereum examples in `/demos`  

## Directory Structure

```text
project_name
├── Cargo.toml
├── host
│   ├── Cargo.toml
│   └── src
│       └── main.rs                    <-- [Host code goes here]
└── methods
    ├── Cargo.toml
    ├── build.rs
    ├── guest
    │   ├── Cargo.toml
    │   └── src
    │       └── method_name.rs         <-- [Guest code goes here]
    └── src
        └── lib.rs
```

## Contributing

Thank you for considering contributing to this project! All contributions large and small are actively accepted.

- To get started, please read the [contribution guidelines](https://github.com/maatlabs/zescrow/blob/main/CONTRIBUTING.md).

- Browse [Good First Issues](https://github.com/maatlabs/zescrow/labels/good%20first%20issue).

## License

Licensed under [Apache License, Version 2.0](https://github.com/maatlabs/zescrow/blob/main/LICENSE).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this codebase by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.
