# End-to-End Flow for Ethereum Escrows

1. Set up your local environment for Ethereum development:

- Install [Node.js & npm](https://nodejs.org/) (or [Yarn](https://classic.yarnpkg.com/lang/en/docs/install/)). This demo uses NodeJS and NPM.

- Install Hardhat:

```sh
npm install --global hardhat
```

2. Install dependencies in the Ethereum adapter

```sh
# project root
cd adapters/ethereum
npm install
```

3. Start the local Ethereum network. In one terminal, launch Hardhat’s built-in node:

```sh
npx hardhat node
```

4. Compile & deploy the `EscrowFactory`, `Escrow` and `Verifier` contracts. In a second terminal:

```sh
cd adapters/ethereum
# Compile Solidity contracts
npx hardhat compile

# Deploy to localhost
npx hardhat run --network localhost scripts/deploy.ts
```

After deployment completes, note the printed addresses for both the escrow and verifier contracts.

5. Edit the [escrow_params.json](/templates/escrow_params.json) file to specify the parameters of your escrow. When in doubt, please check the definition of `EscrowParams` in the [`core` interface](/core/src/interface.rs), which provides the full context for what's expected. Here's an example of what yours might look like:

```json
{
    "network": "ethereum",
    "chain_config": {
        "rpc_url": "http://localhost:8545",
        "sender_private_key": "YOUR-WIF-ENCODED-PRIVATE-KEY",
        "escrow_factory_address": "YOUR_ESCROW_FACTORY_CONTRACT_ADDRESS",
        "verifier_address": "YOUR_VERIFIER_CONTRACT_ADDRESS"
    },
    "asset_type": "native",
    "asset": {
        "chain": "ethereum",
        "amount": 1
    },
    "sender": {
        "identity": {
            "encoding": "hex",
            "value": "0xSENDER_ADDRESS"
        }
    },
    "recipient": {
        "identity": {
            "encoding": "hex",
            "value": "0xRECIPIENT_ADDRESS"
        }
    },
    "has_conditions": false
}
```

If `has_conditions == true` as specified in your `escrow_params.json`, then ensure the conditions and their fulfillment (i.e., the witness data) are specified in the [escrow_conditions.json](/templates/escrow_conditions.json) file.

6. Create a test Ethereum account for the `recipient`. Hardhat’s node prints a list of pre-funded accounts and their private keys. Copy one of the private keys (WIF) and save it for the escrow `Finish` command (steps 8 and 9).

7. To create an escrow transaction:

```sh
cd client
cargo run --release -- create
```

8. To finish (release) an escrow with `has_conditions == false`:

```sh
# From the `client` directory:
cargo run --release -- finish --recipient <RECIPIENT>
```

9. To finish an escrow with `has_conditions == true`...
First, generate the zero-knowledge proof:

```sh
# project root
cd zescrow
cargo run --release
```

Then rerun the `Finish` command as in step 8.

10. To cancel an escrow:

```sh
cd client
cargo run --release -- cancel
```
