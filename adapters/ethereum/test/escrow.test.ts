import { expect } from "chai";
import { ethers, network } from "hardhat";
import type { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

import {
    type EscrowFactory,
    type EscrowFactory__factory,
    Escrow__factory,
} from "../typechain-types";

describe("Escrow", () => {
    let deployer: SignerWithAddress;
    let recipient: SignerWithAddress;
    let factory: EscrowFactory;

    beforeEach(async () => {
        const signers = await ethers.getSigners() as SignerWithAddress[];
        deployer = signers[0];
        recipient = signers[1];

        const Factory = (await ethers.getContractFactory(
            "EscrowFactory",
            deployer
        )) as EscrowFactory__factory;
        factory = await Factory.deploy();
        await factory.waitForDeployment();
    });

    it("fails to finish before finishAfter, then succeeds", async () => {
        const startBlock = await ethers.provider.getBlockNumber();
        const finishAfter = BigInt(startBlock) + 2n;
        const cancelAfter = finishAfter + 3n;

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
        if (!receipt) throw new Error("Transaction failed, no receipt");

        const filter = factory.filters.EscrowCreated(deployer.address, null, null);
        const events = await factory.queryFilter(filter, receipt.blockNumber);
        if (events.length === 0) throw new Error("EscrowCreated event not found");
        const event = events[0];
        const escrowAddr = event.args.escrowAddress;

        const escrow = Escrow__factory.connect(escrowAddr, recipient);
        await expect(escrow.finishEscrow("0x")).to.be.revertedWith("Zescrow: too early to finish");

        // 6. Advance two blocks, then succeed
        await network.provider.send("evm_mine");
        await network.provider.send("evm_mine");
        const balBefore = await ethers.provider.getBalance(recipient.address);
        await (await escrow.finishEscrow("0x")).wait();
        const balAfter = await ethers.provider.getBalance(recipient.address);
        expect(balAfter).to.be.gt(balBefore);
    });

    it("fails to cancel before cancelAfterBlock, then succeeds", async () => {
        const startBlock = await ethers.provider.getBlockNumber();
        // account for Hardhat mining a block at creation time
        const finishAfter = BigInt(startBlock) + 2n;
        const cancelAfter = finishAfter + 1n;

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
        if (!receipt) throw new Error("Transaction failed, no receipt");
        const filter = factory.filters.EscrowCreated(deployer.address, null, null);
        const events = await factory.queryFilter(filter, receipt.blockNumber);
        if (events.length === 0) throw new Error("EscrowCreated event not found");
        const escrowAddr = events[0].args.escrowAddress;

        const escrow = Escrow__factory.connect(escrowAddr, deployer);
        await expect(escrow.cancelEscrow()).to.be.revertedWith("Zescrow: too early to cancel");

        await network.provider.send("evm_mine");
        await network.provider.send("evm_mine");
        const balBefore = await ethers.provider.getBalance(deployer.address);
        await (await escrow.cancelEscrow()).wait();
        const balAfter = await ethers.provider.getBalance(deployer.address);
        expect(balAfter).to.be.gt(balBefore);
    });
});
