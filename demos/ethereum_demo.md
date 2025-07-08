# Ethereum Demo

## End-to-End Flow for Ethereum Escrows

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
        "chain_config": {
            "chain": "ethereum",
            "rpc_url": "http://localhost:8545",
            "sender_private_id": "SENDER_ETHEREUM_PRIVATE_KEY (WIF)",
            "agent_id": "0xFactoryContractAddress",
        },
        "asset": {
            "kind": "token",
            "id": {
                "hex": "0xTokenContractAddress"
            },
            "agent_id": null,
            "amount": "1000000000000000000", // (1 ETH == 1_000_000_000_000_000_000 wei)
            "decimals": 18,
            "total_supply": null
        },
        "sender": {
            "identity": {
                "hex": "0xSenderEthereumAddress"
            }
        },
        "recipient": {
            "identity": {
                "hex": "0xRecipientEthereumAddress"
            }
        },
        "finish_after": 4, // finish escrow after this block
        "cancel_after": 12, // cancel escrow after this block
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

    There's a `client` command (`generate`) to help with creating the conditions file...

    * To generate a _preimage_ `escrow_conditions.json` file:

    ```sh
    cd client
    RUST_LOG=info cargo run -- generate --preimage "Ethereum Escrow"
    ```

    * To generate an _ed25519_ `escrow_conditions.json` file:

    ```sh
    RUST_LOG=info cargo run -- generate \
    --ed25519-pubkey DEAD... \
    --ed25519-sig BEAF... \
    --ed25519-msg "Ethereum Escrow"
    ```

    * To generate a _threshold_ `escrow_conditions.json` file:

    ```sh
    RUST_LOG=info cargo run -- generate --threshold cond1.json,cond2.json,cond3.json --n 2
    ```

6. Create an escrow transaction:

    ```sh
    # From the `client` directory:
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

8. To finish (release) an escrow:

    ```sh
    # From the `client` directory:
    RUST_LOG=info cargo run -- finish --recipient <RECIPIENT_PRIVATE_KEY>
    ```

    To verify that the `recipient` received the funds, you can query the balance inside the Hardhat console you instantiated earlier:

    ```js
    const balance = await ethers.provider.getBalance("0xRECIPIENT_ADDRESS")
    ethers.formatEther(balance)
    ```

9. To cancel an escrow:

    ```sh
    # From the `client` directory:
    RUST_LOG=info cargo run -- cancel
    ```

## Testing

1. If you have a local Hardhat node already running (e.g. via `npx hardhat node`), skip spinning up a new one:

    ```sh
    cd agent/ethereum
    npx hardhat test --network localhost
    ```

2. If you do **not** have a node running, let Hardhat start its in-process network automatically:

    ```sh
    cd agent/ethereum
    npx hardhat test
    ```

    In both cases, Hardhat will compile, deploy your contracts to the test network, run the full Mocha suite, and then tear down the sandbox node when finished.
