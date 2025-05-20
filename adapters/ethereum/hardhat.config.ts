require("@nomicfoundation/hardhat-toolbox");
require("@typechain/hardhat");

module.exports = {
    solidity: "0.8.28",
    typechain: {
        outDir: "typechain-types",
        target: "ethers-v6",
    },
};