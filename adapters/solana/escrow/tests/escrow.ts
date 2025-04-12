// import * as anchor from "@coral-xyz/anchor";
// import { Program, BN } from "@coral-xyz/anchor";
// import { Escrow } from "../target/types/escrow";
// import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
// import { assert } from "chai";

// describe("escrow", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);
//   const program = anchor.workspace.Escrow as Program<Escrow>;

//   const ESCROW_AMOUNT = new BN(anchor.web3.LAMPORTS_PER_SOL);
//   const depositor = Keypair.generate();
//   const beneficiary = Keypair.generate();
//   let escrowAccount: Keypair;

//   before(async () => {
//     // Fund with 3x escrow amount to cover multiple transactions
//     await airdrop(depositor.publicKey, ESCROW_AMOUNT.muln(3).toNumber());
//   });

//   it("should create an escrow account", async () => {
//     escrowAccount = Keypair.generate();

//     await program.methods.createEscrow(ESCROW_AMOUNT)
//       .accounts({
//         depositor: depositor.publicKey,
//         beneficiary: beneficiary.publicKey,
//         escrow: escrowAccount.publicKey,
//       })
//       .signers([depositor, escrowAccount])
//       .rpc();

//     const account = await program.account.escrowAccount.fetch(escrowAccount.publicKey);
//     assert.isTrue(account.depositor.equals(depositor.publicKey));
//     assert.isTrue(account.beneficiary.equals(beneficiary.publicKey));
//     assert.isTrue(account.amount.eq(ESCROW_AMOUNT));
//   });

//   it("should release funds to beneficiary", async () => {
//     const initialBalance = new BN(await getBalance(beneficiary.publicKey));

//     await program.methods.releaseEscrow()
//       .accounts({
//         escrow: escrowAccount.publicKey,
//         beneficiary: beneficiary.publicKey,
//         depositor: depositor.publicKey,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();

//     const finalBalance = new BN(await getBalance(beneficiary.publicKey));
//     assert.isTrue(finalBalance.sub(initialBalance).eq(ESCROW_AMOUNT));
//   });

//   it("should refund depositor after expiry", async () => {
//     const newEscrow = Keypair.generate();
//     await program.methods.createEscrow(ESCROW_AMOUNT)
//       .accounts({
//         depositor: depositor.publicKey,
//         beneficiary: beneficiary.publicKey,
//         escrow: newEscrow.publicKey,
//       })
//       .signers([depositor, newEscrow])
//       .rpc();

//     const escrowState = await program.account.escrowAccount.fetch(newEscrow.publicKey);
//     await advanceSlot(escrowState.expiry.addn(1).toNumber());

//     const initialBalance = new BN(await getBalance(depositor.publicKey));

//     await program.methods.refundEscrow()
//       .accounts({
//         escrow: newEscrow.publicKey,
//         depositor: depositor.publicKey,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();

//     const finalBalance = new BN(await getBalance(depositor.publicKey));
//     assert.isTrue(finalBalance.sub(initialBalance).eq(ESCROW_AMOUNT));
//   });

//   // Utility functions
//   async function airdrop(pubkey: PublicKey, lamports: number) {
//     const sig = await provider.connection.requestAirdrop(pubkey, lamports);
//     await confirmTransaction(sig);
//   }

//   async function getBalance(pubkey: PublicKey): Promise<number> {
//     return provider.connection.getBalance(pubkey);
//   }

//   async function advanceSlot(targetSlot: number) {
//     const currentSlot = await provider.connection.getSlot();
//     if (currentSlot >= targetSlot) return;

//     const advanceKey = provider.wallet.publicKey;
//     await airdrop(advanceKey, 1_000_000_000);

//     while ((await provider.connection.getSlot()) < targetSlot) {
//       const tx = await provider.connection.requestAirdrop(advanceKey, 1);
//       await confirmTransaction(tx);
//     }
//   }

//   async function confirmTransaction(signature: string) {
//     const latestBlockhash = await provider.connection.getLatestBlockhash();
//     await provider.connection.confirmTransaction({
//       signature,
//       ...latestBlockhash,
//     }, 'confirmed');
//   }
// });