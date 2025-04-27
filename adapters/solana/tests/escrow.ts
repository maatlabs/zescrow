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

    const ESCROW_AMOUNT = new BN(LAMPORTS_PER_SOL);
    const ESCROW_EXPIRY = new BN(5);

    const depositor = Keypair.generate();
    let beneficiary: PublicKey;
    let escrowPda: PublicKey;

    before(async () => {
        await airdrop(depositor.publicKey, ESCROW_AMOUNT.muln(3).toNumber());
    });

    it("should create an escrow account", async () => {
        beneficiary = Keypair.generate().publicKey;
        [escrowPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("escrow"),
                depositor.publicKey.toBuffer(),
                beneficiary.toBuffer(),
            ],
            program.programId
        );

        await program.methods
            .createEscrow(ESCROW_AMOUNT, ESCROW_EXPIRY)
            .accounts({
                depositor: depositor.publicKey,
                beneficiary: beneficiary,
                escrow: escrowPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([depositor])
            .rpc();

        const account = await program.account.escrowAccount.fetch(escrowPda);
        assert.isTrue(account.depositor.equals(depositor.publicKey));
        assert.isTrue(account.beneficiary.equals(beneficiary));
        assert.isTrue(account.amount.eq(ESCROW_AMOUNT));
    });

    it("should release funds to beneficiary", async () => {
        const initialBalance = new BN(await getBalance(beneficiary));
        await program.methods
            .releaseEscrow()
            .accounts({
                escrow: escrowPda,
                beneficiary: beneficiary,
                depositor: depositor.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const finalBalance = new BN(await getBalance(beneficiary));
        const diff = finalBalance.sub(initialBalance);
        assert.isTrue(diff.gte(ESCROW_AMOUNT), `Expected ${ESCROW_AMOUNT.toString()}, got ${diff.toString()}`);
    });

    it("should refund depositor after expiry", async () => {
        const newBeneficiary = Keypair.generate().publicKey;
        const [newEscrowPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("escrow"),
                depositor.publicKey.toBuffer(),
                newBeneficiary.toBuffer(),
            ],
            program.programId
        );

        await program.methods
            .createEscrow(ESCROW_AMOUNT, ESCROW_EXPIRY)
            .accounts({
                depositor: depositor.publicKey,
                beneficiary: newBeneficiary,
                escrow: newEscrowPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([depositor])
            .rpc();

        const escrowState = await program.account.escrowAccount.fetch(newEscrowPda);
        await advanceSlot(escrowState.expiry.toNumber() + 1);

        const initialBalance = new BN(await getBalance(depositor.publicKey));
        await program.methods
            .refundEscrow()
            .accounts({
                escrow: newEscrowPda,
                depositor: depositor.publicKey,
                beneficiary: newBeneficiary,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const finalBalance = new BN(await getBalance(depositor.publicKey));
        const diff = finalBalance.sub(initialBalance);
        assert.isTrue(diff.gte(ESCROW_AMOUNT), `Expected ${ESCROW_AMOUNT.toString()}, got ${diff.toString()}`);
    });

    async function airdrop(pubkey: PublicKey, lamports: number) {
        const sig = await provider.connection.requestAirdrop(pubkey, lamports);
        await confirmTransaction(sig);
    }

    async function getBalance(pubkey: PublicKey): Promise<number> {
        return provider.connection.getBalance(pubkey);
    }

    async function advanceSlot(targetSlot: number) {
        while (true) {
            const currentSlot = await provider.connection.getSlot();
            if (currentSlot >= targetSlot) break;
            const dummy = Keypair.generate();
            const sig = await provider.connection.requestAirdrop(dummy.publicKey, 1_000_000);
            await confirmTransaction(sig);
        }
    }

    async function confirmTransaction(signature: string) {
        const latestBlockhash = await provider.connection.getLatestBlockhash();
        await provider.connection.confirmTransaction(
            { signature, ...latestBlockhash },
            "confirmed"
        );
    }
});