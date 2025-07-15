import { ethers } from "hardhat";

async function main() {
    const Escrow = await ethers.getContractFactory("Escrow");
    const escrow = await Escrow.deploy();
    await escrow.waitForDeployment();
    const escrow_addr = await escrow.getAddress();
    console.log("Escrow deployed to:", escrow_addr);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });