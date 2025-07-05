# End-to-End Flow for Ethereum Escrows

1. Set up your local environment for Ethereum development:

* Install [Node.js & npm](https://nodejs.org/) (or [Yarn](https://classic.yarnpkg.com/lang/en/docs/install/)). This demo uses NodeJS and NPM.

* Install Hardhat:

```sh
npm install --global hardhat
```

2. Install dependencies in the Ethereum agent

```sh
cd agent/ethereum
npm install
```

3. Start the local Ethereum network. In one terminal, launch Hardhat’s built-in node:

```sh
# in `agent/ethereum`
npx hardhat node
```

Hardhat will print a list of pre-funded accounts and their WIF private keys. You may use any of these public/private keypairs for the `sender` and `recipient` of your escrow transactions in this demo. Just make sure to remember which ones you used for what purpose. **Note**, however, that unless you specify a different `from` address in your transactions, Hardhat uses the first account (`Account # 0`) by default. Thus, you can use any of the remaining accounts as the `to` (recipient) address.

4. Compile & deploy the `EscrowFactory` and `Verifier` contracts. In a second terminal:

```sh
cd agent/ethereum
npx hardhat compile
npx hardhat run --network localhost scripts/deploy.ts
```

After deployment completes, note the printed `EscrowFactory` and `Verifier` addresses.

5. Edit the [escrow_params.json](/templates/escrow_params.json) file to specify the parameters of your escrow. When in doubt, please check the definition of `EscrowParams` in the [`core` interface](/core/src/interface.rs), which provides the full context for what's expected. Here's an example of what your escrow parameters might look like:

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
        "amount": "100000000000000000000" // 100 ETH
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
    "finish_after": 4, // finishAfter block
    "cancel_after": 12, // cancelAfter block
    "has_conditions": false
}
```

If `has_conditions == true` as specified in your `escrow_params.json`, then ensure the conditions and their fulfillment (i.e., the witness data) are specified in the [escrow_conditions.json](/templates/escrow_conditions.json) file. Here's an example of how your conditions file might look like:

```json
{
  "condition": "preimage",
  "fulfillment": {
    "hash": "<hex-encoded SHA-256 digest of the preimage>",
    "preimage": "<the actual preimage value, as a UTF-8 string>"
  }
}
```

6. To create an escrow transaction:

```sh
cd client
RUST_LOG=info cargo run -- create
```

7. After sending the escrow transaction, the logs from the Hardhat node will most likely show that your transaction was mined in `Block #2`. Before you can finish/release an escrow, `finish_after` must be `>=` current block. For example, if you specified `"finish_after": 4`, then you need to use the JSON-RPC method `evm_mine` to force Hardhat to mine some empty blocks. Each call bumps the block number by one:

```sh
# Connect to the node
cd agent/ethereum
npx hardhat console --network localhost

# While in the console, mine as many blocks as you need
# to reach `finish_after`
await ethers.provider.send("evm_mine", []);
```

8. To finish (release) an escrow with `has_conditions == false`:

```sh
# From the `client` directory:
RUST_LOG=info cargo run -- finish --recipient <RECIPIENT_PRIVATE_KEY>
```

To verify that the `recipient` received the funds, you can query the balance inside the Hardhat console you instantiated earlier:

```js
const balance = await ethers.provider.getBalance("0xRECIPIENT_ADDRESS")
ethers.formatEther(balance)
```

9. To finish an escrow with `has_conditions == true`...
First, generate the zero-knowledge proof:

```sh
# project root
cd zescrow
RUST_LOG=info cargo run --release
```

Then rerun the `Finish` command just like in the previous step.

10. To cancel an escrow:

```sh
cd client
RUST_LOG=info cargo run -- cancel
```
