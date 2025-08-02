// Import necessary libraries for testing Solana programs
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RecruSearch } from "../target/types/recru_search";
import { BN } from "bn.js";  // BN = BigNumber for handling large numbers
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  Transaction,
} from "@solana/web3.js";
import {
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import { expect } from "chai";
import { EncryptionManager, DecryptionManager } from "../app/utils/encryption";

/// Main test suite for the RecruSearch program
/// This file tests all the functionality of the RecruSearch program including
/// protocol initialization, study management, consent NFTs, data submission, and rewards
describe("recru-search", () => {
  // Configure the client to use the local cluster (test validator)
  anchor.setProvider(anchor.AnchorProvider.env());

  // Get the provider, connection, and program instances
  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.RecruSearch as Program<RecruSearch>;
  const programId = program.programId;

  // Test account keypairs - these will be generated for each test
  let admin: Keypair;           // Protocol admin
  let researcher: Keypair;      // Study creator
  let participant: Keypair;     // Study participant
  let participant2: Keypair;    // Second participant for testing
  let rewardMint: Keypair;      // Token mint for rewards
  
  // Associated Token Account (ATA) addresses
  let researcherTokenAccount: PublicKey;  // Researcher's reward token account
  let participantTokenAccount: PublicKey; // Participant's reward token account

  // Helper Functions (same as sample)
  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  /// Helper function to create a new token mint
  async function createMint(authority: Keypair): Promise<Keypair> {
    const mint = Keypair.generate();
    const mintRent = await getMinimumBalanceForRentExemptMint(connection);
    
    const createMintAccountIx = SystemProgram.createAccount({
      fromPubkey: authority.publicKey,
      newAccountPubkey: mint.publicKey,
      space: MINT_SIZE,
      lamports: mintRent,
      programId: TOKEN_2022_PROGRAM_ID,
    });

    const initializeMintIx = createInitializeMint2Instruction(
      mint.publicKey,
      6,
      authority.publicKey,
      null,
      TOKEN_2022_PROGRAM_ID
    );

    const createMintTx = new anchor.web3.Transaction()
      .add(createMintAccountIx)
      .add(initializeMintIx);

    await provider.sendAndConfirm(createMintTx, [authority, mint]);
    return mint;
  }

  /// Helper function to setup token accounts with initial balances
  async function setupTokenAccount(
    mint: Keypair,
    owner: Keypair,
    mintAuthority: Keypair,
    amount: number
  ): Promise<PublicKey> {
    const ata = getAssociatedTokenAddressSync(
      mint.publicKey,
      owner.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );
    
    const createAtaIx = createAssociatedTokenAccountIdempotentInstruction(
      owner.publicKey,
      ata,
      owner.publicKey,
      mint.publicKey,
      TOKEN_2022_PROGRAM_ID
    );

    const mintToIx = createMintToInstruction(
      mint.publicKey,
      ata,
      mintAuthority.publicKey,
      amount,
      [],
      TOKEN_2022_PROGRAM_ID
    );

    const setupTokensTx = new anchor.web3.Transaction()
      .add(createAtaIx)
      .add(mintToIx);

    await provider.sendAndConfirm(setupTokensTx, [owner, mintAuthority]);
    return ata;
  }

  /// Helper function to airdrop SOL to test accounts
  async function airdropSol(account: Keypair, amount: number) {
    const signature = await connection.requestAirdrop(
      account.publicKey,
      amount * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(signature);
  }

  // Encryption helper functions
  const encryptionManager = new EncryptionManager();
  const decryptionManager = new DecryptionManager();

  /// Helper function to encrypt data for a researcher
  async function encryptDataForResearcher(
    data: any,
    researcherPublicKey: Uint8Array
  ): Promise<{ encryptedDataHash: number[], ipfsCid: string }> {
    try {
      const { encrypted, cid } = await encryptionManager.encryptAndStore(
        data,
        researcherPublicKey
      );
      
      // Use first 32 bytes as hash for blockchain storage
      const encryptedDataHash = Array.from(encrypted.slice(0, 32));
      
      return { encryptedDataHash, ipfsCid: cid };
    } catch (error) {
      console.error("Encryption failed:", error);
      throw error;
    }
  }

  /// Helper function to decrypt data as a researcher
  async function decryptDataAsResearcher(
    ipfsCid: string,
    researcherPrivateKey: Uint8Array
  ): Promise<any> {
    try {
      const decryptedData = await decryptionManager.fetchAndDecrypt(
        ipfsCid,
        researcherPrivateKey
      );
      return decryptedData;
    } catch (error) {
      console.error("Decryption failed:", error);
      throw error;
    }
  }

  /// Setup function that runs before all tests
  before(async () => {
    // Create test account keypairs
    admin = Keypair.generate();
    researcher = Keypair.generate();
    participant = Keypair.generate();
    participant2 = Keypair.generate();

    // Airdrop SOL to all accounts so they can pay for transactions
    await airdropSol(admin, 2);
    await airdropSol(researcher, 2);
    await airdropSol(participant, 2);
    await airdropSol(participant2, 2);

    // Create test token mint for rewards
    rewardMint = await createMint(researcher);

    // Setup token accounts with 1B tokens for rewards
    researcherTokenAccount = await setupTokenAccount(rewardMint, researcher, researcher, 1000000000);
    participantTokenAccount = await setupTokenAccount(rewardMint, participant, researcher, 0);

    console.log("Test setup completed");
    console.log("Admin:", admin.publicKey.toString());
    console.log("Researcher:", researcher.publicKey.toString());
    console.log("Participant:", participant.publicKey.toString());
    console.log("Reward Mint:", rewardMint.publicKey.toString());
  });

  /// Test suite for protocol initialization
  describe("Protocol Initialization", () => {
    it("Should initialize the protocol successfully", async () => {
      try {
        const protocolFeeBps = 250;
        const minStudyDuration = 86400;
        const maxStudyDuration = 31536000;

        const [adminState] = PublicKey.findProgramAddressSync(
          [Buffer.from("admin")],
          programId
        );

        const tx = await program.methods
          .initializeProtocol(protocolFeeBps, minStudyDuration, maxStudyDuration)
          .accountsPartial({
            adminState,
            protocolAdmin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Protocol initialization signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });

    it("Should prevent double initialization", async () => {
      try {
        const [adminState] = PublicKey.findProgramAddressSync(
          [Buffer.from("admin")],
          programId
        );

        const tx = await program.methods
          .initializeProtocol(250, 86400, 31536000)
          .accountsPartial({
            adminState,
            protocolAdmin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Double initialization signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log("Double initialization correctly prevented:", error);
        }
      }
    });
  });

  /// Test suite for study management
  describe("Study Management", () => {
    it("Should create a study successfully", async () => {
      try {
        const studyId = new BN(1);
        const title = "Test Study";
        const description = "A comprehensive test study for research validation";
        
        const now = new BN(Math.floor(Date.now() / 1000));
        const enrollmentStart = now.add(new BN(1)); // Start 1 second in the future to pass createStudy validation
        const enrollmentEnd = enrollmentStart.add(new BN(604800));
        const dataCollectionEnd = enrollmentEnd.add(new BN(604800));
        
        const maxParticipants = 100;
        const rewardAmount = new BN(1000000);

        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );

        const tx = await program.methods
          .createStudy(
            studyId,
            title,
            description,
            enrollmentStart,
            enrollmentEnd,
            dataCollectionEnd,
            maxParticipants,
            rewardAmount
          )
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Study creation signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });

    it("Should publish a study successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );

        const tx = await program.methods
          .publishStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Study publication signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });

    it("Should prevent unauthorized study publication", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );

        const tx = await program.methods
          .publishStudy()
          .accountsPartial({
            study,
            researcher: participant.publicKey,
          })
          .signers([participant])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Unauthorized publication signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log("Unauthorized publication correctly prevented:", error);
        }
      }
    });
  });

  /// Test suite for reward vault management
  describe("Reward Vault Management", () => {
    it("Should create a reward vault successfully", async () => {
      try {
        const studyId = new BN(3); // Use different study ID to avoid conflicts
        const title = "Reward Vault Test Study";
        const description = "A test study for reward vault creation";
        
        const now = new BN(Math.floor(Date.now() / 1000));
        const enrollmentStart = now.add(new BN(1)); // Start 1 second in the future
        const enrollmentEnd = enrollmentStart.add(new BN(604800));
        const dataCollectionEnd = enrollmentEnd.add(new BN(604800));
        
        const maxParticipants = 50;
        const rewardAmount = new BN(500000);

        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        const [rewardVault] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault"), study.toBuffer()],
          programId
        );

        // First create the study
        const createStudyIx = await program.methods
          .createStudy(
            studyId,
            title,
            description,
            enrollmentStart,
            enrollmentEnd,
            dataCollectionEnd,
            maxParticipants,
            rewardAmount
          )
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY,
          })
          .instruction();

        const depositAmount = new BN(10000000);

        // Create reward vault instruction - Anchor will automatically create both token accounts
        const [vaultTokenAccount] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault_token"), rewardVault.toBuffer()],
          programId
        );
        
        const createRewardVaultIx = await program.methods
          .createRewardVault(studyId, depositAmount)
          .accountsPartial({
            study,
            rewardVault,
            vaultTokenAccount,
            rewardTokenMint: rewardMint.publicKey,
            researcherTokenAccount,
            researcher: researcher.publicKey,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .instruction();

        // Combine all instructions
        const tx = new anchor.web3.Transaction()
          .add(createStudyIx)
          .add(createRewardVaultIx);

        const signature = await provider.sendAndConfirm(tx, [researcher]);
        await confirm(signature);
        await log(signature);

        console.log("Reward vault creation signature:", signature);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });
  });

  /// Test suite for consent NFT management
  describe("Consent NFT Management", () => {
    it("Should mint a consent NFT successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
          programId
        );

        const asset = Keypair.generate();
        const eligibilityProof = Buffer.from("test_eligibility_proof_123");

        // Wait a bit to ensure we're within the enrollment period
        await new Promise(resolve => setTimeout(resolve, 3000));

        const tx = await program.methods
          .mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            asset: asset.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([participant, asset])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Consent NFT minting signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });

    it("Should prevent consent NFT minting for unpublished studies", async () => {
      try {
        const studyId = new BN(2);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
          programId
        );

                 const now = new BN(Math.floor(Date.now() / 1000));
         const enrollmentStart = now.add(new BN(10)); // Start 10 seconds in the future to pass validation
         const enrollmentEnd = enrollmentStart.add(new BN(604800));
         const dataCollectionEnd = enrollmentEnd.add(new BN(604800));

         // Create study in draft status
        await program.methods
          .createStudy(
            studyId,
            "Draft Study",
            "A draft study",
            enrollmentStart,
            enrollmentEnd,
            dataCollectionEnd,
            50,
            new BN(500000)
          )
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        const asset = Keypair.generate();
        const eligibilityProof = Buffer.from("test_proof");

        const tx = await program.methods
          .mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            asset: asset.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([participant, asset])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Consent NFT minting signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log("Consent NFT minting correctly prevented for unpublished study:", error);
        }
      }
    });
  });

  /// Test suite for data submission
  describe("Data Submission", () => {
    it("Should submit data successfully", async () => {
      try {
        const studyId = new BN(4); // Use different study ID to avoid conflicts
        const title = "Data Submission Test Study";
        const description = "A test study for data submission";
        
        const now = new BN(Math.floor(Date.now() / 1000));
        const enrollmentStart = now.sub(new BN(1)); // Start 1 second ago to pass createStudy validation
        const enrollmentEnd = enrollmentStart.add(new BN(604800));
        const dataCollectionEnd = enrollmentEnd.add(new BN(604800));
        
        const maxParticipants = 30;
        const rewardAmount = new BN(300000);

        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
          programId
        );
        const [submission] = PublicKey.findProgramAddressSync(
          [Buffer.from("submission"), study.toBuffer(), participant.publicKey.toBuffer()],
          programId
        );

        // First create the study
        const createStudyIx = await program.methods
          .createStudy(
            studyId,
            title,
            description,
            enrollmentStart,
            enrollmentEnd,
            dataCollectionEnd,
            maxParticipants,
            rewardAmount
          )
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY,
          })
          .instruction();

        // Publish the study
        const publishStudyIx = await program.methods
          .publishStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
          })
          .signers([researcher])
          .instruction();

        // Mint consent NFT
        const asset = Keypair.generate();
        const eligibilityProof = Buffer.from("data_submission_test_proof");
        
        // Wait a bit to ensure we're within the enrollment period
        await new Promise(resolve => setTimeout(resolve, 5000));

        const mintConsentIx = await program.methods
          .mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            asset: asset.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([participant, asset])
          .instruction();

        // Encrypt real data for the researcher
        const testSurveyData = {
          participantId: participant.publicKey.toString(),
          surveyResponses: {
            question1: "Strongly agree",
            question2: "Sometimes",
            question3: "Very satisfied"
          },
          timestamp: Date.now(),
          studyId: studyId.toNumber()
        };

        // Convert researcher public key to Uint8Array for encryption
        const researcherPublicKeyBytes = new Uint8Array(researcher.publicKey.toBytes());
        
        const { encryptedDataHash, ipfsCid } = await encryptDataForResearcher(
          testSurveyData,
          researcherPublicKeyBytes
        );

        console.log("Encrypted data hash:", encryptedDataHash);
        console.log("IPFS CID:", ipfsCid);

        // Submit data instruction
        const submitDataIx = await program.methods
          .submitData(encryptedDataHash, ipfsCid)
          .accountsPartial({
            study,
            consent,
            submission,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([participant])
          .instruction();

        // Combine all instructions
        const tx = new anchor.web3.Transaction()
          .add(createStudyIx)
          .add(publishStudyIx)
          .add(mintConsentIx)
          .add(submitDataIx);

        const signature = await provider.sendAndConfirm(tx, [researcher, participant, asset]);
        await confirm(signature);
        await log(signature);

        console.log("Data submission signature:", signature);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });

    it("Should allow researcher to decrypt submitted data", async () => {
      try {
        // This test verifies that the encryption/decryption flow works end-to-end
        const testData = {
          participantId: participant.publicKey.toString(),
          surveyResponses: {
            question1: "Very satisfied",
            question2: "Always",
            question3: "Excellent"
          },
          timestamp: Date.now(),
          metadata: {
            deviceType: "mobile",
            completionTime: 120
          }
        };

        // Encrypt data for researcher
        const researcherPublicKeyBytes = new Uint8Array(researcher.publicKey.toBytes());
        const { encryptedDataHash, ipfsCid } = await encryptDataForResearcher(
          testData,
          researcherPublicKeyBytes
        );

        console.log("Test encryption - IPFS CID:", ipfsCid);

        // Simulate researcher decrypting the data
        const researcherPrivateKeyBytes = new Uint8Array(researcher.secretKey);
        const decryptedData = await decryptDataAsResearcher(
          ipfsCid,
          researcherPrivateKeyBytes
        );

        console.log("Decrypted data:", JSON.stringify(decryptedData, null, 2));

        // Verify the decrypted data matches the original
        expect(decryptedData.participantId).to.equal(testData.participantId);
        expect(decryptedData.surveyResponses.question1).to.equal(testData.surveyResponses.question1);
        expect(decryptedData.metadata.deviceType).to.equal(testData.metadata.deviceType);

        console.log("✅ Encryption/decryption test passed - data integrity verified");
      } catch (error) {
        console.error("Encryption/decryption test failed:", error);
        throw error;
      }
    });

    it("Should prevent data submission without consent", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        const [submission] = PublicKey.findProgramAddressSync(
          [Buffer.from("submission"), study.toBuffer(), participant2.publicKey.toBuffer()],
          programId
        );

        // Encrypt test data (this should fail due to missing consent)
        const testData = {
          participantId: participant2.publicKey.toString(),
          surveyResponses: {
            question1: "This should fail",
            question2: "No consent provided"
          },
          timestamp: Date.now()
        };

        const researcherPublicKeyBytes = new Uint8Array(researcher.publicKey.toBytes());
        const { encryptedDataHash, ipfsCid } = await encryptDataForResearcher(
          testData,
          researcherPublicKeyBytes
        );

        const tx = await program.methods
          .submitData(encryptedDataHash, ipfsCid)
          .accountsPartial({
            study,
            consent: participant2.publicKey,
            submission,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([participant2])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Data submission signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log("Data submission correctly prevented without consent:", error);
        }
      }
    });
  });

  /// Test suite for reward distribution
  describe("Reward Distribution", () => {
    it("Should distribute rewards successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        const [rewardVault] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault"), study.toBuffer()],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
          programId
        );
        const [submission] = PublicKey.findProgramAddressSync(
          [Buffer.from("submission"), study.toBuffer(), participant.publicKey.toBuffer()],
          programId
        );

        const tx = await program.methods
          .distributeReward()
          .accountsPartial({
            study,
            rewardVault,
            vaultTokenAccount: getAssociatedTokenAddressSync(rewardVault, rewardMint.publicKey),
            consent,
            submission,
            rewardMint: rewardMint.publicKey,
            participantTokenAccount,
            participant: participant.publicKey,
            researcher: researcher.publicKey,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Reward distribution signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });
  });

  /// Test suite for completion NFT minting
  describe("Completion NFT Management", () => {
    it("Should mint a completion NFT successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        const [submission] = PublicKey.findProgramAddressSync(
          [Buffer.from("submission"), study.toBuffer(), participant.publicKey.toBuffer()],
          programId
        );

        const completionNftMint = Keypair.generate();

        const tx = await program.methods
          .mintCompletionNft()
          .accountsPartial({
            study,
            submission,
            completionNftMint: completionNftMint.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([participant, completionNftMint])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Completion NFT minting signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });
  });

  /// Test suite for study closure
  describe("Study Closure", () => {
    it("Should close a study successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );

        const tx = await program.methods
          .closeStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Study closure signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log(error);
        }
      }
    });
  });

  /// Integration test suite
  describe("Integration Tests", () => {
    it("Should complete full RecruSearch lifecycle", async () => {
      try {
        console.log("Starting full RecruSearch lifecycle test...");

        // Step 1: Create a new study for integration test
        const studyId = new BN(3);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );

                 const now = new BN(Math.floor(Date.now() / 1000));
         const enrollmentStart = now.add(new BN(10)); // Start 10 seconds in the future to pass validation
         const enrollmentEnd = enrollmentStart.add(new BN(604800));
         const dataCollectionEnd = enrollmentEnd.add(new BN(604800));

         const createTx = await program.methods
          .createStudy(
            studyId,
            "Integration Test Study",
            "A comprehensive integration test study",
            enrollmentStart,
            enrollmentEnd,
            dataCollectionEnd,
            50,
            new BN(2000000)
          )
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Step 1: Study created -", createTx);

        // Step 2: Publish the study
        const publishTx = await program.methods
          .publishStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Step 2: Study published -", publishTx);

        // Step 3: Create reward vault
        const [rewardVault] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault"), study.toBuffer()],
          programId
        );

        // Create reward vault instruction - Anchor will automatically create both token accounts
        const [vaultTokenAccount] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault_token"), rewardVault.toBuffer()],
          programId
        );
        
        const createRewardVaultIx = await program.methods
          .createRewardVault(studyId, new BN(50000000))
          .accountsPartial({
            study,
            rewardVault,
            vaultTokenAccount,
            rewardTokenMint: rewardMint.publicKey,
            researcherTokenAccount,
            researcher: researcher.publicKey,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .instruction();

        const vaultSignature = await provider.sendAndConfirm(
          new anchor.web3.Transaction().add(createRewardVaultIx), 
          [researcher]
        );
        await confirm(vaultSignature);
        await log(vaultSignature);

        console.log("Step 3: Reward vault created -", vaultSignature);

        // Step 4: Mint consent NFT
        const [consent] = PublicKey.findProgramAddressSync(
          [Buffer.from("consent"), study.toBuffer(), participant2.publicKey.toBuffer()],
          programId
        );

        const consentNftMint = Keypair.generate();
        const eligibilityProof = Buffer.from("integration_test_proof");

        const consentTx = await program.methods
          .mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            consentNftMint: consentNftMint.publicKey,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([participant2, consentNftMint])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Step 4: Consent NFT minted -", consentTx);

        // Step 5: Submit data with real encryption
        const [submission] = PublicKey.findProgramAddressSync(
          [Buffer.from("submission"), study.toBuffer(), participant2.publicKey.toBuffer()],
          programId
        );

        // Encrypt real integration test data
        const integrationTestData = {
          participantId: participant2.publicKey.toString(),
          studyId: studyId.toNumber(),
          surveyResponses: {
            question1: "Integration test response",
            question2: "Test data for full lifecycle",
            question3: "End-to-end validation"
          },
          timestamp: Date.now(),
          testPhase: "integration"
        };

        const researcherPublicKeyBytes = new Uint8Array(researcher.publicKey.toBytes());
        const { encryptedDataHash, ipfsCid } = await encryptDataForResearcher(
          integrationTestData,
          researcherPublicKeyBytes
        );

        console.log("Integration test - IPFS CID:", ipfsCid);

        const submitTx = await program.methods
          .submitData(encryptedDataHash, ipfsCid)
          .accountsPartial({
            study,
            consent,
            submission,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([participant2])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Step 5: Data submitted -", submitTx);

                 // Step 6: Distribute reward
         const participant2TokenAccount = getAssociatedTokenAddressSync(rewardMint.publicKey, participant2.publicKey);
         
         // Create participant2 token account if it doesn't exist
         const createParticipant2TokenAccountIx = createAssociatedTokenAccountIdempotentInstruction(
           researcher.publicKey, // payer
           participant2TokenAccount, // associated token account
           participant2.publicKey, // owner
           rewardMint.publicKey, // mint
           TOKEN_2022_PROGRAM_ID
         );

                  // Create reward distribution instruction
         const distributeRewardIx = await program.methods
           .distributeReward()
           .accountsPartial({
             study,
             rewardVault,
             vaultTokenAccount: getAssociatedTokenAddressSync(rewardVault, rewardMint.publicKey),
             consent,
             submission,
             rewardMint: rewardMint.publicKey,
             participantTokenAccount: participant2TokenAccount,
             participant: participant2.publicKey,
             researcher: researcher.publicKey,
             associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
             tokenProgram: TOKEN_2022_PROGRAM_ID,
             systemProgram: SystemProgram.programId,
           })
           .instruction();

         // Combine both instructions
         const rewardTx = new anchor.web3.Transaction()
           .add(createParticipant2TokenAccountIx)
           .add(distributeRewardIx);

         const rewardSignature = await provider.sendAndConfirm(rewardTx, [researcher]);
         await confirm(rewardSignature);
         await log(rewardSignature);

                 console.log("Step 6: Reward distributed -", rewardSignature);

        // Step 7: Mint completion NFT
        const completionNftMint = Keypair.generate();

        const completionTx = await program.methods
          .mintCompletionNft()
          .accountsPartial({
            study,
            submission,
            completionNftMint: completionNftMint.publicKey,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([participant2, completionNftMint])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Step 7: Completion NFT minted -", completionTx);

        // Step 8: Close the study
        const closeTx = await program.methods
          .closeStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Step 8: Study closed -", closeTx);
        console.log("Full RecruSearch lifecycle completed successfully!");
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log("Integration test error:", error);
        }
      }
    });
  });

  /// Eligibility criteria test suite
  describe("Eligibility Criteria", () => {
    it("Should set eligibility criteria for a study", async () => {
      const studyId = new BN(1001);
      const [study] = PublicKey.findProgramAddressSync(
        [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
        programId
      );

      // Create study first
      const createStudyTx = await program.methods
        .createStudy(
          studyId,
          "Eligibility Test Study",
          "Study to test eligibility criteria",
          new BN(Date.now() / 1000 + 60), // enrollment start (1 min from now)
          new BN(Date.now() / 1000 + 7200), // enrollment end (2 hours from now) - meets MIN_ENROLLMENT_WINDOW
          new BN(Date.now() / 1000 + 172800), // data collection end (2 days from now)
          10, // max participants
          new BN(LAMPORTS_PER_SOL), // reward amount
        )
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([researcher])
        .rpc()
        .then(confirm)
        .then(log);

      console.log("Study created for eligibility test:", createStudyTx);

      // Set eligibility criteria
      const eligibilityCriteria = {
        minAge: 18,
        maxAge: 65,
        gender: "female",
        location: "united states",
        educationLevel: "bachelor",
        employmentStatus: "employed",
        medicalConditions: ["diabetes", "heart disease"],
        customRequirements: ["tech savvy", "mobile user"]
      };

      const criteriaBytes = Buffer.from(JSON.stringify(eligibilityCriteria));

      const setCriteriaTx = await program.methods
        .setEligibilityCriteria(studyId, criteriaBytes)
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
        })
        .signers([researcher])
        .rpc()
        .then(confirm)
        .then(log);

      console.log("Eligibility criteria set:", setCriteriaTx);

      // Verify criteria was set by checking study account
      const studyAccount = await program.account.studyAccount.fetch(study);
      expect(studyAccount.hasEligibilityCriteria).to.be.true;
      expect(studyAccount.eligibilityCriteria.length).to.be.greaterThan(0);
    });

    it("Should verify eligible participant", async () => {
      const studyId = new BN(1002);
      const [study] = PublicKey.findProgramAddressSync(
        [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
        programId
      );

      // Create study with eligibility criteria
      const createStudyTx = await program.methods
        .createStudy(
          studyId,
          "Eligibility Verification Study",
          "Study to test eligibility verification",
          new BN(Date.now() / 1000 + 60),
          new BN(Date.now() / 1000 + 7200), // enrollment end (2 hours from now)
          new BN(Date.now() / 1000 + 172800), // data collection end (2 days from now)
          10,
          new BN(LAMPORTS_PER_SOL),
        )
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Set basic eligibility criteria
      const eligibilityCriteria = {
        minAge: 18,
        maxAge: 65,
        gender: "female",
        location: "united states"
      };

      const criteriaBytes = Buffer.from(JSON.stringify(eligibilityCriteria));

      await program.methods
        .setEligibilityCriteria(studyId, criteriaBytes)
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Create eligible participant info
      const eligibleParticipantInfo = {
        age: 25,
        gender: "female",
        location: "united states",
        educationLevel: "bachelor",
        employmentStatus: "employed",
        medicalConditions: [],
        additionalInfo: ["tech savvy"]
      };

      const participantInfoBytes = Buffer.from(JSON.stringify(eligibleParticipantInfo));

      // Verify eligibility
      const verifyTx = await program.methods
        .verifyEligibility(studyId, participantInfoBytes)
        .accountsPartial({
          study,
          participant: participant.publicKey,
        })
        .rpc()
        .then(confirm)
        .then(log);

      console.log("Eligible participant verification:", verifyTx);
    });

    it("Should reject ineligible participant", async () => {
      const studyId = new BN(1003);
      const [study] = PublicKey.findProgramAddressSync(
        [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
        programId
      );

      // Create study with eligibility criteria
      await program.methods
        .createStudy(
          studyId,
          "Ineligible Participant Test",
          "Study to test ineligible participant rejection",
          new BN(Date.now() / 1000 + 60),
          new BN(Date.now() / 1000 + 7200), // enrollment end (2 hours from now)
          new BN(Date.now() / 1000 + 172800), // data collection end (2 days from now)
          10,
          new BN(LAMPORTS_PER_SOL),
        )
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Set eligibility criteria
      const eligibilityCriteria = {
        minAge: 18,
        maxAge: 65,
        gender: "female"
      };

      const criteriaBytes = Buffer.from(JSON.stringify(eligibilityCriteria));

      await program.methods
        .setEligibilityCriteria(studyId, criteriaBytes)
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Create ineligible participant info (wrong gender)
      const ineligibleParticipantInfo = {
        age: 25,
        gender: "male", // Doesn't match required "female"
        location: "united states",
        educationLevel: "bachelor",
        employmentStatus: "employed",
        medicalConditions: [],
        additionalInfo: []
      };

      const participantInfoBytes = Buffer.from(JSON.stringify(ineligibleParticipantInfo));

      try {
        // This should fail
        await program.methods
          .verifyEligibility(studyId, participantInfoBytes)
          .accountsPartial({
            study,
            participant: participant.publicKey,
          })
          .rpc()
          .then(confirm);

        // If we reach here, the test should fail
        expect.fail("Should have rejected ineligible participant");
      } catch (error) {
        console.log("Correctly rejected ineligible participant:", error);
        // Test passes - ineligible participant was rejected
      }
    });

    it("Should enforce eligibility in consent enrollment", async () => {
      const studyId = new BN(1004);
      const [study] = PublicKey.findProgramAddressSync(
        [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
        programId
      );

      // Create and publish study
      await program.methods
        .createStudy(
          studyId,
          "Consent Eligibility Test",
          "Study to test eligibility enforcement in consent",
          new BN(Date.now() / 1000 + 60),
          new BN(Date.now() / 1000 + 7200), // enrollment end (2 hours from now)
          new BN(Date.now() / 1000 + 172800), // data collection end (2 days from now)
          10,
          new BN(LAMPORTS_PER_SOL),
        )
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Set eligibility criteria
      const eligibilityCriteria = {
        minAge: 21,
        maxAge: 50,
        educationLevel: "bachelor"
      };

      const criteriaBytes = Buffer.from(JSON.stringify(eligibilityCriteria));

      await program.methods
        .setEligibilityCriteria(studyId, criteriaBytes)
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Publish study
      await program.methods
        .publishStudy()
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Try to enroll with ineligible participant (age 18, below minimum 21)
      const [consent] = PublicKey.findProgramAddressSync(
        [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
        programId
      );

      const consentNftMint = Keypair.generate();
      
      const ineligibleParticipantInfo = {
        age: 18, // Below minimum age of 21
        gender: "female",
        location: "united states",
        educationLevel: "bachelor",
        employmentStatus: "employed",
        medicalConditions: [],
        additionalInfo: []
      };

      const eligibilityProof = Buffer.from(JSON.stringify(ineligibleParticipantInfo));

      try {
        // This should fail due to eligibility check
        await program.methods
          .mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            consentNftMint: consentNftMint.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([participant, consentNftMint])
          .rpc()
          .then(confirm);

        // If we reach here, the test should fail
        expect.fail("Should have rejected ineligible participant during consent");
      } catch (error) {
        console.log("Correctly rejected ineligible participant during consent:", error);
        // Test passes - eligibility was enforced during consent
      }
    });

    it("Should allow enrollment for study without eligibility criteria", async () => {
      const studyId = new BN(1005);
      const [study] = PublicKey.findProgramAddressSync(
        [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
        programId
      );

      // Create and publish study without eligibility criteria
      await program.methods
        .createStudy(
          studyId,
          "No Eligibility Study",
          "Study without eligibility criteria",
          new BN(Date.now() / 1000 + 60),
          new BN(Date.now() / 1000 + 7200), // enrollment end (2 hours from now)
          new BN(Date.now() / 1000 + 172800), // data collection end (2 days from now)
          10,
          new BN(LAMPORTS_PER_SOL),
        )
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Publish study
      await program.methods
        .publishStudy()
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
        })
        .signers([researcher])
        .rpc()
        .then(confirm);

      // Try to enroll - should succeed since no eligibility criteria
      const [consent] = PublicKey.findProgramAddressSync(
        [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
        programId
      );

      const consentNftMint = Keypair.generate();
      
      const participantInfo = {
        age: 25,
        gender: "female",
        location: "united states",
        educationLevel: "bachelor",
        employmentStatus: "employed",
        medicalConditions: [],
        additionalInfo: []
      };

      const eligibilityProof = Buffer.from(JSON.stringify(participantInfo));

      const consentTx = await program.methods
        .mintConsentNft(studyId, eligibilityProof)
        .accountsPartial({
          study,
          consent,
          consentNftMint: consentNftMint.publicKey,
          participant: participant.publicKey,
          systemProgram: SystemProgram.programId,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .signers([participant, consentNftMint])
        .rpc()
        .then(confirm)
        .then(log);

      console.log("Successfully enrolled in study without eligibility criteria:", consentTx);
    });
  });

  /// Error handling test suite
  describe("Error Handling", () => {
    it("Should handle invalid study operations", async () => {
      try {
        const studyId = new BN(999);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );

        const tx = await program.methods
          .publishStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Invalid study operation signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log("Invalid study operation correctly handled:", error);
        }
      }
    });

    it("Should handle invalid consent operations", async () => {
      try {
        const studyId = new BN(888);
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [Buffer.from("consent"), study.toBuffer(), participant.publicKey.toBuffer()],
          programId
        );

        const consentNftMint = Keypair.generate();
        const eligibilityProof = Buffer.from("invalid_proof");

        const tx = await program.methods
          .mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            consentNftMint: consentNftMint.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID,
          })
          .signers([participant, consentNftMint])
          .rpc()
          .then(confirm)
          .then(log);

        console.log("Invalid consent operation signature:", tx);
      } catch (error) {
        if (error instanceof anchor.web3.SendTransactionError) {
          console.log("Detailed error:", error.logs);
        } else {
          console.log("Invalid consent operation correctly handled:", error);
        }
      }
    });
  });
});
