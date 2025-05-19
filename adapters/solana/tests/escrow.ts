import * as anchor from "@coral-xyz/anchor";
// biome-ignore lint/style/useImportType: <explanation>
import { Program, BN } from "@coral-xyz/anchor";
// biome-ignore lint/style/useImportType: <explanation>
import { Escrow } from "../target/types/escrow";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";

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

    it("should create and finish without condition", async () => {
        // 1) Create escrow with no ZK‐condition
        await program.methods
            .createEscrow({
                amount: AMOUNT,
                finishAfter: null,
                cancelAfter: null,
                hasConditions: false,
            })
            .accounts({
                sender: sender.publicKey,
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([sender])
            .rpc();

        // 2) Capture recipient balance, then finishEscrow (proof is ignored when hasConditions==false)
        const before = await provider.connection.getBalance(recipient.publicKey);
        await program.methods
            .finishEscrow({ proof: Buffer.alloc(0) })
            .accounts({
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
                verifierProgram: program.programId
            })
            .signers([recipient])
            .rpc();
        const after = await provider.connection.getBalance(recipient.publicKey);
        assert.ok(after - before >= AMOUNT.toNumber(), "recipient got funds");

        // 3) PDA must be closed
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
        // 1) Create escrow with immediate cancelAfter
        await program.methods
            .createEscrow({
                amount: AMOUNT,
                finishAfter: null,
                cancelAfter: new BN(0),
                hasConditions: false,
            })
            .accounts({
                sender: sender.publicKey,
                recipient: recipient.publicKey,
                escrowAccount: escrowPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([sender])
            .rpc();

        // 2) Capture sender balance, then cancelEscrow
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

        // 3) PDA must be closed
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

    // Helpers
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

