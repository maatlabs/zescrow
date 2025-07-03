import { ethers } from "hardhat";

async function main() {
    // 1. Deploy `EscrowFactory`
    const Factory = await ethers.getContractFactory("EscrowFactory");
    const factory = await Factory.deploy();
    await factory.waitForDeployment();
    const factory_addr = await factory.getAddress();
    console.log("EscrowFactory deployed to:", factory_addr);

    // 2. TODO: Uncomment once `Verifier.sol` is implemented
    // const Verifier = await ethers.getContractFactory("Verifier");
    // const verifier = await Verifier.deploy();
    // await verifier.waitForDeployment();
    // const verifier_addr = await verifier.getAddress();
    // console.log("Verifier deployed to:", verifier_addr);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });