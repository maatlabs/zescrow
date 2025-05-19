import type { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "@typechain/hardhat";

const config: HardhatUserConfig = {
    solidity: {
        version: "0.8.28"
    },
    typechain: {
        outDir: "typechain-types",
        target: "ethers-v5",
        alwaysGenerateOverloads: false,
        externalArtifacts: ["externalArtifacts/*.json"],
        dontOverrideCompile: false,
    }
};

export default config;