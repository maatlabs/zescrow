#!/usr/bin/env bash
#
# Solana Recipient Account Setup Script
#
# Creates a Solana keypair for the escrow recipient and funds it via airdrop.
#
# Usage:
#   ./deploy/create_recipient_sol.sh [--network local|devnet]
#
# Examples:
#   ./deploy/create_recipient_sol.sh                        # Defaults to localnet
#   ./deploy/create_recipient_sol.sh --network local        # Local test validator
#   ./deploy/create_recipient_sol.sh --network devnet       # Devnet
#
# Prerequisites:
#   - Solana CLI installed
#   - For local: test validator running (solana-test-validator)
#   - For devnet: network connectivity

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYPAIR="$SCRIPT_DIR/recipient_keypair.json"

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
            echo "Usage: $0 [--network local|devnet]"
            exit 1
            ;;
    esac
done

# Set RPC URL and airdrop amount based on network
case $NETWORK in
    local|localhost)
        RPC_URL="http://localhost:8899"
        AIRDROP_AMOUNT=2
        ;;
    devnet)
        RPC_URL="https://api.devnet.solana.com"
        AIRDROP_AMOUNT=1
        ;;
    *)
        echo "Error: Invalid network '$NETWORK'. Use 'local' or 'devnet'."
        exit 1
        ;;
esac

echo "Creating recipient Solana account for $NETWORK..."
echo "RPC URL: $RPC_URL"
echo ""

# Generate keypair (overwrites if exists)
solana-keygen new --outfile "$KEYPAIR" --force --no-bip39-passphrase

PUBKEY=$(solana address --keypair "$KEYPAIR")

echo ""
echo "Requesting airdrop of $AIRDROP_AMOUNT SOL..."
solana airdrop "$AIRDROP_AMOUNT" "$PUBKEY" --url "$RPC_URL"

echo ""
echo "Recipient account created:"
echo "  Keypair: $KEYPAIR"
echo "  Pubkey:  $PUBKEY"
echo ""
echo "Add to your .env:"
echo "  SOLANA_RECIPIENT_PUBKEY=$PUBKEY"
echo ""
echo "Use this keypair to release the escrow:"
echo "  cargo run --release -p zescrow-client -- finish --recipient $KEYPAIR"
