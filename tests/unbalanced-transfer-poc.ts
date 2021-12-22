import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { UnbalancedTransferPoc } from "../target/types/unbalanced_transfer_poc";

describe("unbalanced-transfer-poc", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  var otherWallet: Keypair;
  // @ts-ignore
  const program = anchor.workspace
    .UnbalancedTransferPoc as Program<UnbalancedTransferPoc>;
  const wallet = program.provider.wallet;
  const AMOUNT = 1 * LAMPORTS_PER_SOL;
  const connection = program.provider.connection;

  before(async () => {
    otherWallet = Keypair.generate();
    await connection.requestAirdrop(
      otherWallet.publicKey,
      10 * LAMPORTS_PER_SOL
    );
  });

  it("Is initialized!", async () => {
    const [prefix, prefixBump] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("prefix"),
        wallet.publicKey.toBuffer(),
        new anchor.BN(AMOUNT).toBuffer("le", 8),
      ],
      program.programId
    );
    const [escrow, escrowBump] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("prefix"),
        wallet.publicKey.toBuffer(),
        anchor.utils.bytes.utf8.encode("escrow"),
      ],
      program.programId
    );
    const tx = await program.rpc.initialize(
      prefixBump,
      escrowBump,
      new anchor.BN(AMOUNT),
      {
        accounts: {
          escrowAccount: escrow,
          prefixAccount: prefix,
          systemProgram: SystemProgram.programId,
          payer: wallet.publicKey,
        },
      }
    );
    await connection.confirmTransaction(tx);
    console.log("Your transaction signature", tx);
  });

  it("can close", async () => {
    const [prefix, prefixBump] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("prefix"),
        wallet.publicKey.toBuffer(),
        new anchor.BN(AMOUNT).toBuffer("le", 8),
      ],
      program.programId
    );
    const [escrow, escrowBump] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("prefix"),
        wallet.publicKey.toBuffer(),
        anchor.utils.bytes.utf8.encode("escrow"),
      ],
      program.programId
    );
    console.log(
      prefix.toBase58(),
      escrow.toBase58(),
      wallet.publicKey.toBase58(),
      otherWallet.publicKey.toBase58()
    );
    const tx = await program.rpc.close(escrowBump, {
      accounts: {
        escrowAccount: escrow,
        prefixAccount: prefix,
        systemProgram: SystemProgram.programId,
        payer: otherWallet.publicKey,
        user: wallet.publicKey,
      },
      signers: [otherWallet],
    });
    await connection.confirmTransaction(tx);
    console.log("Your transaction signature", tx);
  });
});
