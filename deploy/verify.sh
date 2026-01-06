#!/usr/bin/env bash
#
# Deployment Verification Script
#
# Verifies that deployed contracts are functioning correctly.
#
# Usage:
#   ./deploy/verify.sh solana <program_id>
#   ./deploy/verify.sh ethereum <contract_address>

set -euo pipefail

CHAIN="${1:-}"
ADDRESS="${2:-}"

usage() {
    echo "Usage:"
    echo "  $0 solana <program_id>"
    echo "  $0 ethereum <contract_address>"
    exit 1
}

if [[ -z "$CHAIN" || -z "$ADDRESS" ]]; then
    usage
fi

verify_solana() {
    local program_id="$1"
    local rpc_url="https://api.devnet.solana.com"

    echo "Verifying Solana program on devnet..."
    echo "Program ID: $program_id"
    echo ""

    if solana program show "$program_id" --url "$rpc_url" 2>/dev/null; then
        echo ""
        echo "Program verified successfully!"
    else
        echo "Error: Program not found or not executable"
        exit 1
    fi
}

verify_ethereum() {
    local contract_address="$1"
    local rpc_url="${ETHEREUM_SEPOLIA_RPC_URL:-https://eth-sepolia.public.blastapi.io}"

    echo "Verifying Ethereum contract on Sepolia..."
    echo "Contract: $contract_address"
    echo "Explorer: https://sepolia.etherscan.io/address/$contract_address"
    echo ""

    # Check if contract has code
    CODE=$(curl -s -X POST "$rpc_url" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getCode\",\"params\":[\"$contract_address\",\"latest\"],\"id\":1}" \
        | grep -oE '"result":"0x[a-fA-F0-9]+"' | cut -d'"' -f4)

    if [[ "$CODE" == "0x" || -z "$CODE" ]]; then
        echo "Error: No contract code found at address"
        exit 1
    fi

    echo "Contract code found (${#CODE} chars)"

    # Query escrowCount to verify contract is functional
    CALL_DATA="0xd3a5bcda"  # escrowCount() selector
    RESULT=$(curl -s -X POST "$rpc_url" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_call\",\"params\":[{\"to\":\"$contract_address\",\"data\":\"$CALL_DATA\"},\"latest\"],\"id\":1}" \
        | grep -oE '"result":"0x[a-fA-F0-9]+"' | cut -d'"' -f4)

    if [[ -n "$RESULT" ]]; then
        ESCROW_COUNT=$((16#${RESULT:2}))
        echo "Escrow count: $ESCROW_COUNT"
        echo ""
        echo "Contract verified successfully!"
    else
        echo "Warning: Could not query escrowCount()"
    fi
}

case "$CHAIN" in
    solana)
        verify_solana "$ADDRESS"
        ;;
    ethereum)
        verify_ethereum "$ADDRESS"
        ;;
    *)
        echo "Error: Unknown chain '$CHAIN'"
        usage
        ;;
esac
