# Zescrow Deployment Guide

Deploy and interact with Zescrow escrows on Solana and Ethereum networks.

This guide covers both **local development** (test validators) and **devnet/testnet deployment** (devnet/Sepolia).

## Quick Start

```bash
# 1. Set up environment
cp deploy/.env.template .env
# Edit .env with your configuration

# 2. Deploy (choose network)
./deploy/solana/run.sh --network local          # Local test validator
./deploy/solana/run.sh --network devnet         # Solana devnet
./deploy/ethereum/run.sh --network local        # Local Hardhat node
./deploy/ethereum/run.sh --network sepolia      # Ethereum Sepolia

# 3. Copy the escrow parameters template and edit accordingly
cp deploy/solana/escrow_params.json deploy/    # For Solana
cp deploy/ethereum/escrow_params.json deploy/  # For Ethereum

# 4. Create an escrow
cargo run --release -p zescrow-client -- create
```

## Prerequisites

### Solana

- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v0.32.1+)
- For devnet: ~3 SOL for deployment (use `solana airdrop`)

### Ethereum

- [Node.js](https://nodejs.org/) (v18+)
- For Sepolia: ETH from a [faucet](https://sepoliafaucet.com/)
- For Sepolia: RPC endpoint (Alchemy, Infura, etc.)

## Directory Structure

```sh
deploy/
├── .env.template             # Environment variables (copy to project root)
├── escrow_conditions.json    # ZK conditions template
├── create_recipient_sol.sh   # Helper: create Solana recipient keypair
├── README.md                 # This file
├── solana/
│   ├── escrow_params.json    # Solana config template
│   └── run.sh                # Deployment script (--network local|devnet)
└── ethereum/
    ├── escrow_params.json    # Ethereum config template
    └── run.sh                # Deployment script (--network local|sepolia)

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

2. Edit `.env` with your values. The same file works for both local and devnet; just change the RPC URLs:

```bash
# For local development
SOLANA_RPC_URL=http://localhost:8899
ETHEREUM_RPC_URL=http://localhost:8545

# For devnet/Sepolia
SOLANA_RPC_URL=https://api.devnet.solana.com
ETHEREUM_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY
```

## Solana Deployment

### Localnet

1. Start the test validator:

```bash
solana config set --url localhost
solana-test-validator -r
```

2. In a new terminal, deploy the program:

```bash
./deploy/solana/run.sh --network local
```

3. Create a recipient keypair and fund it:

```bash
./deploy/create_recipient_sol.sh --network local
```

This creates `deploy/recipient_keypair.json` and outputs the public key. Add to your `.env`:

```bash
SOLANA_RECIPIENT_PUBKEY=<pubkey_from_output>
```

4. Copy and edit the escrow config template:

```bash
cp deploy/solana/escrow_params.json deploy/
```

5. Create and complete the escrow:

```bash
# Create escrow (funds are locked)
cargo run --release -p zescrow-client -- create

# Release to recipient (after finish_after slot)
cargo run --release -p zescrow-client -- finish \
  --recipient deploy/recipient_keypair.json

# Or cancel/refund (after cancel_after slot)
cargo run --release -p zescrow-client -- cancel
```

### Devnet

1. Configure for devnet and fund your deployer wallet:

```bash
solana config set --url https://api.devnet.solana.com
solana airdrop 5 $(solana address)
```

2. Deploy the program:

```bash
./deploy/solana/run.sh --network devnet
```

3. Create a recipient keypair and fund it:

```bash
./deploy/create_recipient_sol.sh --network devnet
```

This creates `deploy/recipient_keypair.json` and airdrops 1 SOL. Add to your `.env`:

```bash
SOLANA_RECIPIENT_PUBKEY=<pubkey_from_output>
```

4. Copy and edit the escrow config template:

```bash
cp deploy/solana/escrow_params.json deploy/
```

5. Create and complete the escrow:

```bash
# Create escrow (funds are locked)
cargo run --release -p zescrow-client -- create

# Release to recipient (after finish_after slot)
cargo run --release -p zescrow-client -- finish \
  --recipient deploy/recipient_keypair.json

# Or cancel/refund (after cancel_after slot)
cargo run --release -p zescrow-client -- cancel
```

## Ethereum Deployment

### Local (Hardhat node)

1. Start Hardhat node:

```bash
cd agent/ethereum
npm install
npx hardhat node
```

Note the pre-funded accounts printed to the console. Pick one for sender and one for recipient.

2. In a new terminal, deploy the contract:

```bash
./deploy/ethereum/run.sh --network local
```

3. Configure sender and recipient in your `.env`:

```bash
# Use accounts from Hardhat node output (without 0x prefix for private keys)
ETHEREUM_SENDER_PRIVATE_KEY=<account_0_private_key>
ETHEREUM_SENDER_ADDRESS=<account_0_address>
ETHEREUM_RECIPIENT_ADDRESS=<account_1_address>
```

Keep the recipient's private key handy for step 6.

4. Copy and edit the escrow config template:

```bash
cp deploy/ethereum/escrow_params.json deploy/
```

5. Create and complete the escrow:

```bash
# Create escrow (funds are locked)
cargo run --release -p zescrow-client -- create

# Release to recipient (after finish_after block)
# Use recipient's private key without 0x prefix
cargo run --release -p zescrow-client -- finish \
  --recipient <RECIPIENT_PRIVATE_KEY>

# Or cancel/refund (after cancel_after block)
cargo run --release -p zescrow-client -- cancel
```

6. (Optional) Mine blocks to advance past `finish_after`:

```bash
cd agent/ethereum
npx hardhat console --network localhost
> await ethers.provider.send("evm_mine", [])
```

### Sepolia (testnet)

1. Set environment variables for deployment:

```bash
export ETHEREUM_SENDER_PRIVATE_KEY="your_private_key"
export ETHERSCAN_API_KEY="your_api_key"  # Optional, for verification
```

2. Deploy the contract:

```bash
./deploy/ethereum/run.sh --network sepolia
```

3. Configure sender and recipient in your `.env`:

```bash
ETHEREUM_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY
ESCROW_CONTRACT_ADDRESS=<from_deploy_output>
ETHEREUM_SENDER_PRIVATE_KEY=<your_private_key>
ETHEREUM_SENDER_ADDRESS=<your_address>
ETHEREUM_RECIPIENT_ADDRESS=<recipient_address>
```

4. Copy and edit the escrow config template:

```bash
cp deploy/ethereum/escrow_params.json deploy/
```

5. Create and complete the escrow:

```bash
# Create escrow (funds are locked)
cargo run --release -p zescrow-client -- create

# Release to recipient (after finish_after block)
# Use recipient's private key without 0x prefix
cargo run --release -p zescrow-client -- finish \
  --recipient <RECIPIENT_PRIVATE_KEY>

# Or cancel/refund (after cancel_after block)
cargo run --release -p zescrow-client -- cancel
```

## Cryptographic Conditions

For escrows with ZK conditions, install the [RISC Zero toolchain](https://dev.risczero.com/api/zkvm/quickstart#1-install-the-risc-zero-toolchain) and use the `--features prover` flag.

### Generate Conditions

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

### Use Conditions

1. Set `"has_conditions": true` in `escrow_params.json`
2. Build and run with the `prover` feature:

```bash
cargo run --release -p zescrow-client --features prover -- create
cargo run --release -p zescrow-client --features prover -- finish --recipient <KEY>
```

## Configuration Reference

### Environment Variables

| Variable                      | Description                     |
| ----------------------------- | ------------------------------- |
| `SOLANA_RPC_URL`              | Solana RPC endpoint             |
| `SOLANA_PROGRAM_ID`           | Deployed program ID             |
| `SOLANA_SENDER_KEYPAIR_PATH`  | Path to sender's keypair file   |
| `SOLANA_SENDER_PUBKEY`        | Sender's public key (base58)    |
| `SOLANA_RECIPIENT_PUBKEY`     | Recipient's public key (base58) |
| `ETHEREUM_RPC_URL`            | Ethereum RPC endpoint           |
| `ESCROW_CONTRACT_ADDRESS`     | Deployed contract address       |
| `ETHEREUM_SENDER_PRIVATE_KEY` | Sender's private key (no 0x)    |
| `ETHEREUM_SENDER_ADDRESS`     | Sender's address (0x...)        |
| `ETHEREUM_RECIPIENT_ADDRESS`  | Recipient's address (0x...)     |

### escrow_params.json Fields

| Field                            | Description                                     |
| -------------------------------- | ----------------------------------------------- |
| `chain_config.chain`             | `"solana"` or `"ethereum"`                      |
| `chain_config.rpc_url`           | Network RPC endpoint (uses env var)             |
| `chain_config.sender_private_id` | Keypair path (Solana) or private key (Ethereum) |
| `chain_config.agent_id`          | Program ID or contract address                  |
| `asset.kind`                     | `"native"` for SOL/ETH                          |
| `asset.amount`                   | Amount in smallest unit (lamports/wei)          |
| `finish_after`                   | Slot/block after which release is allowed       |
| `cancel_after`                   | Slot/block after which cancel is allowed        |
| `has_conditions`                 | `true` if ZK conditions apply                   |

## Running Tests

### Solana (Anchor)

```bash
cd agent/solana/escrow
anchor test                           # Starts its own validator
anchor test --skip-local-validator    # Uses running validator
```

### Ethereum (Hardhat)

```bash
cd agent/ethereum
npx hardhat test                      # Starts its own node
npx hardhat test --network localhost  # Uses running node
```
