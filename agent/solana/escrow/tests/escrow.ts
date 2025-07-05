import * as anchor from "@coral-xyz/anchor";
import type { Program } from "@coral-xyz/anchor";
import type { Escrow } from "../target/types/escrow.ts";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import { BN } from "bn.js";

describe("escrow", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.Escrow as Program<Escrow>;

    const PREFIX = Buffer.from("escrow");
    const AMOUNT = new BN(LAMPORTS_PER_SOL);

    function derivePda(sender: PublicKey, recipient: PublicKey): [PublicKey, number] {
        return PublicKey.findProgramAddressSync(
            [PREFIX, sender.toBuffer(), recipient.toBuffer()],
            program.programId
        );
    }

    let sender: Keypair;
    let recipient: Keypair;
    let escrowPda: PublicKey;

    beforeEach(async () => {
        sender = Keypair.generate();
        recipient = Keypair.generate();
        [escrowPda] = derivePda(sender.publicKey, recipient.publicKey);
        await airdrop(sender.publicKey, AMOUNT.mul(new BN(2)).toNumber());
    });

    it("should create and finish without `finishAfter`", async () => {
        await program.methods
            .createEscrow({
                amount: AMOUNT,
                finishAfter: null,
                cancelAfter: null,
            })
            .accounts({
                sender: sender.publicKey,
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([sender])
            .rpc();

        const before = await provider.connection.getBalance(recipient.publicKey);
        await program.methods
            .finishEscrow()
            .accounts({
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
            })
            .signers([recipient])
            .rpc();
        const after = await provider.connection.getBalance(recipient.publicKey);
        assert.ok(after - before >= AMOUNT.toNumber(), "recipient got funds");

        // PDA must be closed
        try {
            await program.account.escrow.fetch(escrowPda);
            assert.fail("Expected escrow PDA to be closed");
        } catch (err: unknown) {
            assert.match(
                String(err),
                /Account does not exist/,
                "escrow PDA was correctly closed"
            );
        }
    });

    it("should create and finish with `finishAfter`", async () => {
        // set finishAfter to slot 0 so it's immediately expired
        await program.methods
            .createEscrow({
                amount: AMOUNT,
                finishAfter: new BN(0),
                cancelAfter: null,
            })
            .accounts({
                sender: sender.publicKey,
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([sender])
            .rpc();

        const before = await provider.connection.getBalance(recipient.publicKey);
        await program.methods
            .finishEscrow()
            .accounts({
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
            })
            .signers([recipient])
            .rpc();
        const after = await provider.connection.getBalance(recipient.publicKey);
        assert.ok(after - before >= AMOUNT.toNumber(), "recipient got funds");

        try {
            await program.account.escrow.fetch(escrowPda);
            assert.fail("Expected escrow PDA to be closed");
        } catch (err: unknown) {
            assert.match(
                String(err),
                /Account does not exist/,
                "escrow PDA was correctly closed"
            );
        }
    });

    it("should create and cancel after `cancelAfter` (expiration)", async () => {
        await program.methods
            .createEscrow({
                amount: AMOUNT,
                finishAfter: null,
                cancelAfter: new BN(0),
            })
            .accounts({
                sender: sender.publicKey,
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([sender])
            .rpc();

        const before = await provider.connection.getBalance(sender.publicKey);
        await program.methods
            .cancelEscrow()
            .accounts({
                sender: sender.publicKey,
                escrowAccount: escrowPda,
            })
            .signers([sender])
            .rpc();
        const after = await provider.connection.getBalance(sender.publicKey);
        assert.ok(after - before >= AMOUNT.toNumber(), "sender got refund");

        try {
            await program.account.escrow.fetch(escrowPda);
            assert.fail("Expected escrow PDA to be closed");
        } catch (err: unknown) {
            assert.match(
                String(err),
                /Account does not exist/,
                "escrow PDA was correctly closed"
            );
        }
    });

    async function airdrop(pubkey: PublicKey, lamports: number): Promise<void> {
        const sig = await provider.connection.requestAirdrop(pubkey, lamports);
        await confirmTransaction(sig);
    }

    async function confirmTransaction(signature: string): Promise<void> {
        const latestBlockhash = await provider.connection.getLatestBlockhash();
        await provider.connection.confirmTransaction(
            { signature, ...latestBlockhash },
            "confirmed"
        );
    }
});
