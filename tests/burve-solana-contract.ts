import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { BurveSolanaContract } from "../target/types/burve_solana_contract";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { assert, expect } from "chai";

const TOKEN_2022_PROGRAM_ID = new anchor.web3.PublicKey(
  "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
);

export function associatedAddress({
  mint,
  owner,
}: {
  mint: PublicKey;
  owner: PublicKey;
}): PublicKey {
  return PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_2022_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ASSOCIATED_PROGRAM_ID
  )[0];
}

describe("burve-solana-contract", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .BurveSolanaContract as Program<BurveSolanaContract>;

  const payer = Keypair.generate();

  it("airdrop payer", async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );
  });

  it("Initialize burve solana contract", async () => {
    const [burveBase] = PublicKey.findProgramAddressSync(
      [Buffer.from("burve")],
      program.programId
    );
    await program.methods
      .initialize({
        admin: payer.publicKey,
        treasury: payer.publicKey,
      })
      .accountsStrict({
        burveBase,
        systemProgram: anchor.web3.SystemProgram.programId,
        signer: payer.publicKey,
      })
      .signers([payer])
      .rpc();
  });

  const treasury = Keypair.generate();

  it("Create new project test passes", async () => {
    const [extraMetasAccount] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("extra-account-metas"),
        payer.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [mint] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("token-mint-account"),
        anchor.utils.bytes.utf8.encode("QP"),
      ],
      program.programId
    );
    const [vault] = PublicKey.findProgramAddressSync(
      [anchor.utils.bytes.utf8.encode("vault"), mint.toBuffer()],
      program.programId
    );

    const bondingCurveType = { exponential: { a: 10, b: 20 } };
    await program.methods
      .createNewProject({
        name: "quick project",
        symbol: "QP",
        uri: "https://my-project-data.com/metadata.json",
        admin: payer.publicKey,
        treasury: treasury.publicKey,
        mintTax: 50,
        burnTax: 50,
        bondingCurveType: bondingCurveType,
      })
      .accountsStrict({
        projectMetadata: extraMetasAccount,
        payer: payer.publicKey,
        raisingToken: null,
        vault,
        mint,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();
  });

  let mint = new Keypair();

  // it("Create mint account test passes", async () => {
  //   const [extraMetasAccount] = PublicKey.findProgramAddressSync(
  //     [
  //       anchor.utils.bytes.utf8.encode("extra-account-metas"),
  //       mint.publicKey.toBuffer(),
  //     ],
  //     program.programId
  //   );
  //   await program.methods
  //     .createMintAccount({
  //       name: "quick token",
  //       symbol: "QT",
  //       uri: "https://my-token-data.com/metadata.json",
  //       admin: payer.publicKey,
  //       mintTax: 0,
  //       burnTax: 0,
  //       bondingCurveType: undefined,
  //     })
  //     .accountsStrict({
  //       // FIXME: correct the account here
  //       projectMetadata: extraMetasAccount,
  //       payer: payer.publicKey,

  //       mint: mint.publicKey,
  //       //mintTokenAccount: associatedAddress({
  //       //  mint: mint.publicKey,
  //       //  owner: payer.publicKey,
  //       //}),
  //       extraMetasAccount: extraMetasAccount,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
  //       tokenProgram: TOKEN_2022_PROGRAM_ID,
  //       treasury: "",
  //     })
  //     .signers([mint, payer])
  //     .rpc();
  // });

  it("mint extension constraints test passes", async () => {
    try {
      const tx = await program.methods
        .checkMintExtensionsConstraints()
        .accountsStrict({
          authority: payer.publicKey,
          mint: mint.publicKey,
        })
        .signers([payer])
        .rpc();
      assert.ok(tx, "transaction should be processed without error");
    } catch (e) {
      assert.fail("should not throw error");
    }
  });
  it("mint extension constraints fails with invalid authority", async () => {
    const wrongAuth = Keypair.generate();
    try {
      const x = await program.methods
        .checkMintExtensionsConstraints()
        .accountsStrict({
          authority: wrongAuth.publicKey,
          mint: mint.publicKey,
        })
        .signers([payer, wrongAuth])
        .rpc();
      assert.fail("should have thrown an error");
    } catch (e) {
      expect(e, "should throw error");
    }
  });
});
