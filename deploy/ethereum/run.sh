#!/usr/bin/env bash
#
# Ethereum Contract Deployment Script
#
# Deploys the Zescrow Escrow contract to Ethereum (local or Sepolia).
#
# Usage:
#   ./deploy/ethereum/run.sh [--network local|sepolia]
#
# Examples:
#   ./deploy/ethereum/run.sh                   # Defaults to local
#   ./deploy/ethereum/run.sh --network local   # Local Hardhat node
#   ./deploy/ethereum/run.sh --network sepolia # Sepolia testnet
#
# Prerequisites:
#   - Node.js and npm installed
#   - For Sepolia: ETHEREUM_SENDER_PRIVATE_KEY environment variable set
#   - For Sepolia: ETHERSCAN_API_KEY environment variable set (optional)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/agent/ethereum"

# Default to local network
NETWORK="local"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--network local|sepolia]"
            exit 1
            ;;
    esac
done

# Set RPC URL based on network
case $NETWORK in
    local|localhost)
        RPC_URL="http://localhost:8545"
        HARDHAT_NETWORK="localhost"
        ;;
    sepolia)
        RPC_URL="${ETHEREUM_RPC_URL:-https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY}"
        HARDHAT_NETWORK="sepolia"
        if [[ -z "${ETHEREUM_SENDER_PRIVATE_KEY:-}" ]]; then
            echo "Error: ETHEREUM_SENDER_PRIVATE_KEY environment variable is required for Sepolia"
            echo ""
            echo "Set it in your .env file or export it:"
            echo "  export ETHEREUM_SENDER_PRIVATE_KEY=\"your_private_key\""
            exit 1
        fi
        ;;
    *)
        echo "Error: Invalid network '$NETWORK'. Use 'local' or 'sepolia'."
        exit 1
        ;;
esac

echo "Deploying Zescrow to Ethereum $NETWORK..."
echo "RPC URL: $RPC_URL"

cd "$CONTRACTS_DIR"

if [[ ! -d "node_modules" ]]; then
    echo "Installing dependencies..."
    npm install
fi

echo ""
echo "Compiling contracts..."
npx hardhat compile

echo ""
echo "Deploying to $NETWORK..."
DEPLOYED_ADDRESS=$(npx hardhat run scripts/deploy.ts --network "$HARDHAT_NETWORK" 2>&1 | grep -oE '0x[a-fA-F0-9]{40}' | tail -1)

if [[ -z "$DEPLOYED_ADDRESS" ]]; then
    echo "Error: Failed to extract deployed contract address"
    exit 1
fi

echo ""
echo "Deployment complete!"
echo "Contract address: $DEPLOYED_ADDRESS"

# Verify on Etherscan for Sepolia
if [[ "$NETWORK" == "sepolia" ]] && [[ -n "${ETHERSCAN_API_KEY:-}" ]]; then
    echo ""
    echo "Verifying contract on Etherscan..."
    npx hardhat verify --network sepolia "$DEPLOYED_ADDRESS" || true
fi

echo ""
echo "Next steps:"
echo "  1. Add to your .env:"
echo "     ETHEREUM_RPC_URL=$RPC_URL"
echo "     ESCROW_CONTRACT_ADDRESS=$DEPLOYED_ADDRESS"
echo "  2. Copy the escrow parameters template:"
echo "     cp deploy/ethereum/escrow_params.json deploy/"
echo "  3. Create an escrow:"
echo "     cargo run --release -p zescrow-client -- create"
