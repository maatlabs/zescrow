#!/usr/bin/env bash
#
# Solana Program Deployment Script
#
# Deploys the Zescrow escrow program to Solana devnet.
#
# Usage:
#   ./deploy/solana/run.sh
#
# Prerequisites:
#   - Solana CLI installed and configured
#   - Anchor CLI installed (v0.32.1+)
#   - Sufficient SOL balance for deployment (~3 SOL)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PROGRAM_DIR="$PROJECT_ROOT/agent/solana/escrow"

RPC_URL="https://api.devnet.solana.com"

echo "Deploying Zescrow to Solana devnet..."
echo "RPC URL: $RPC_URL"

ANCHOR_VERSION=$(anchor --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
echo "Anchor CLI: v$ANCHOR_VERSION"

WALLET=$(solana address)
BALANCE=$(solana balance --url "$RPC_URL" 2>/dev/null || echo "0 SOL")
echo "Deployer: $WALLET"
echo "Balance: $BALANCE"

cd "$PROGRAM_DIR"

echo ""
echo "Building program..."
anchor build

echo ""
echo "Deploying to devnet..."
anchor deploy --provider.cluster devnet

PROGRAM_ID=$(solana-keygen pubkey target/deploy/escrow-keypair.json)

echo ""
echo "Deployment complete!"
echo "Program ID: $PROGRAM_ID"
echo ""
echo "Next steps:"
echo "  1. Copy deploy/solana/escrow_params.json to the working directory"
echo "  2. Set environment variables in .env"
echo "  3. Run: cargo run --release -p zescrow-client -- create"
