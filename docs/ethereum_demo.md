# End-to-End Flow for Ethereum Escrows

1. Set up your local environment for Ethereum development:

* Install [Node.js & npm](https://nodejs.org/) (or [Yarn](https://classic.yarnpkg.com/lang/en/docs/install/)). This demo uses NodeJS and NPM.

* Install Hardhat:

```sh
npm install --global hardhat
```

2. Install dependencies in the Ethereum adapter

```sh
cd adapters/ethereum
npm install
```

3. Start the local Ethereum network. In one terminal, launch Hardhat’s built-in node:

```sh
# in `adapters/ethereum`
npx hardhat node
```

Hardhat will print a list of pre-funded accounts and their WIF private keys. You may use any of these public/private keypairs for the `sender` and `recipient` of your escrow transactions in this demo. Just make sure to remember which ones you used for what purpose. **Note**, however, that unless you specify a different `from` address in your transactions, Hardhat uses the first account (`Account # 0`) by default. Thus, you can use any of the remaining accounts as the `to` (recipient) address.

4. Compile & deploy the `EscrowFactory` and `Verifier` contracts. In a second terminal:

```sh
cd adapters/ethereum
npx hardhat compile
npx hardhat run --network localhost scripts/deploy.ts
```

After deployment completes, note the printed `EscrowFactory` and `Verifier` addresses.

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
    "finish_after": 4,
    "cancel_after": 12,
    "has_conditions": false
}
```

If `has_conditions == true` as specified in your `escrow_params.json`, then ensure the conditions and their fulfillment (i.e., the witness data) are specified in the [escrow_conditions.json](/templates/escrow_conditions.json) file.

6. To create an escrow transaction:

```sh
cd client
cargo run --release -- create
```

7. To finish (release) an escrow with `has_conditions == false`:

```sh
# From the `client` directory:
cargo run --release -- finish --recipient <RECIPIENT_PRIVATE_KEY>
```

8. To finish an escrow with `has_conditions == true`...
First, generate the zero-knowledge proof:

```sh
# project root
cd zescrow
cargo run --release
```

Then rerun the `Finish` command as in step 7.

9. To cancel an escrow:

```sh
cd client
cargo run --release -- cancel
```
