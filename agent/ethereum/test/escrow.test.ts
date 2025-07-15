import { expect } from "chai";
import { ethers, network } from "hardhat";
import type { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";
import type { Signer } from "ethers";

import {
    type Escrow,
    Escrow__factory,
} from "../typechain-types";

describe("Escrow", () => {
    let deployer: SignerWithAddress;
    let recipient: SignerWithAddress;
    let escrow: Escrow;

    beforeEach(async () => {
        [deployer, recipient] = await ethers.getSigners();
        escrow = await new Escrow__factory(deployer as unknown as Signer).deploy();
        await escrow.waitForDeployment();
    });

    it("fails to finish before finishAfter, then succeeds", async () => {
        const startBlock = await ethers.provider.getBlockNumber();
        const finishAfter = startBlock + 3;
        const cancelAfter = startBlock + 4;
        const value = ethers.parseEther("1");

        const tx = await escrow.createEscrow(
            recipient.address,
            finishAfter,
            cancelAfter,
            { value }
        );
        const receipt = await tx.wait();
        if (!receipt) throw new Error("Transaction failed to be mined");

        const filter = escrow.filters.EscrowCreated(
            undefined,
            deployer.address,
            recipient.address,
            undefined,
            undefined,
            undefined
        );
        const events = await escrow.queryFilter(filter, receipt.blockNumber);
        if (events.length === 0) throw new Error("EscrowCreated event not found");
        const escrowId = events[0].args.escrowId;

        // Attempt to finish; should revert as too early
        await expect(escrow.connect(recipient).finishEscrow(escrowId)).to.be.revertedWithCustomError(escrow, "TooEarlyToFinish");

        // Mine two blocks to reach finishAfter
        await network.provider.send("evm_mine");
        await network.provider.send("evm_mine");

        const balBefore = await ethers.provider.getBalance(recipient.address);
        await (await escrow.connect(recipient).finishEscrow(escrowId)).wait();
        const balAfter = await ethers.provider.getBalance(recipient.address);
        expect(balAfter).to.be.gt(balBefore);
    });

    it("fails to cancel before cancelAfter, then succeeds", async () => {
        const startBlock = await ethers.provider.getBlockNumber();
        const finishAfter = startBlock + 3;
        const cancelAfter = startBlock + 4;
        const value = ethers.parseEther("1");

        const tx = await escrow.createEscrow(
            recipient.address,
            finishAfter,
            cancelAfter,
            { value }
        );
        const receipt = await tx.wait();
        if (!receipt) throw new Error("Transaction failed to be mined");

        const filter = escrow.filters.EscrowCreated(
            undefined,
            deployer.address,
            recipient.address,
            undefined,
            undefined,
            undefined
        );
        const events = await escrow.queryFilter(filter, receipt.blockNumber);
        if (events.length === 0) throw new Error("EscrowCreated event not found");
        const escrowId = events[0].args.escrowId;

        // Attempt to cancel before cancelAfter; should revert as too early
        await expect(escrow.cancelEscrow(escrowId)).to.be.revertedWithCustomError(escrow, "TooEarlyToCancel");

        // Mine three blocks to reach cancelAfter
        await network.provider.send("evm_mine");
        await network.provider.send("evm_mine");
        await network.provider.send("evm_mine");

        const balBefore = await ethers.provider.getBalance(deployer.address);
        // Cancel (now allowed)
        await (await escrow.cancelEscrow(escrowId)).wait();
        const balAfter = await ethers.provider.getBalance(deployer.address);
        expect(balAfter).to.be.gt(balBefore);
    });
});

