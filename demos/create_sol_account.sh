#!/usr/bin/env bash
#
# Creates a test Solana account for local development
#
# Usage:
#   ./demos/create_sol_account.sh
#
# Prerequisites:
#   - Solana CLI installed
#   - Local test validator running (solana-test-validator)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYPAIR="$SCRIPT_DIR/test_keypair.json"

echo "Creating test Solana account..."

solana-keygen new --outfile "$KEYPAIR" --force --no-bip39-passphrase

PUBKEY=$(solana address --keypair "$KEYPAIR")

solana airdrop 2 "$PUBKEY" --url http://localhost:8899

echo ""
echo "Test account created:"
echo "  Keypair: $KEYPAIR"
echo "  Pubkey:  $PUBKEY"
echo ""
echo "Use this as the recipient in your escrow:"
echo "  cargo run -- finish --recipient $KEYPAIR"
