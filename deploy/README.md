# Zescrow Deployment Guide

Deploy and interact with Zescrow escrows on Solana devnet and Ethereum Sepolia.

## Quick Start

```bash
# 1. Set up environment
cp deploy/.env.template .env
# Edit .env with your keys

# 2. Deploy contracts
./deploy/solana/run.sh      # For Solana devnet
./deploy/ethereum/run.sh    # For Ethereum Sepolia

# 3. Copy the appropriate config template
cp deploy/solana/escrow_params.json deploy/escrow_params.json
# Or for Ethereum:
# cp deploy/ethereum/escrow_params.json deploy/escrow_params.json

# 4. Create an escrow
cargo run --release -p zescrow-client -- create
```

## Prerequisites

### Solana

- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v0.32.1+)
- Devnet SOL (~3 SOL for deployment)

```bash
# Configure for devnet
solana config set --url https://api.devnet.solana.com

# Get devnet SOL
solana airdrop 2
```

### Ethereum

- [Node.js](https://nodejs.org/) (v18+)
- Sepolia ETH (use a [faucet](https://sepoliafaucet.com/))
- (Optional) [Etherscan API key](https://etherscan.io/apis) for verification

## Directory Structure

```sh
deploy/
├── solana/
│   ├── escrow_params.json    # Devnet config template (copy to deploy/)
│   └── run.sh                # Program deployment script
├── ethereum/
│   ├── escrow_params.json    # Sepolia config template (copy to deploy/)
│   └── run.sh                # Contract deployment script
├── escrow_conditions.json    # ZK conditions template
├── .env.template             # Environment variables template
└── README.md                 # This file

# Generated at runtime (git-ignored):
# ├── escrow_params.json      # Active config (copied from solana/ or ethereum/)
# ├── escrow_metadata.json    # Output from 'create' command
# └── proof_data.json         # ZK proof data
```

## Environment Setup

1. Copy the template to project root:

```bash
cp deploy/.env.template .env
```

1. Fill in your values:

```bash
# Solana
SOLANA_SENDER_KEYPAIR_PATH=~/.config/solana/id.json
SOLANA_SENDER_PUBKEY=YourPublicKey
SOLANA_RECIPIENT_PUBKEY=RecipientPublicKey

# Ethereum
ETHEREUM_SEPOLIA_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY
ETHEREUM_SENDER_PRIVATE_KEY=your_private_key
ETHEREUM_SENDER_ADDRESS=0xYourAddress
ETHEREUM_RECIPIENT_ADDRESS=0xRecipientAddress
ESCROW_CONTRACT_ADDRESS=0xDeployedContract
```

## Solana Deployment

### Deploy Program

```bash
./deploy/solana/run.sh
```

This builds and deploys the Anchor program to devnet. Note the Program ID.

### Create Escrow

```bash
# Copy config template
cp deploy/solana/escrow_params.json deploy/

# Create escrow
cargo run --release -p zescrow-client -- create
```

### Complete Escrow

```bash
# Release to recipient
cargo run --release -p zescrow-client -- finish --recipient ~/.config/solana/recipient.json

# Or cancel/refund
cargo run --release -p zescrow-client -- cancel
```

## Ethereum Deployment

### Deploy Contract

```bash
# Ensure ETHEREUM_SENDER_PRIVATE_KEY is set
./deploy/ethereum/run.sh
```

Note the deployed contract address and add it to your `.env`:

```bash
ESCROW_CONTRACT_ADDRESS=0x...
```

### Create Escrow

```bash
# Copy config template
cp deploy/ethereum/escrow_params.json deploy/

# Create escrow
cargo run --release -p zescrow-client -- create
```

### Complete Escrow

```bash
# Release to recipient (private key without 0x)
cargo run --release -p zescrow-client -- finish --recipient <RECIPIENT_PRIVATE_KEY>

# Or cancel/refund
cargo run --release -p zescrow-client -- cancel
```

## Cryptographic Conditions

For escrows with ZK conditions, use the `generate` command:

```bash
# Hashlock (SHA-256 preimage)
cargo run -p zescrow-client -- generate hashlock --preimage ./secret.txt

# Ed25519 signature
cargo run -p zescrow-client -- generate ed25519 \
  --pubkey <hex> --msg <hex> --sig <hex>

# Threshold (M-of-N)
cargo run -p zescrow-client -- generate threshold \
  --subconditions cond1.json cond2.json cond3.json \
  --threshold 2
```

Set `"has_conditions": true` in your escrow_params.json when using conditions.

## Configuration Reference

### Environment Variables

| Variable                      | Description                     |
| ----------------------------- | ------------------------------- |
| `SOLANA_SENDER_KEYPAIR_PATH`  | Path to sender's Solana keypair |
| `SOLANA_SENDER_PUBKEY`        | Sender's public key (base58)    |
| `SOLANA_RECIPIENT_PUBKEY`     | Recipient's public key (base58) |
| `ETHEREUM_SEPOLIA_RPC_URL`    | Sepolia JSON-RPC endpoint       |
| `ETHEREUM_SENDER_PRIVATE_KEY` | Sender's private key (no 0x)    |
| `ETHEREUM_SENDER_ADDRESS`     | Sender's address (0x...)        |
| `ETHEREUM_RECIPIENT_ADDRESS`  | Recipient's address (0x...)     |
| `ESCROW_CONTRACT_ADDRESS`     | Deployed contract address       |

### escrow_params.json Fields

| Field                            | Description                                     |
| -------------------------------- | ----------------------------------------------- |
| `chain_config.chain`             | `"solana"` or `"ethereum"`                      |
| `chain_config.rpc_url`           | Network RPC endpoint                            |
| `chain_config.sender_private_id` | Keypair path (Solana) or private key (Ethereum) |
| `chain_config.agent_id`          | Program ID or contract address                  |
| `asset.kind`                     | `"native"` for SOL/ETH                          |
| `asset.amount`                   | Amount in smallest unit (lamports/wei)          |
| `finish_after`                   | Slot/block after which release is allowed       |
| `cancel_after`                   | Slot/block after which cancel is allowed        |
| `has_conditions`                 | `true` if ZK conditions apply                   |

## Deployed Addresses

| Network          | Address                                        |
| ---------------- | ---------------------------------------------- |
| Solana Devnet    | `8u5bT8xkx6X4qKuRnn7oeDdrE1v4jG1F749YzqP1Z7BQ` |
| Ethereum Sepolia | TBD (deploy with `./deploy/ethereum/run.sh`)   |
