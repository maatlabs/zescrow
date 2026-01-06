# Solana Demo

## End-to-End Flow for Solana Escrows (Local Development)

This guide walks through creating and completing an escrow on a **local Solana test validator**. For devnet deployment, see the [Deployment Guide](/deploy/README.md).

### Prerequisites

- [Solana CLI](https://solana.com/docs/intro/installation) installed
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v0.32.1+)

### 1. Start the Test Validator

In a terminal, run the Solana test validator:

```sh
solana config set --url localhost
solana-test-validator -r
```

### 2. Deploy the Escrow Program

In a separate terminal, build and deploy the escrow program:

```sh
cd agent/solana/escrow

# Sync keys for local deploy
anchor keys sync

# Build and deploy
anchor build
anchor deploy
```

Note the printed Program ID.

### 3. Create a Test Recipient Account

Use the helper script to create and fund a test account:

```sh
./demos/create_sol_account.sh
```

This creates `demos/test_keypair.json` and funds it with 2 SOL.

### 4. Configure Escrow Parameters

Create an `escrow_params.json` file in the `deploy/` directory. You can start by copying the devnet template and modifying it for localhost:

```sh
cp deploy/solana/escrow_params.json deploy/escrow_params.json
```

Then edit `deploy/escrow_params.json`:

```json
{
    "chain_config": {
        "chain": "solana",
        "rpc_url": "http://localhost:8899",
        "sender_private_id": "/absolute/path/to/.config/solana/id.json",
        "agent_id": "YOUR_PROGRAM_ID_FROM_STEP_2"
    },
    "asset": {
        "kind": "native",
        "id": null,
        "agent_id": null,
        "amount": "1000000000",
        "decimals": null,
        "total_supply": null
    },
    "sender": {
        "identity": {
            "base58": "YOUR_SENDER_PUBKEY"
        }
    },
    "recipient": {
        "identity": {
            "base58": "RECIPIENT_PUBKEY_FROM_STEP_3"
        }
    },
    "finish_after": 1000,
    "cancel_after": 1200,
    "has_conditions": false
}
```

Notes:

- `amount`: 1 SOL = 1,000,000,000 lamports
- `finish_after`/`cancel_after`: slot numbers
- Get your sender pubkey with `solana address`

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

### 7. Complete the Escrow

**To release funds to recipient:**

```sh
RUST_LOG=info cargo run -p zescrow-client -- finish \
  --recipient ./demos/test_keypair.json
```

Verify the recipient received funds:

```sh
solana balance <RECIPIENT_PUBKEY>
```

**To cancel and refund:**

```sh
RUST_LOG=info cargo run -p zescrow-client -- cancel
```

## Testing the Anchor Program

With the test validator running:

```sh
cd agent/solana/escrow
anchor test --skip-local-validator
```

Without a running validator (Anchor starts one automatically):

```sh
cd agent/solana/escrow
anchor test
```
