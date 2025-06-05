// import { expect } from "chai";
// import { ethers, network } from "hardhat";
// import type { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";
// import type { Signer } from "ethers";

// import {
//     type EscrowFactory,
//     EscrowFactory__factory,
//     Escrow__factory,
// } from "../typechain-types";

// describe("Escrow", () => {
//     let deployer: SignerWithAddress;
//     let recipient: SignerWithAddress;
//     let factory: EscrowFactory;

//     beforeEach(async () => {
//         [deployer, recipient] = await ethers.getSigners();
//         factory = await new EscrowFactory__factory(deployer as unknown as Signer).deploy();
//         await factory.waitForDeployment();
//     });

//     it("fails to finish before finishAfter, then succeeds", async () => {
//         const startBlock = await ethers.provider.getBlockNumber();
//         const finishAfter = startBlock + 3;
//         const cancelAfter = startBlock + 4;
//         const value = ethers.parseEther("1");

//         const tx = await factory.createEscrow(
//             recipient.address,
//             finishAfter,
//             cancelAfter,
//             false,
//             ethers.ZeroAddress,
//             { value }
//         );
//         const receipt = await tx.wait();
//         if (!receipt) throw new Error("Transaction failed to be mined");

//         const filter = factory.filters.EscrowCreated(
//             deployer.address,
//             undefined,
//             recipient.address,
//             undefined,
//             undefined,
//             undefined,
//             undefined
//         );
//         const events = await factory.queryFilter(filter, receipt.blockNumber);
//         if (events.length === 0) throw new Error("EscrowCreated event not found");
//         const escrowAddr = events[0].args.escrowAddress;

//         // Attempt to finish via factory (msg.sender = factory); should revert as too early
//         await expect(factory.finishEscrow(escrowAddr, "0x")).to.be.revertedWith("Zescrow: too early to finish");

//         // Mine two blocks to reach finishAfter
//         await network.provider.send("evm_mine");
//         await network.provider.send("evm_mine");

//         const balBefore = await ethers.provider.getBalance(recipient.address);
//         // Finish via factory (no sender check)
//         await (await factory.finishEscrow(escrowAddr, "0x")).wait();
//         const balAfter = await ethers.provider.getBalance(recipient.address);
//         expect(balAfter).to.be.gt(balBefore);
//     });

//     it("fails to cancel before cancelAfter, then succeeds", async () => {
//         const startBlock = await ethers.provider.getBlockNumber();
//         const finishAfter = startBlock + 3;
//         const cancelAfter = startBlock + 4;
//         const value = ethers.parseEther("1");

//         const tx = await factory.createEscrow(
//             recipient.address,
//             finishAfter,
//             cancelAfter,
//             false,
//             ethers.ZeroAddress,
//             { value }
//         );
//         const receipt = await tx.wait();
//         if (!receipt) throw new Error("Transaction failed to be mined");

//         const filter = factory.filters.EscrowCreated(
//             deployer.address,
//             undefined,
//             recipient.address,
//             undefined,
//             undefined,
//             undefined,
//             undefined
//         );
//         const events = await factory.queryFilter(filter, receipt.blockNumber);
//         if (events.length === 0) throw new Error("EscrowCreated event not found");
//         const escrowAddr = events[0].args.escrowAddress;

//         const escrow = Escrow__factory.connect(escrowAddr, deployer as unknown as Signer);

//         // Attempt to cancel earlier (msg.sender = factory); should revert with "too early ..."
//         await expect(factory.cancelEscrow(escrowAddr)).to.be.revertedWith("Zescrow: too early to cancel");

//         // Mine three blocks to reach cancelAfter
//         await network.provider.send("evm_mine");
//         await network.provider.send("evm_mine");
//         await network.provider.send("evm_mine");

//         const balBefore = await ethers.provider.getBalance(deployer.address);
//         // Cancel after cancelAfter (msg.sender = factory)
//         await (await factory.cancelEscrow(escrowAddr)).wait();
//         const balAfter = await ethers.provider.getBalance(deployer.address);
//         expect(balAfter).to.be.gt(balBefore);
//     });
// });

import { expect } from "chai";
import { ethers, network } from "hardhat";
import type { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";
import type { Signer } from "ethers";

import {
    type EscrowFactory,
    EscrowFactory__factory,
} from "../typechain-types";

describe("Escrow", () => {
    let deployer: SignerWithAddress;
    let recipient: SignerWithAddress;
    let factory: EscrowFactory;

    beforeEach(async () => {
        [deployer, recipient] = await ethers.getSigners();
        factory = await new EscrowFactory__factory(deployer as unknown as Signer).deploy();
        await factory.waitForDeployment();
    });

    it("fails to finish before finishAfter, then succeeds", async () => {
        const startBlock = await ethers.provider.getBlockNumber();
        const finishAfter = startBlock + 3;
        const cancelAfter = startBlock + 4;
        const value = ethers.parseEther("1");

        const tx = await factory.createEscrow(
            recipient.address,
            finishAfter,
            cancelAfter,
            false,
            ethers.ZeroAddress,
            { value }
        );
        const receipt = await tx.wait();
        if (!receipt) throw new Error("Transaction failed to be mined");

        const filter = factory.filters.EscrowCreated(
            deployer.address,
            undefined,
            recipient.address,
            undefined,
            undefined,
            undefined,
            undefined
        );
        const events = await factory.queryFilter(filter, receipt.blockNumber);
        if (events.length === 0) throw new Error("EscrowCreated event not found");
        const escrowAddr = events[0].args.escrowAddress;

        // Attempt to finish via factory (msg.sender = factory); should revert as too early
        await expect(factory.finishEscrow(escrowAddr, "0x")).to.be.revertedWith("Zescrow: too early to finish");

        // Mine two blocks to reach finishAfter
        await network.provider.send("evm_mine");
        await network.provider.send("evm_mine");

        const balBefore = await ethers.provider.getBalance(recipient.address);
        // Finish via factory (no sender restriction)
        await (await factory.finishEscrow(escrowAddr, "0x")).wait();
        const balAfter = await ethers.provider.getBalance(recipient.address);
        expect(balAfter).to.be.gt(balBefore);
    });

    it("fails to cancel before cancelAfter, then succeeds", async () => {
        const startBlock = await ethers.provider.getBlockNumber();
        const finishAfter = startBlock + 3;
        const cancelAfter = startBlock + 4;
        const value = ethers.parseEther("1");

        const tx = await factory.createEscrow(
            recipient.address,
            finishAfter,
            cancelAfter,
            false,
            ethers.ZeroAddress,
            { value }
        );
        const receipt = await tx.wait();
        if (!receipt) throw new Error("Transaction failed to be mined");

        const filter = factory.filters.EscrowCreated(
            deployer.address,
            undefined,
            recipient.address,
            undefined,
            undefined,
            undefined,
            undefined
        );
        const events = await factory.queryFilter(filter, receipt.blockNumber);
        if (events.length === 0) throw new Error("EscrowCreated event not found");
        const escrowAddr = events[0].args.escrowAddress;

        // const escrow = Escrow__factory.connect(escrowAddr, deployer as unknown as Signer);

        // Attempt to cancel before cancelAfter (msg.sender = factory); should revert as too early
        await expect(factory.cancelEscrow(escrowAddr)).to.be.revertedWith("Zescrow: too early to cancel");

        // Mine three blocks to reach cancelAfter
        await network.provider.send("evm_mine");
        await network.provider.send("evm_mine");
        await network.provider.send("evm_mine");

        const balBefore = await ethers.provider.getBalance(deployer.address);
        // Cancel via factory (now allowed)
        await (await factory.cancelEscrow(escrowAddr)).wait();
        const balAfter = await ethers.provider.getBalance(deployer.address);
        expect(balAfter).to.be.gt(balBefore);
    });
});

