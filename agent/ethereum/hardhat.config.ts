import "@nomicfoundation/hardhat-toolbox";
import type { HardhatUserConfig } from "hardhat/config";

const PRIVATE_KEY = process.env.ETHEREUM_SENDER_PRIVATE_KEY || "";
const ETHERSCAN_API_KEY = process.env.ETHERSCAN_API_KEY || "";

const config: HardhatUserConfig = {
    solidity: {
        version: "0.8.28",
        settings: {
            optimizer: {
                enabled: true,
                runs: 200,
            },
        },
    },
    typechain: {
        outDir: "typechain-types",
        target: "ethers-v6",
    },
    networks: {
        hardhat: {
            loggingEnabled: true,
            allowUnlimitedContractSize: true,
            chainId: 31337,
        },
        sepolia: {
            url: process.env.ETHEREUM_SEPOLIA_RPC_URL || "https://eth-sepolia.public.blastapi.io",
            accounts: PRIVATE_KEY ? [PRIVATE_KEY] : [],
            chainId: 11155111,
        },
    },
    etherscan: {
        apiKey: ETHERSCAN_API_KEY,
    },
};

export default config;