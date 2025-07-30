import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RecruSearch } from "../target/types/recru_search";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddressSync,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";

describe("recru-search", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const program = anchor.workspace.RecruSearch as Program<RecruSearch>;

  const admin = Keypair.generate();
  const researcher = Keypair.generate();
  const participant = Keypair.generate();
  const participant2 = Keypair.generate();

  let rewardMint: PublicKey;
  let researcherTokenAccount: PublicKey;

  const airdropSol = async (account: Keypair, amount: number) => {
    const signature = await provider.connection.requestAirdrop(
      account.publicKey,
      amount * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);
  };

  before(async () => {
    await airdropSol(admin, 1);
    await airdropSol(researcher, 1);
    await airdropSol(participant, 1);
    await airdropSol(participant2, 1);

    rewardMint = await createMint(
      provider.connection,
      researcher,
      researcher.publicKey,
      null,
      6
    );

    const tokenAccountInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      researcher,
      rewardMint,
      researcher.publicKey
    );
    researcherTokenAccount = tokenAccountInfo.address;

    await mintTo(
      provider.connection,
      researcher,
      rewardMint,
      researcherTokenAccount,
      researcher,
      1000000000
    );
  });

  it("Initializes the protocol", async () => {
    const [adminState] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin")],
      program.programId
    );

    await program.methods
      .initializeProtocol(250, 86400, 31536000)
      .accounts({
        adminState,
        protocolAdmin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    const adminStateAccount = await program.account.adminState.fetch(adminState);
    expect(adminStateAccount.protocolFeeBasisPoints).to.equal(250);
  });

  it("Creates a study", async () => {
    const studyId = new anchor.BN(1);
    const [study] = PublicKey.findProgramAddressSync(
      [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const now = new anchor.BN(Date.now() / 1000);
    const enrollmentStart = now.add(new anchor.BN(3600));
    const enrollmentEnd = enrollmentStart.add(new anchor.BN(86400 * 7));
    const dataCollectionEnd = enrollmentEnd.add(new anchor.BN(604800));

    await program.methods
      .createStudy(
        studyId,
        "Test Study",
        "A test study for research",
        enrollmentStart,
        enrollmentEnd,
        dataCollectionEnd,
        100,
        new anchor.BN(1000000)
      )
      .accounts({
        study,
        researcher: researcher.publicKey,
        systemProgram: SystemProgram.programId,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .signers([researcher])
      .rpc();

    const studyAccount = await program.account.studyAccount.fetch(study);
    expect(studyAccount.title).to.equal("Test Study");
    expect(studyAccount.status).to.deep.equal({ draft: {} });
  });

  it("Publishes a study", async () => {
    const studyId = new anchor.BN(1);
    const [study] = PublicKey.findProgramAddressSync(
      [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    await program.methods
      .publishStudy()
      .accounts({
        study,
        researcher: researcher.publicKey,
      })
      .signers([researcher])
      .rpc();

    const studyAccount = await program.account.studyAccount.fetch(study);
    expect(studyAccount.status).to.deep.equal({ published: {} });
  });

  it("Creates a reward vault", async () => {
    const studyId = new anchor.BN(1);
    const [study] = PublicKey.findProgramAddressSync(
      [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const [rewardVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), study.toBuffer()],
      program.programId
    );

    await program.methods
      .createRewardVault(studyId, new anchor.BN(10000000))
      .accounts({
        study,
        rewardVault,
        vaultTokenAccount: getAssociatedTokenAddressSync(rewardVault, rewardMint),
        rewardTokenMint: rewardMint,
        researcherTokenAccount,
        researcher: researcher.publicKey,
        associatedTokenProgram: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
        tokenProgram: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        systemProgram: SystemProgram.programId,
      })
      .signers([researcher])
      .rpc();

    const vaultAccount = await program.account.rewardVault.fetch(rewardVault);
    expect(vaultAccount.totalDeposited.toString()).to.equal("10000000");
  });

  it("Mints consent NFT", async () => {
    const studyId = new anchor.BN(1);
    const [study] = PublicKey.findProgramAddressSync(
      [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const [consent] = PublicKey.findProgramAddressSync(
      [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
      program.programId
    );

    const consentNftMint = Keypair.generate();
    const eligibilityProof = Buffer.from("test_eligibility_proof");

    await program.methods
      .mintConsentNft(studyId, eligibilityProof)
      .accounts({
        study,
        consent,
        consentNftMint: consentNftMint.publicKey,
        participant: participant.publicKey,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: new PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"),
      })
      .signers([participant, consentNftMint])
      .rpc();

    const consentAccount = await program.account.consentAccount.fetch(consent);
    expect(consentAccount.participant.toString()).to.equal(participant.publicKey.toString());
    expect(consentAccount.nftMint.toString()).to.equal(consentNftMint.publicKey.toString());
  });

  it("Submits data", async () => {
    const studyId = new anchor.BN(1);
    const [study] = PublicKey.findProgramAddressSync(
      [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const [consent] = PublicKey.findProgramAddressSync(
      [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
      program.programId
    );
    const [submission] = PublicKey.findProgramAddressSync(
      [Buffer.from("submission"), study.toBuffer(), participant.publicKey.toBuffer()],
      program.programId
    );

    const encryptedDataHash = Array.from(Buffer.alloc(32, 1));
    const ipfsCid = "QmTestHash123";

    await program.methods
      .submitData(encryptedDataHash, ipfsCid)
      .accounts({
        study,
        consent,
        submission,
        participant: participant.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([participant])
      .rpc();

    const submissionAccount = await program.account.submissionAccount.fetch(submission);
    expect(submissionAccount.participant.toString()).to.equal(participant.publicKey.toString());
    expect(submissionAccount.ipfsCid).to.equal(ipfsCid);
  });

  it("Distributes reward", async () => {
    const studyId = new anchor.BN(1);
    const [study] = PublicKey.findProgramAddressSync(
      [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const [rewardVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), study.toBuffer()],
      program.programId
    );
    const [consent] = PublicKey.findProgramAddressSync(
      [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
      program.programId
    );
    const [submission] = PublicKey.findProgramAddressSync(
      [Buffer.from("submission"), study.toBuffer(), participant.publicKey.toBuffer()],
      program.programId
    );

    const participantTokenAccount = getAssociatedTokenAddressSync(
      rewardMint,
      participant.publicKey
    );

    await program.methods
      .distributeReward()
      .accounts({
        study,
        rewardVault,
        vaultTokenAccount: getAssociatedTokenAddressSync(rewardVault, rewardMint),
        consent,
        submission,
        rewardMint,
        participantTokenAccount,
        participant: participant.publicKey,
        researcher: researcher.publicKey,
        associatedTokenProgram: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
        tokenProgram: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        systemProgram: SystemProgram.programId,
      })
      .signers([researcher])
      .rpc();

    const submissionAccount = await program.account.submissionAccount.fetch(submission);
    expect(submissionAccount.rewardClaimed).to.be.true;
  });

  it("Closes a study", async () => {
    const studyId = new anchor.BN(1);
    const [study] = PublicKey.findProgramAddressSync(
      [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    await program.methods
      .closeStudy()
      .accounts({
        study,
        researcher: researcher.publicKey,
      })
      .signers([researcher])
      .rpc();

    const studyAccount = await program.account.studyAccount.fetch(study);
    expect(studyAccount.status).to.deep.equal({ closed: {} });
  });

  it("Handles edge cases", async () => {
    const studyId = new anchor.BN(2);
    const [study] = PublicKey.findProgramAddressSync(
      [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    try {
      await program.methods
        .publishStudy()
        .accounts({
          study,
          researcher: researcher.publicKey,
        })
        .signers([researcher])
        .rpc();
      expect.fail("Should have failed - study not created");
    } catch (error) {
      expect(error.message).to.include("AnchorError");
    }
  });
});
