# Solana Demo

## End-to-End Flow for Solana Escrows

1. Set up your local environment for Solana development by installing the [Solana CLI](https://solana.com/docs/intro/installation).

2. In a terminal instance, run the Solana test validator:

    ```sh
    # project root
    cd zescrow

    solana config set --url localhost
    solana-test-validator -r
    ```

3. In a separate terminal, build and deploy the escrow program:

    ```sh
    # Go to the Solana escrow program to deploy it locally
    cd agent/solana/escrow

    # Sync keys for local deploy
    anchor keys sync

    # Build the program
    anchor build

    # Deploy the program locally
    anchor deploy
    ```

4. With the programs deployed, you are ready to interact with them via the `client`. First, create a test Solana account that you can use as the `recipient` (or beneficiary) of the escrow. The following command creates and funds an account with 2 SOL using the [create_sol_account.sh](/templates/create_sol_account.sh) file:

    ```sh
    cd templates
    ./create_sol_account.sh
    ```

5. Edit the [escrow_params.json](/templates/escrow_params.json) file to specify the parameters of your escrow. When in doubt, please check the definition of `EscrowParams` in the [`core` interface](/core/src/interface.rs), which provides the full context for what's expected.

    An example of how your `escrow_params.json` might look like:

    ```json
    {
        "chain_config": {
            "chain": "solana",
            "rpc_url": "http://localhost:8899",
            "sender_private_id": "absolute/path/to/user/.config/solana/id.json",
            "agent_id": "EscrowProgramID"
        },
        "asset": {
            "kind": "native",
            "id": null,
            "agent_id": null,
            "amount": "1000000000", // (1 SOL == 1_000_000_000 lamports)
            "decimals": null,
            "total_supply": null
        },
        "sender": {
            "identity": {
                "base58": "SenderSolanaPublicKey"
            }
        },
        "recipient": {
            "identity": {
                "base58": "RecipientSolanaPublicKey"
            }
        },
        "finish_after": 1000, // release escrow after this slot
        "cancel_after": 1200, // cancel escrow after this slot
        "has_conditions": false
    }
    ```

    If `has_conditions == true` as specified in your `escrow_params.json`, then ensure the conditions and their fulfillment (i.e., the witness data) are specified in the [escrow_conditions.json](/templates/escrow_conditions.json) file. Here's an example of how your conditions file might look like:

    ```json
    {
        "condition": "hashlock",
        "fulfillment": {
            "hash": "<hex-encoded SHA-256 digest of the preimage>",
            "preimage": "<the actual preimage value, as a UTF-8 string>"
        }
    }
    ```

    There's a `client` command (`generate`) to help with creating the conditions file... The default `--output` path is always `./templates/escrow_conditions.json`.

    ```sh
    cd client

    # Generate a hashlock condition JSON

    cargo run -- generate hashlock --preimage <PATH> [--output <PATH>]

    # Generate an Ed25519 signature condition JSON

    cargo run -- generate ed25519 --pubkey <PUBKEY> --msg <MSG> --sig <SIG> [--output <PATH>]

    # Generate a Secp256k1 signature condition JSON

    cargo run -- generate secp256k1 --pubkey <PUBKEY> --msg <MSG> --sig <SIG> [--output <PATH>]

    # Generate a threshold condition JSON

    cargo run -- generate threshold --subconditions file1.json file2.json file3.json --threshold <N> [--output <PATH>]
    ```

    As an example, if you run the `hashlock` subcommand with the `rustfmt.toml` file (at project root) as the `preimage` file:

    ```sh
    cargo run -- generate hashlock --preimage ../rustfmt.toml
    ```

    The output of the above command will be a `/templates/escrow_conditions.json` file with the following content:

    ```json
    {
        "condition": "hashlock",
        "fulfillment": {
            "hash": "485b6baec8c0d6304648c3b924399835fdd09c8df9be1b65d169b07ded2237f9",
            "preimage": "# Run with `cargo +nightly fmt`\n\nimports_granularity = \"Module\"\ngroup_imports = \"StdExternalCrate\"\n"
        }
    }
    ```

6. Create an escrow transaction:

    ```sh
    RUST_LOG=info cargo run -- create
    ```

7. To release an escrow, execute:

    ```sh
    RUST_LOG=info cargo run -- finish --recipient <KEYPAIR_FILE_PATH>
    ```

    For example, using the `test_keypair.json` created earlier, the above command will be:

    ```sh
    RUST_LOG=info cargo run -- finish --recipient ../templates/test_keypair.json
    ```

    To verify that the `recipient` received the funds, you can query the balance on the corresponding address/pubkey:

    ```sh
    # In any terminal instance
    solana balance <RECIPIENT_PUBKEY>
    ```

8. To cancel an escrow, execute:

    ```sh
    RUST_LOG=info cargo run -- cancel
    ```

## Testing

To test the escrow program (while the test validator is running in the background):

```sh
cd agent/solana/escrow
anchor test --skip-local-validator
```

If the Solana test validator is **not** running, you can simply do:

```sh
cd agent/solana/escrow
anchor test
```
