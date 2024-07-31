import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { BurveSolanaContract } from "../target/types/burve_solana_contract";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { assert, expect } from "chai";
import { min } from "bn.js";

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

  it("Airdrop payer", async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );
  });

  const [burveBase] = PublicKey.findProgramAddressSync(
    [Buffer.from("burve")],
    program.programId
  );
  it("Initialize burve solana contract", async () => {
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

  const [projectMetadata] = PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("project-metadata"), mint.toBuffer()],
    program.programId
  );

  it("Create new project test passes", async () => {
    const bondingCurveType = {
      exponential: { a: new anchor.BN(10), b: new anchor.BN(10) },
    };
    await program.methods
      .createNewProjectWithSol({
        name: "quick project",
        symbol: "QP",
        uri: "https://my-project-data.com/metadata.json",
        admin: payer.publicKey,
        treasury: treasury.publicKey,
        mintTax: 50,
        burnTax: 50,
        bondingCurveType,
      })
      .accountsStrict({
        projectMetadata,
        payer: payer.publicKey,
        vault,
        mint,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();
  });

  it("Airdrop treasury to keep it alive", async () => {
    await provider.connection.confirmTransaction(
      // The existential deposit is 0.00203928 SOL (2,039,280 lamports) accourding to the solana docs https://solana.com/docs/more/exchange
      await provider.connection.requestAirdrop(treasury.publicKey, 3000000),
      "confirmed"
    );
  });

  it("Mint token test passes", async () => {
    await program.methods
      .mintTokenWithSol({
        amount: new anchor.BN(100),
        symbol: "QP",
        minReceive: new anchor.BN(100),
      })
      .accountsStrict({
        burveBase,
        projectMetadata,
        projectTreasury: treasury.publicKey,
        from: payer.publicKey,
        mint: mint,
        vault,
        mintTokenAccount: associatedAddress({
          mint: mint,
          owner: payer.publicKey,
        }),

        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();
  });

  it("Burn token test passes", async () => {
    await program.methods
      .burnTokenToSol({
        amount: new anchor.BN(10),
        symbol: "QP",
        minReceive: new anchor.BN(10),
      })
      .accountsStrict({
        burveBase,
        projectMetadata,
        projectTreasury: treasury.publicKey,
        from: payer.publicKey,
        burnTokenAccount: associatedAddress({
          mint: mint,
          owner: payer.publicKey,
        }),
        mint: mint,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();
  });

  const newPayer = Keypair.generate();

  const [newMint] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("token-mint-account"),
      anchor.utils.bytes.utf8.encode("NPS"),
    ],
    program.programId
  );

  const [newVault] = PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("vault"), newMint.toBuffer()],
    program.programId
  );

  const [newProjectMetadata] = PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("project-metadata"), newMint.toBuffer()],
    program.programId
  );

  const newTreasury = associatedAddress({
    mint: mint,
    owner: newPayer.publicKey,
  });

  it("Airdrop new payer", async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(newPayer.publicKey, 10000000000),
      "confirmed"
    );
  });

  it("Mint QP token to new treasury", async () => {
    await program.methods
      .mintTokenWithSol({
        amount: new anchor.BN(1000000),
        symbol: "QP",
        minReceive: new anchor.BN(100),
      })
      .accountsStrict({
        burveBase,
        projectMetadata,
        projectTreasury: treasury.publicKey,
        from: newPayer.publicKey,
        mint: mint,
        vault,
        mintTokenAccount: newTreasury,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([newPayer])
      .rpc();
  });

  it("Create new project with QP test passes", async () => {
    const bondingCurveType = {
      linear: { a: new anchor.BN(10), b: new anchor.BN(10) },
    };
    await program.methods
      .createNewProjectWithSpl({
        name: "new project with spl",
        symbol: "NPS",
        uri: "https://my-project-data.com/metadata.json",
        admin: newPayer.publicKey,
        treasury: newTreasury,
        mintTax: 50,
        burnTax: 50,
        bondingCurveType,
      })
      .accountsStrict({
        projectMetadata: newProjectMetadata,
        payer: newPayer.publicKey,
        vault: newVault,
        projectTreasury: newTreasury,
        mint: newMint,
        raisingToken: mint,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([newPayer])
      .rpc();
  });

  it("Airdrop new treasury to keep it alive", async () => {
    await provider.connection.confirmTransaction(
      // The existential deposit is 0.00203928 SOL (2,039,280 lamports) accourding to the solana docs https://solana.com/docs/more/exchange
      await provider.connection.requestAirdrop(newTreasury, 3000000),
      "confirmed"
    );
  });

  it("Mint token test passes", async () => {
    await program.methods
      .mintTokenWithSpl({
        amount: new anchor.BN(100),
        symbol: "NPS",
        minReceive: new anchor.BN(100),
      })
      .accountsStrict({
        burveBase,
        projectMetadata: newProjectMetadata,
        projectTreasury: newTreasury,
        signer: newPayer.publicKey,
        raisingToken: mint,
        fromAta: associatedAddress({
          mint: mint,
          owner: newPayer.publicKey,
        }),
        mint: newMint,
        vault: newVault,
        mintTokenAccount: associatedAddress({
          mint: newMint,
          owner: newPayer.publicKey,
        }),

        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([newPayer])
      .rpc();
  });

  it("Burn token test passes", async () => {
    await program.methods
      .burnTokenToSpl({
        amount: new anchor.BN(10),
        symbol: "NPS",
        minReceive: new anchor.BN(10),
      })
      .accountsStrict({
        burveBase,
        projectMetadata: newProjectMetadata,
        projectTreasury: newTreasury,
        signer: newPayer.publicKey,
        raisingToken: mint,
        burnTokenAccount: associatedAddress({
          mint: newMint,
          owner: newPayer.publicKey,
        }),
        mint: newMint,
        vault: newVault,
        toAta: associatedAddress({
          mint: mint,
          owner: newPayer.publicKey,
        }),
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([newPayer])
      .rpc();
  });

  it("Claim burve sol tax test passes", async () => {
    await program.methods
      .claimBurveSolTax({
        symbol: "QP",
      })
      .accountsStrict({
        burveBase,
        projectMetadata,
        admin: payer.publicKey,
        burveTreasury: payer.publicKey,
        mint,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([payer])
      .rpc();
  });

  it("Claim burve spl tax test passes", async () => {
    await program.methods
      .claimBurveSplTax({
        symbol: "NPS",
      })
      .accountsStrict({
        burveBase,
        projectMetadata: newProjectMetadata,
        admin: payer.publicKey,
        burveTreasury: newTreasury,
        mint: newMint,
        raisingToken: mint,
        vault: newVault,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([payer])
      .rpc();
  });
});
