#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYPAIR="$SCRIPT_DIR/test_keypair.json"

solana-keygen new --outfile "$KEYPAIR" --force --no-bip39-passphrase

PUBKEY=$(solana address --keypair "$KEYPAIR")

solana airdrop 2 "$PUBKEY" --url http://localhost:8899

echo "Test account ready:"
echo "Keypair: $KEYPAIR"
echo "Pubkey:  $PUBKEY"
