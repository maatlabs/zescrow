# End-to-End Flow for Solana Escrows

1. Set up your local environment for Solana development by installing the [Solana CLI](https://solana.com/docs/intro/installation).

2. In a terminal instance, configure/ensure you are on the localnet cluster:

```sh
# project root
cd zescrow
solana config set --url localhost && solana config get
```

3. Start the Solana test validator:

```sh
solana-test-validator
```

4. In a separate terminal, build the Solana adapter (i.e., `escrow` + `verifier` programs):

```sh
cd adapters/solana
anchor build
```

5. Generate and retrieve the program IDs via the following commands:

```sh
solana address -k target/deploy/escrow-keypair.json
solana address -k target/deploy/verifier-keypair.json
```

6. Update the program IDs in each of the program's respective `Anchor.toml` and `lib.rs` files:

E.g., in `Anchor.toml` for the `escrow` program:

```toml
[programs.localnet]
escrow = "YOUR-NEW-ESCROW-PROGRAM-ID"
```

and in `lib.rs`:

```rust
declare_id!("YOUR-NEW-ESCROW-PROGRAM-ID");
```

7. Build the programs one more time and then deploy:

```sh
anchor build && anchor deploy
```

8. With the programs deployed, you are ready to interact with them via the `client`. First, create a test Solana account that you can use as the `recipient` (or beneficiary) of the escrow. The following command creates and funds an account with 2 SOL using the [create_sol_account.sh](../templates/create_sol_account.sh) file:

```sh
cd templates
./create_sol_account.sh
```

9. Edit the [escrow_params.json](/templates/escrow_params.json) file to specify the parameters of your escrow. When in doubt, please check the definition of `EscrowParams` in the [`core` interface](/core/src/interface.rs), which provides the full context for what's expected.

If `has_conditions == true` as specified in your `escrow_params.json`, then ensure the conditions and their fulfillment (i.e., the witness data) are specified in the [escrow_conditions.json](/templates/escrow_conditions.json) file.

10. Create an escrow transaction:

```sh
cd client
cargo run --release -- create
```

11. To release an escrow with `has_conditions == false` execute:

```sh
cargo run --release -- finish --recipient-keypair-path <PATH>
```

For example, using the `test_keypair.json` created earlier, the above command will be:

```sh
cargo run --release -- finish --recipient-keypair-path ../templates/test_keypair.json
```

12. To release an escrow with `has_conditions == true`, first run the `prover` to generate a valid receipt for the execution of the cryptographic conditions:

```sh
# project root
cd zescrow
cargo run --release
```

Then execute the `Finish` command of the `client`, just like in step 11.

13. To cancel an escrow, execute:

```sh
cd client
cargo run --release -- cancel
```
