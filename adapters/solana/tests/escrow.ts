import * as anchor from "@coral-xyz/anchor";
// biome-ignore lint/style/useImportType: <explanation>
import { Program, BN } from "@coral-xyz/anchor";
// biome-ignore lint/style/useImportType: <explanation>
import { Escrow } from "../target/types/escrow";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
// biome-ignore lint/style/useNodejsImportProtocol: <explanation>
import { createHash } from "crypto";

describe("escrow", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.Escrow as Program<Escrow>;

    const PREFIX = Buffer.from("escrow");
    const AMOUNT = new BN(LAMPORTS_PER_SOL);

    function derivePda(s: PublicKey, r: PublicKey): [PublicKey, number] {
        return PublicKey.findProgramAddressSync(
            [PREFIX, s.toBuffer(), r.toBuffer()],
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

    it("should create and finish without condition", async () => {
        await program.methods
            .createEscrow({
                amount: AMOUNT,
                finishAfter: null,
                cancelAfter: null,
                condition: null,
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
            .finishEscrow(null)
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
            // biome-ignore lint/suspicious/noExplicitAny: <explanation>
        } catch (err: any) {
            assert.ok(
                err.message.includes("Account does not exist"),
                "escrow PDA was correctly closed"
            );
        }
    });

    it("should create and cancel after expiration", async () => {
        await program.methods
            .createEscrow({
                amount: AMOUNT,
                finishAfter: null,
                cancelAfter: new BN(0),
                condition: null,
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
            // biome-ignore lint/suspicious/noExplicitAny: <explanation>
        } catch (err: any) {
            assert.ok(
                err.message.includes("Account does not exist"),
                "escrow PDA was correctly closed"
            );
        }
    });

    it("should create and finish with SHA-256 condition", async () => {
        const preimage = Buffer.from("secret");
        const hash = createHash("sha256").update(preimage).digest();
        const cond = Array.from(hash);

        await program.methods
            .createEscrow({
                amount: AMOUNT,
                finishAfter: null,
                cancelAfter: null,
                condition: cond,
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
            .finishEscrow(preimage)
            .accounts({
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
            })
            .signers([recipient])
            .rpc();

        const after = await provider.connection.getBalance(recipient.publicKey);
        assert.ok(after - before >= AMOUNT.toNumber(), "recipient got funds");
    });

    async function airdrop(pubkey: PublicKey, lamports: number) {
        const sig = await provider.connection.requestAirdrop(pubkey, lamports);
        await confirmTransaction(sig);
    }

    async function confirmTransaction(signature: string) {
        const latestBlockhash = await provider.connection.getLatestBlockhash();
        await provider.connection.confirmTransaction(
            { signature, ...latestBlockhash },
            "confirmed"
        );
    }
});

