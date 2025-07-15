import "@nomicfoundation/hardhat-toolbox";
import type { HardhatUserConfig } from "hardhat/config";

const config: HardhatUserConfig = {
    solidity: "0.8.28",
    typechain: {
        outDir: "typechain-types",
        target: "ethers-v6",
    },
    networks: {
        hardhat: {
            loggingEnabled: true,
            allowUnlimitedContractSize: true,
            chainId: 31337
        }
    }
};

export default config;