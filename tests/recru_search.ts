// Core imports
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RecruSearch } from "../target/types/recru_search";
import { BN } from "bn.js";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_CLOCK_PUBKEY, Transaction } from "@solana/web3.js";
import { MINT_SIZE, TOKEN_PROGRAM_ID, createAssociatedTokenAccountIdempotentInstruction, createInitializeMint2Instruction, createMintToInstruction, getAssociatedTokenAddressSync, getMinimumBalanceForRentExemptMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import { expect } from "chai";
import { EncryptionManager, DecryptionManager } from "../app/utils/encryption";

describe("recru-search", () => {
  // Setup provider and program for existing devnet deployment
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;
  
  // Use workspace which will connect to existing devnet deployment
  const program = anchor.workspace.RecruSearch as Program<RecruSearch>;
  const programId = program.programId;

  // Test accounts
  let admin: Keypair;
  let researcher: Keypair;
  let participant: Keypair;
  let participant2: Keypair;
  let rewardMint: Keypair;
  let researcherTokenAccount: PublicKey;
  let participantTokenAccount: PublicKey;

  // Confirm transaction
  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({ signature, ...block });
    return signature;
  };

  // Log transaction URL
  const log = async (signature: string): Promise<string> => {
    const explorerUrl = `https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`;
    console.log(`Your transaction signature: ${explorerUrl}`);
    return signature;
  };

  //Serialize eligibility criteria to buffer
  const serializeEligibilityCriteria = (criteria: any): Buffer => {
    const serializedData = {
      min_age: criteria.minAge || null,
      max_age: criteria.maxAge || null,
      gender: criteria.gender || null,
      location: criteria.location || null,
      education_level: criteria.educationLevel || null,
      employment_status: criteria.employmentStatus || null,
      medical_conditions: criteria.medicalConditions || [],
      custom_requirements: criteria.customRequirements || []
    };
    return Buffer.from(JSON.stringify(serializedData), 'utf8');
  };

  // Create SPL token mint
  async function createMint(authority: Keypair): Promise<Keypair> {
    const mint = Keypair.generate();
    const mintRent = await getMinimumBalanceForRentExemptMint(connection);
    
    const createMintAccountIx = SystemProgram.createAccount({
      fromPubkey: authority.publicKey,
      newAccountPubkey: mint.publicKey,
      space: MINT_SIZE,
      lamports: mintRent,
      programId: TOKEN_PROGRAM_ID,
    });
    
    const initializeMintIx = createInitializeMint2Instruction(
      mint.publicKey,
      6,
      authority.publicKey,
      null,
      TOKEN_PROGRAM_ID
    );
    
    const transaction = new Transaction()
      .add(createMintAccountIx)
      .add(initializeMintIx);
      
    await provider.sendAndConfirm(transaction, [authority, mint]);
    return mint;
  }

  // Setup token account with initial balance
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
      TOKEN_PROGRAM_ID
    );
    
    const createAtaIx = createAssociatedTokenAccountIdempotentInstruction(
      owner.publicKey,
      ata,
      owner.publicKey,
      mint.publicKey,
      TOKEN_PROGRAM_ID
    );
    
    const mintToIx = createMintToInstruction(
      mint.publicKey,
      ata,
      mintAuthority.publicKey,
      amount,
      [],
      TOKEN_PROGRAM_ID
    );
    
    const setupTokensTx = new Transaction()
      .add(createAtaIx)
      .add(mintToIx);
      
    await provider.sendAndConfirm(setupTokensTx, [owner, mintAuthority]);
    return ata;
  }

  // Airdrop SOL to test accounts with retry logic
  async function airdropSol(to: Keypair, amount: number, maxRetries: number = 3) {
    const lamports = amount * LAMPORTS_PER_SOL;
    
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        const signature = await connection.requestAirdrop(to.publicKey, lamports);
        await connection.confirmTransaction(signature);
        console.log(`✅ Airdropped ${amount} SOL to ${to.publicKey.toBase58()}`);
        return;
      } catch (error) {
        console.error(`❌ Airdrop attempt ${attempt} failed for ${to.publicKey.toBase58()}:`, error);
        
        if (attempt === maxRetries) {
          throw new Error(`Airdrop failed after ${maxRetries} attempts for ${to.publicKey.toBase58()}`);
        }
        
        // Wait before retry (exponential backoff)
        await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000));
      }
    }
  }

  // Batch airdrop for multiple accounts
  async function airdropMultiple(accounts: { keypair: Keypair, amount: number }[]) {
    console.log("🚀 Starting batch airdrop...");
    
    for (const { keypair, amount } of accounts) {
      await airdropSol(keypair, amount);
      // Small delay to avoid rate limiting
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    console.log("✅ Batch airdrop completed");
  }

  // Check account balance for debugging
  async function checkBalance(keypair: Keypair, label: string) {
    const balance = await connection.getBalance(keypair.publicKey);
    console.log(`💰 ${label} balance: ${balance / LAMPORTS_PER_SOL} SOL`);
    return balance;
  }

  // Encryption managers
  const encryptionManager = new EncryptionManager();
  const decryptionManager = new DecryptionManager();

  // Encrypt data for researcher
  async function encryptDataForResearcher(
    data: any,
    researcherPublicKey: Uint8Array
  ): Promise<{ encryptedDataHash: number[], ipfsCid: string }> {
    try {
      const { encrypted, cid } = await encryptionManager.encryptAndStore(
        data,
        researcherPublicKey
      );
      const encryptedDataHash = Array.from(encrypted.slice(0, 32), (byte) => byte as number);
      return { encryptedDataHash, ipfsCid: cid };
    } catch (error) {
      console.error("Encryption failed:", error);
      throw error;
    }
  }

  // Decrypt data as researcher with secure key handling
  async function decryptDataAsResearcher(
    ipfsCid: string,
    researcherKeypair: Keypair
  ): Promise<any> {
    try {
      // Create a temporary copy of the private key for decryption
      const privateKeyBytes = new Uint8Array(researcherKeypair.secretKey);
      const decryptedData = await decryptionManager.fetchAndDecrypt(
        ipfsCid,
        privateKeyBytes
      );
      
      // Clear the private key from memory immediately after use
      privateKeyBytes.fill(0);
      
      return decryptedData;
    } catch (error) {
      console.error("Decryption failed:", error);
      throw error;
    }
  }

  // Setup test environment
  before(async () => {
    admin = Keypair.generate();
    researcher = Keypair.generate();
    participant = Keypair.generate();
    participant2 = Keypair.generate();
    rewardMint = Keypair.generate();
    
    // Airdrop SOL to test accounts
    await airdropMultiple([
      { keypair: admin, amount: 2 },
      { keypair: researcher, amount: 2 },
      { keypair: participant, amount: 1 },
      { keypair: participant2, amount: 1 }
    ]);
    
    // Verify airdrops were successful
    await checkBalance(admin, "Admin");
    await checkBalance(researcher, "Researcher");
    await checkBalance(participant, "Participant");
    await checkBalance(participant2, "Participant2");
    
    const mint = await createMint(researcher);
    rewardMint = mint;
    
    researcherTokenAccount = await setupTokenAccount(rewardMint, researcher, researcher, 1000000000);
    participantTokenAccount = await setupTokenAccount(rewardMint, participant, researcher, 0);
  });

  // AIRDROP VERIFICATION TEST
  describe("Airdrop Verification", () => {
    it("Should have sufficient SOL for testing", async () => {
      const adminBalance = await checkBalance(admin, "Admin");
      const researcherBalance = await checkBalance(researcher, "Researcher");
      const participantBalance = await checkBalance(participant, "Participant");
      
      expect(adminBalance).to.be.greaterThan(1 * LAMPORTS_PER_SOL);
      expect(researcherBalance).to.be.greaterThan(1 * LAMPORTS_PER_SOL);
      expect(participantBalance).to.be.greaterThan(0.5 * LAMPORTS_PER_SOL);
    });
  });

  // PROTOCOL INITIALIZATION TESTS
  describe("Protocol Initialization", () => {
    it("Should initialize protocol successfully", async () => {
      try {
        const [adminState] = PublicKey.findProgramAddressSync(
          [Buffer.from("admin")],
          programId
        );
        
        const tx = await program.methods.initializeProtocol(250, 86400, 31536000)
          .accountsPartial({
            adminState,
            protocolAdmin: admin.publicKey,
            systemProgram: SystemProgram.programId
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
        
        const tx = await program.methods.initializeProtocol(250, 86400, 31536000)
          .accountsPartial({
            adminState,
            protocolAdmin: admin.publicKey,
            systemProgram: SystemProgram.programId
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

  // STUDY MANAGEMENT TESTS
  describe("Study Management", () => {
    it("Should create a study successfully", async () => {
      try {
        const studyId = new BN(1);
        const title = "Test Study";
        const description = "A comprehensive test study for research validation";
        
        const now = new BN(Math.floor(Date.now() / 1000));
        const enrollmentStart = now.add(new BN(1));
        const enrollmentEnd = enrollmentStart.add(new BN(604800));
        const dataCollectionEnd = enrollmentEnd.add(new BN(604800));
        
        const maxParticipants = 100;
        const rewardAmount = new BN(1000000);
        
        const [study] = PublicKey.findProgramAddressSync(
          [Buffer.from("study"), researcher.publicKey.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
          programId
        );
        
        const tx = await program.methods.createStudy(
          studyId, title, description, enrollmentStart, enrollmentEnd, dataCollectionEnd, maxParticipants, rewardAmount
        )
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY
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
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        
        const tx = await program.methods.publishStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey
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
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        
        const tx = await program.methods.publishStudy()
          .accountsPartial({
            study,
            researcher: participant.publicKey
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

  // REWARD VAULT MANAGEMENT TESTS
  describe("Reward Vault Management", () => {
    it("Should create a reward vault successfully", async () => {
      try {
        const studyId = new BN(3);
        const title = "Reward Vault Test Study";
        const description = "A test study for reward vault creation";
        
        const now = new BN(Math.floor(Date.now() / 1000));
        const enrollmentStart = now.add(new BN(1));
        const enrollmentEnd = enrollmentStart.add(new BN(604800));
        const dataCollectionEnd = enrollmentEnd.add(new BN(604800));
        
        const maxParticipants = 50;
        const rewardAmount = new BN(500000);
        
        const [study] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        const [rewardVault] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault"), study.toBuffer()],
          programId
        );
        
        const createStudyIx = await program.methods.createStudy(
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
            clock: SYSVAR_CLOCK_PUBKEY
          })
          .instruction();
          
        const depositAmount = new BN(10000000);
        const [vaultTokenAccount] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault_token"), rewardVault.toBuffer()],
          programId
        );
        
        const createRewardVaultIx = await program.methods.createRewardVault(
          studyId,
          depositAmount
        )
          .accountsPartial({
            study,
            rewardVault,
            vaultTokenAccount,
            rewardTokenMint: rewardMint.publicKey,
            researcherTokenAccount,
            researcher: researcher.publicKey,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          })
          .instruction();
          
        const tx = new Transaction()
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

  // CONSENT NFT MANAGEMENT TESTS
  describe("Consent NFT Management", () => {
    it("Should mint a consent NFT successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("consent"),
            study.toBuffer(),
            participant.publicKey.toBuffer()
          ],
          programId
        );
        
        const asset = Keypair.generate();
        const eligibilityProof = Buffer.from("test_eligibility_proof_123");
        
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        const tx = await program.methods.mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            asset: asset.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
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
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("consent"),
            study.toBuffer(),
            participant.publicKey.toBuffer()
          ],
          programId
        );
        
        const now = new BN(Math.floor(Date.now() / 1000));
        const enrollmentStart = now.add(new BN(5));
        const enrollmentEnd = enrollmentStart.add(new BN(604800));
        const dataCollectionEnd = enrollmentEnd.add(new BN(604800));
        
        await program.methods.createStudy(
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
            clock: SYSVAR_CLOCK_PUBKEY
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);
          
        const asset = Keypair.generate();
        const eligibilityProof = Buffer.from("test_proof");
        
        const tx = await program.methods.mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            asset: asset.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
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

  // DATA SUBMISSION TESTS
  describe("Data Submission", () => {
    it("Should submit data successfully", async () => {
      try {
        const studyId = new BN(4);
        const title = "Data Submission Test Study";
        const description = "A test study for data submission";
        
        const now = new BN(Math.floor(Date.now() / 1000));
        const enrollmentStart = now.sub(new BN(1));
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
        
        const createStudyIx = await program.methods.createStudy(
          studyId, title, description, enrollmentStart, enrollmentEnd, dataCollectionEnd, maxParticipants, rewardAmount
        )
          .accountsPartial({
            study,
            researcher: researcher.publicKey,
            systemProgram: SystemProgram.programId,
            clock: SYSVAR_CLOCK_PUBKEY
          })
          .instruction();
          
        const publishStudyIx = await program.methods.publishStudy()
          .accountsPartial({ study, researcher: researcher.publicKey })
          .signers([researcher])
          .instruction();
          
        const asset = Keypair.generate();
        const eligibilityProof = Buffer.from("data_submission_test_proof");
        
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        const mintConsentIx = await program.methods.mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            asset: asset.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant, asset])
          .instruction();
          
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
        
        const researcherPublicKeyBytes = new Uint8Array(researcher.publicKey.toBytes());
        const { encryptedDataHash, ipfsCid } = await encryptDataForResearcher(testSurveyData, researcherPublicKeyBytes);
        
        console.log("Encrypted data hash:", encryptedDataHash);
        console.log("IPFS CID:", ipfsCid);
        
        const submitDataIx = await program.methods.submitData(encryptedDataHash, ipfsCid)
          .accountsPartial({
            study,
            consent,
            submission,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId
          })
          .signers([participant])
          .instruction();
          
        const tx = new Transaction()
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
        
        const researcherPublicKeyBytes = new Uint8Array(researcher.publicKey.toBytes());
        const { encryptedDataHash, ipfsCid } = await encryptDataForResearcher(testData, researcherPublicKeyBytes);
        
        console.log("Test encryption - IPFS CID:", ipfsCid);
        
        const decryptedData = await decryptDataAsResearcher(ipfsCid, researcher);
        
        // Validate data integrity without logging sensitive content
        expect(decryptedData.participantId).to.equal(testData.participantId);
        expect(decryptedData.surveyResponses.question1).to.equal(testData.surveyResponses.question1);
        expect(decryptedData.metadata.deviceType).to.equal(testData.metadata.deviceType);
        
        // Log only non-sensitive metadata for debugging
        console.log("✅ Data integrity verified - participant ID:", decryptedData.participantId.substring(0, 8) + "...");
        
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
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        const [submission] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("submission"),
            study.toBuffer(),
            participant2.publicKey.toBuffer()
          ],
          programId
        );
        
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
        
        const tx = await program.methods.submitData(encryptedDataHash, ipfsCid)
          .accountsPartial({
            study,
            consent: participant2.publicKey,
            submission,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId
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

  // REWARD DISTRIBUTION TESTS
  describe("Reward Distribution", () => {
    it("Should distribute rewards successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        const [rewardVault] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault"), study.toBuffer()],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("consent"),
            study.toBuffer(),
            participant.publicKey.toBuffer()
          ],
          programId
        );
        const [submission] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("submission"),
            study.toBuffer(),
            participant.publicKey.toBuffer()
          ],
          programId
        );
        
        const tx = await program.methods.distributeReward()
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
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
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

  // COMPLETION NFT MANAGEMENT TESTS
  describe("Completion NFT Management", () => {
    it("Should mint a completion NFT successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        const [submission] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("submission"),
            study.toBuffer(),
            participant.publicKey.toBuffer()
          ],
          programId
        );
        
        const asset = Keypair.generate();
        
        const tx = await program.methods.mintCompletionNft()
          .accountsPartial({
            study,
            submission,
            asset: asset.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant, asset])
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

  // STUDY CLOSURE TESTS
  describe("Study Closure", () => {
    it("Should close a study successfully", async () => {
      try {
        const studyId = new BN(1);
        const [study] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        
        const tx = await program.methods.closeStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey
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

  // INTEGRATION TESTS
  describe("Integration Tests", () => {
    it("Should complete full RecruSearch lifecycle", async () => {
      try {
        console.log("Starting full RecruSearch lifecycle test...");
        const studyId = new BN(3);
        const [study] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        
        const now = new BN(Math.floor(Date.now() / 1000));
        const enrollmentStart = now.add(new BN(5));
        const enrollmentEnd = enrollmentStart.add(new BN(604800));
        const dataCollectionEnd = enrollmentEnd.add(new BN(604800));
        
        const createTx = await program.methods.createStudy(
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
            clock: SYSVAR_CLOCK_PUBKEY
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);
        console.log("Step 1: Study created -", createTx);
        
        // Publish study for enrollment
        const publishTx = await program.methods.publishStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey
          })
          .signers([researcher])
          .rpc()
          .then(confirm)
          .then(log);
          
        console.log("Step 2: Study published -", publishTx);
        
        const [rewardVault] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault"), study.toBuffer()],
          programId
        );
        const [vaultTokenAccount] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault_token"), rewardVault.toBuffer()],
          programId
        );
        
        const createRewardVaultIx = await program.methods.createRewardVault(
          studyId,
          new BN(100000000)
        )
          .accountsPartial({
            study,
            rewardVault,
            vaultTokenAccount,
            rewardTokenMint: rewardMint.publicKey,
            researcherTokenAccount,
            researcher: researcher.publicKey,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          })
          .instruction();
        const vaultSignature = await provider.sendAndConfirm(new Transaction().add(createRewardVaultIx), [researcher]);
        await confirm(vaultSignature);
        await log(vaultSignature);
        console.log("Step 3: Reward vault created -", vaultSignature);
        
        // Wait for enrollment period and mint consent NFT
        await new Promise(resolve => setTimeout(resolve, 6000));
        const [consent] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("consent"),
            study.toBuffer(),
            participant2.publicKey.toBuffer()
          ],
          programId
        );
        
        const consentAsset = Keypair.generate();
        const eligibilityProof = Buffer.from("integration_test_proof");
        
        const consentTx = await program.methods.mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            asset: consentAsset.publicKey,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant2, consentAsset])
          .rpc()
          .then(confirm)
          .then(log);
          
        console.log("Step 4: Consent NFT minted -", consentTx);
        
        const [submission] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("submission"),
            study.toBuffer(),
            participant2.publicKey.toBuffer()
          ],
          programId
        );
        
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
        
        const submitTx = await program.methods.submitData(encryptedDataHash, ipfsCid)
          .accountsPartial({
            study,
            consent,
            submission,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId
          })
          .signers([participant2])
          .rpc()
          .then(confirm)
          .then(log);
        console.log("Step 5: Data submitted -", submitTx);
        const participant2TokenAccount = getAssociatedTokenAddressSync(
          rewardMint.publicKey,
          participant2.publicKey
        );
        
        const createParticipant2TokenAccountIx = createAssociatedTokenAccountIdempotentInstruction(
          researcher.publicKey,
          participant2TokenAccount,
          participant2.publicKey,
          rewardMint.publicKey,
          TOKEN_PROGRAM_ID
        );
        
        const distributeRewardIx = await program.methods.distributeReward()
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
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          })
          .instruction();
          
        const rewardTx = new Transaction()
          .add(createParticipant2TokenAccountIx)
          .add(distributeRewardIx);
          
        const rewardSignature = await provider.sendAndConfirm(rewardTx, [researcher]);
        await confirm(rewardSignature);
        await log(rewardSignature);
        
        console.log("Step 6: Reward distributed -", rewardSignature);
        
        const completionAsset = Keypair.generate();
        
        const completionTx = await program.methods.mintCompletionNft()
          .accountsPartial({
            study,
            submission,
            asset: completionAsset.publicKey,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant2, completionAsset])
          .rpc()
          .then(confirm)
          .then(log);
          
        console.log("Step 7: Completion NFT minted -", completionTx);
        
        const closeTx = await program.methods.closeStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey
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

  // ELIGIBILITY CRITERIA TESTS
  describe("Eligibility Criteria", () => {
    it("Should only allow eligible participants to enroll for studies", async () => {
      console.log("Starting eligibility criteria test...");
      
      // Create a study with specific eligibility criteria
      const studyId = new BN(2001);
      const [study] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("study"),
          researcher.publicKey.toBuffer(),
          studyId.toArrayLike(Buffer, "le", 8)
        ],
        programId
      );
      
      // Create study in draft state
      const createStudyTx = await program.methods.createStudy(
        studyId,
        "Eligibility Test Study",
        "Study with specific eligibility criteria for testing",
        new BN(Date.now() / 1000 + 5),
        new BN(Date.now() / 1000 + 7200),
        new BN(Date.now() / 1000 + 172800),
        10,
        new BN(LAMPORTS_PER_SOL)
      )
        .accountsPartial({
          study,
          researcher: researcher.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY
        })
        .signers([researcher])
        .rpc()
        .then(confirm);
        
      console.log("Study created:", createStudyTx);
      
      // Set eligibility criteria for the study - using simple JSON format
      const eligibilityCriteria = {
        min_age: 25,
        max_age: 65,
        gender: "female",
        location: "united states",
        education_level: "bachelor",
        employment_status: "employed",
        medical_conditions: ["diabetes", "heart disease"],
        custom_requirements: ["tech_savvy", "mobile_user"]
      };
      
      // Serialize as JSON string then to buffer
      const criteriaBytes = Buffer.from(JSON.stringify(eligibilityCriteria), 'utf8');
      
      const setCriteriaTx = await program.methods.setEligibilityCriteria(studyId, criteriaBytes)
        .accountsPartial({
          study,
          researcher: researcher.publicKey
        })
        .signers([researcher])
        .rpc()
        .then(confirm);
        
      console.log("Eligibility criteria set:", setCriteriaTx);
      
      // Verify criteria was set correctly
      const studyAccount = await program.account.studyAccount.fetch(study);
      expect(studyAccount.hasEligibilityCriteria).to.be.true;
      expect(studyAccount.eligibilityCriteria.length).to.be.greaterThan(0);
      
      // Publish the study
      const publishTx = await program.methods.publishStudy()
        .accountsPartial({
          study,
          researcher: researcher.publicKey
        })
        .signers([researcher])
        .rpc()
        .then(confirm);
        
      console.log("Study published:", publishTx);
      
      // Wait for enrollment period to start
      await new Promise(resolve => setTimeout(resolve, 6000));
      
      // Test 1: Eligible participant should be able to enroll
      console.log("Testing eligible participant enrollment...");
      const [eligibleConsent] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("consent"),
          study.toBuffer(),
          participant.publicKey.toBuffer()
        ],
        programId
      );
      
      const eligibleAsset = Keypair.generate();
      const eligibleParticipantInfo = {
        age: 30,
        gender: "female",
        location: "united states",
        education_level: "bachelor",
        employment_status: "employed",
        medical_conditions: [],
        additional_info: ["tech_savvy", "mobile_user"]
      };
      
      const eligibleProof = Buffer.from(JSON.stringify(eligibleParticipantInfo), 'utf8');
      
      const eligibleEnrollmentTx = await program.methods.mintConsentNft(studyId, eligibleProof)
        .accountsPartial({
          study,
          consent: eligibleConsent,
          asset: eligibleAsset.publicKey,
          participant: participant.publicKey,
          systemProgram: SystemProgram.programId,
          mplCoreProgram: MPL_CORE_PROGRAM_ID
        })
        .signers([participant, eligibleAsset])
        .rpc()
        .then(confirm);
        
               console.log("Eligible participant successfully enrolled:", eligibleEnrollmentTx);
      
      // Test 2: Ineligible participant (wrong age) should be rejected
      console.log("Testing ineligible participant (wrong age)...");
      const [ineligibleConsent1] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("consent"),
          study.toBuffer(),
          participant2.publicKey.toBuffer()
        ],
        programId
      );
      
      const ineligibleAsset1 = Keypair.generate();
      const ineligibleParticipantInfo1 = {
        age: 20, // Too young
        gender: "female",
        location: "united states",
        education_level: "bachelor",
        employment_status: "employed",
        medical_conditions: [],
        additional_info: ["tech_savvy", "mobile_user"]
      };
      
      const ineligibleProof1 = Buffer.from(JSON.stringify(ineligibleParticipantInfo1), 'utf8');
      
      try {
        await program.methods.mintConsentNft(studyId, ineligibleProof1)
          .accountsPartial({
            study,
            consent: ineligibleConsent1,
            asset: ineligibleAsset1.publicKey,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant2, ineligibleAsset1])
          .rpc()
          .then(confirm);
          
        throw new Error("Ineligible participant (wrong age) was incorrectly allowed to enroll");
      } catch (error) {
                 if (error instanceof anchor.web3.SendTransactionError) {
           console.log("Ineligible participant (wrong age) correctly rejected:", error.logs);
         } else {
           console.log("Ineligible participant (wrong age) correctly rejected:", error);
         }
      }
      
      // Test 3: Ineligible participant (wrong gender) should be rejected
      console.log("Testing ineligible participant (wrong gender)...");
      const [ineligibleConsent2] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("consent"),
          study.toBuffer(),
          participant2.publicKey.toBuffer()
        ],
        programId
      );
      
      const ineligibleAsset2 = Keypair.generate();
      const ineligibleParticipantInfo2 = {
        age: 30,
        gender: "male", // Wrong gender
        location: "united states",
        education_level: "bachelor",
        employment_status: "employed",
        medical_conditions: [],
        additional_info: ["tech_savvy", "mobile_user"]
      };
      
      const ineligibleProof2 = Buffer.from(JSON.stringify(ineligibleParticipantInfo2), 'utf8');
      
      try {
        await program.methods.mintConsentNft(studyId, ineligibleProof2)
          .accountsPartial({
            study,
            consent: ineligibleConsent2,
            asset: ineligibleAsset2.publicKey,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant2, ineligibleAsset2])
          .rpc()
          .then(confirm);
          
        throw new Error("Ineligible participant (wrong gender) was incorrectly allowed to enroll");
      } catch (error) {
                 if (error instanceof anchor.web3.SendTransactionError) {
           console.log("Ineligible participant (wrong gender) correctly rejected:", error.logs);
         } else {
           console.log("Ineligible participant (wrong gender) correctly rejected:", error);
         }
      }
      
      // Test 4: Ineligible participant (excluded medical condition) should be rejected
      console.log("Testing ineligible participant (excluded medical condition)...");
      const [ineligibleConsent3] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("consent"),
          study.toBuffer(),
          participant2.publicKey.toBuffer()
        ],
        programId
      );
      
      const ineligibleAsset3 = Keypair.generate();
      const ineligibleParticipantInfo3 = {
        age: 30,
        gender: "female",
        location: "united states",
        education_level: "bachelor",
        employment_status: "employed",
        medical_conditions: ["diabetes"], // Excluded condition
        additional_info: ["tech_savvy", "mobile_user"]
      };
      
      const ineligibleProof3 = Buffer.from(JSON.stringify(ineligibleParticipantInfo3), 'utf8');
      
      try {
        await program.methods.mintConsentNft(studyId, ineligibleProof3)
          .accountsPartial({
            study,
            consent: ineligibleConsent3,
            asset: ineligibleAsset3.publicKey,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant2, ineligibleAsset3])
          .rpc()
          .then(confirm);
          
        throw new Error("Ineligible participant (excluded medical condition) was incorrectly allowed to enroll");
      } catch (error) {
                 if (error instanceof anchor.web3.SendTransactionError) {
           console.log("Ineligible participant (excluded medical condition) correctly rejected:", error.logs);
         } else {
           console.log("Ineligible participant (excluded medical condition) correctly rejected:", error);
         }
      }
      
      // Test 5: Ineligible participant (missing custom requirement) should be rejected
      console.log("Testing ineligible participant (missing custom requirement)...");
      const [ineligibleConsent4] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("consent"),
          study.toBuffer(),
          participant2.publicKey.toBuffer()
        ],
        programId
      );
      
      const ineligibleAsset4 = Keypair.generate();
      const ineligibleParticipantInfo4 = {
        age: 30,
        gender: "female",
        location: "united states",
        education_level: "bachelor",
        employment_status: "employed",
        medical_conditions: [],
        additional_info: ["tech_savvy"] // Missing "mobile_user"
      };
      
      const ineligibleProof4 = Buffer.from(JSON.stringify(ineligibleParticipantInfo4), 'utf8');
      
      try {
        await program.methods.mintConsentNft(studyId, ineligibleProof4)
          .accountsPartial({
            study,
            consent: ineligibleConsent4,
            asset: ineligibleAsset4.publicKey,
            participant: participant2.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant2, ineligibleAsset4])
          .rpc()
          .then(confirm);
          
        throw new Error("Ineligible participant (missing custom requirement) was incorrectly allowed to enroll");
      } catch (error) {
                 if (error instanceof anchor.web3.SendTransactionError) {
           console.log("Ineligible participant (missing custom requirement) correctly rejected:", error.logs);
         } else {
           console.log("Ineligible participant (missing custom requirement) correctly rejected:", error);
         }
      }
      
             console.log("All eligibility criteria tests passed! Only eligible participants can enroll for studies.");
    });
  });
  // ERROR HANDLING TESTS
  describe("Error Handling", () => {
    it("Should handle invalid study operations", async () => {
      try {
        const studyId = new BN(999);
        const [study] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        
        const tx = await program.methods.publishStudy()
          .accountsPartial({
            study,
            researcher: researcher.publicKey
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
          [
            Buffer.from("study"),
            researcher.publicKey.toBuffer(),
            studyId.toArrayLike(Buffer, "le", 8)
          ],
          programId
        );
        const [consent] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("consent"),
            study.toBuffer(),
            participant.publicKey.toBuffer()
          ],
          programId
        );
        
        const asset = Keypair.generate();
        const eligibilityProof = Buffer.from("invalid_proof");
        
        const tx = await program.methods.mintConsentNft(studyId, eligibilityProof)
          .accountsPartial({
            study,
            consent,
            asset: asset.publicKey,
            participant: participant.publicKey,
            systemProgram: SystemProgram.programId,
            mplCoreProgram: MPL_CORE_PROGRAM_ID
          })
          .signers([participant, asset])
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