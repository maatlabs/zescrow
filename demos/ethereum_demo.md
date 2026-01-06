# Ethereum Demo

## End-to-End Flow for Ethereum Escrows (Local Development)

This guide walks through creating and completing an escrow on a **local Hardhat node**. For Sepolia testnet deployment, see the [Deployment Guide](/deploy/README.md).

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- npm or yarn

### 1. Install Dependencies

```sh
cd agent/ethereum
npm install
```

### 2. Start the Local Network

In one terminal, launch Hardhat's built-in node:

```sh
cd agent/ethereum
npx hardhat node
```

Hardhat prints a list of pre-funded accounts with their private keys. Use any of these for testing. By default, Hardhat uses Account #0 for transactions.

### 3. Deploy the Contract

In a second terminal:

```sh
cd agent/ethereum
npx hardhat compile
npx hardhat run --network localhost scripts/deploy.ts
```

Note the printed `EscrowFactory` contract address.

### 4. Configure Escrow Parameters

Create an `escrow_params.json` file in the `deploy/` directory. You can start by copying the Sepolia template and modifying it for localhost:

```sh
cp deploy/ethereum/escrow_params.json deploy/escrow_params.json
```

Then edit `deploy/escrow_params.json`:

```json
{
    "chain_config": {
        "chain": "ethereum",
        "rpc_url": "http://localhost:8545",
        "sender_private_id": "ACCOUNT_0_PRIVATE_KEY",
        "agent_id": "0xCONTRACT_ADDRESS_FROM_STEP_3"
    },
    "asset": {
        "kind": "native",
        "id": null,
        "agent_id": null,
        "amount": "1000000000000000000",
        "decimals": null,
        "total_supply": null
    },
    "sender": {
        "identity": {
            "hex": "0xSENDER_ADDRESS"
        }
    },
    "recipient": {
        "identity": {
            "hex": "0xRECIPIENT_ADDRESS"
        }
    },
    "finish_after": 4,
    "cancel_after": 12,
    "has_conditions": false
}
```

Notes:

- `amount`: 1 ETH = 1,000,000,000,000,000,000 wei
- `finish_after`/`cancel_after`: block numbers
- Use different Hardhat accounts for sender and recipient

### 5. (Optional) Configure Cryptographic Conditions

If `has_conditions` is `true`, create a conditions file using the `generate` command:

```sh
# Hashlock condition
cargo run -p zescrow-client -- generate hashlock --preimage ./secret.txt

# Ed25519 signature condition
cargo run -p zescrow-client -- generate ed25519 \
  --pubkey <PUBKEY_HEX> --msg <MSG_HEX> --sig <SIG_HEX>

# Threshold condition (M-of-N)
cargo run -p zescrow-client -- generate threshold \
  --subconditions cond1.json cond2.json cond3.json \
  --threshold 2
```

The output is written to `deploy/escrow_conditions.json` by default.

### 6. Create an Escrow

```sh
RUST_LOG=info cargo run -p zescrow-client -- create
```

### 7. Mine Blocks (if needed)

The escrow transaction is typically mined in Block #2. Before releasing, the current block must be >= `finish_after`. Use the Hardhat console to mine blocks:

```sh
cd agent/ethereum
npx hardhat console --network localhost
```

In the console:

```js
// Mine blocks until finish_after is reached
await ethers.provider.send("evm_mine", []);
```

### 8. Complete the Escrow

**To release funds to recipient:**

```sh
RUST_LOG=info cargo run -p zescrow-client -- finish \
  --recipient <RECIPIENT_PRIVATE_KEY>
```

Verify the recipient received funds in the Hardhat console:

```js
const balance = await ethers.provider.getBalance("0xRECIPIENT_ADDRESS")
ethers.formatEther(balance)
```

**To cancel and refund:**

```sh
RUST_LOG=info cargo run -p zescrow-client -- cancel
```

## Testing the Solidity Contract

With a local Hardhat node running:

```sh
cd agent/ethereum
npx hardhat test --network localhost
```

Without a running node (Hardhat starts one automatically):

```sh
cd agent/ethereum
npx hardhat test
```
