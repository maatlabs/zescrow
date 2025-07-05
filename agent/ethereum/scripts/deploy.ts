import { ethers } from "hardhat";

async function main() {
    const Factory = await ethers.getContractFactory("EscrowFactory");
    const factory = await Factory.deploy();
    await factory.waitForDeployment();
    const factory_addr = await factory.getAddress();
    console.log("EscrowFactory deployed to:", factory_addr);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });