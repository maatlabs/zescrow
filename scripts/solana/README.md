# RISC Zero Solana Verifier Deploy Scripts

## Prerequisites

- Rust and Cargo
- Solana Tool Suite
- Node.js and Yarn
- Anchor Framework

## Installation

1. Install dependencies:

```bash
cd scripts/solana
yarn install
```

2. Configure environment:

```bash
cp example.env .env
# Edit .env with your configuration
```

## Deployment

Note: Deployment accounts need at minimum a 6 SOL balance by default and any non-deployment actions require an account with a 1 SOL minimum balance.

1. Deploy the router and initial verifier:

```bash
anchor keys sync
anchor build
yarn run client
yarn run deploy
```

2. (Optional) Add additional verifiers programs:

```bash
yarn run add
```
