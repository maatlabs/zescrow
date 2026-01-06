#!/usr/bin/env bash
#
# Ethereum Contract Deployment Script
#
# Deploys the Zescrow Escrow contract to Ethereum Sepolia testnet.
#
# Usage:
#   ./deploy/ethereum/run.sh
#
# Prerequisites:
#   - Node.js and npm installed
#   - ETHEREUM_SENDER_PRIVATE_KEY environment variable set
#   - ETHERSCAN_API_KEY environment variable set (optional, for verification)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/agent/ethereum"

echo "Deploying Zescrow to Ethereum Sepolia..."

if [[ -z "${ETHEREUM_SENDER_PRIVATE_KEY:-}" ]]; then
    echo "Error: ETHEREUM_SENDER_PRIVATE_KEY environment variable is required"
    echo ""
    echo "Set it in your .env file or export it:"
    echo "  export ETHEREUM_SENDER_PRIVATE_KEY=\"your_private_key\""
    exit 1
fi

cd "$CONTRACTS_DIR"

if [[ ! -d "node_modules" ]]; then
    echo "Installing dependencies..."
    npm install
fi

echo ""
echo "Compiling contracts..."
npx hardhat compile

echo ""
echo "Deploying to Sepolia..."
DEPLOYED_ADDRESS=$(npx hardhat run scripts/deploy.ts --network sepolia 2>&1 | grep -oE '0x[a-fA-F0-9]{40}' | tail -1)

if [[ -z "$DEPLOYED_ADDRESS" ]]; then
    echo "Error: Failed to extract deployed contract address"
    exit 1
fi

echo ""
echo "Deployment complete!"
echo "Contract address: $DEPLOYED_ADDRESS"

if [[ -n "${ETHERSCAN_API_KEY:-}" ]]; then
    echo ""
    echo "Verifying contract on Etherscan..."
    npx hardhat verify --network sepolia "$DEPLOYED_ADDRESS" || true
fi

echo ""
echo "Next steps:"
echo "  1. Add ESCROW_CONTRACT_ADDRESS=$DEPLOYED_ADDRESS to your .env"
echo "  2. Copy deploy/ethereum/escrow_params.json to the working directory"
echo "  3. Run: cargo run --release -p zescrow-client -- create"
