import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RecruSearch } from "../target/types/recru_search";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { BN } from "bn.js";
import * as borsh from '@project-serum/borsh';

const { programId } = anchor.workspace.RecruSearch as Program<RecruSearch>;

// Generate and airdrop signer
export async function generateSigner(provider: anchor.AnchorProvider, amount: number = 2): Promise<Keypair> {
    const keypair = Keypair.generate();
    const signature = await provider.connection.requestAirdrop(keypair.publicKey, amount * LAMPORTS_PER_SOL);
    const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({ blockhash, lastValidBlockHeight, signature });
    return keypair;
}

// PDA helpers
export function getAdminPDA(programId: PublicKey): PublicKey {
    const [adminPDA] = PublicKey.findProgramAddressSync([Buffer.from("admin")], programId);
    return adminPDA;
}

export function getStudyPDA(programId: PublicKey, researcher: PublicKey, studyId: InstanceType<typeof BN>): PublicKey {
    const [studyPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("study"), researcher.toBuffer(), studyId.toArrayLike(Buffer, "le", 8)],
        programId
    );
    return studyPDA;
}

export function getConsentPDA(programId: PublicKey, study: PublicKey, participant: PublicKey): PublicKey {
    const [consentPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("consent"), study.toBuffer(), participant.toBuffer()],
        programId
    );
    return consentPDA;
}

export function getSubmissionPDA(study: PublicKey, participant: PublicKey): PublicKey {
    const [submissionPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("submission"), study.toBuffer(), participant.toBuffer()],
        programId
    );
    return submissionPDA;
}

export function getCompletionPDA(study: PublicKey, participant: PublicKey): PublicKey {
    const [completionPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("completion"), study.toBuffer(), participant.toBuffer()],
        programId
    );
    return completionPDA;
}

export function getRewardVaultPDA(study: PublicKey): PublicKey {
    const [vaultPDA] = PublicKey.findProgramAddressSync([Buffer.from("vault"), study.toBuffer()], programId);
    return vaultPDA;
}

export function getSurveySchemaPDA(study: PublicKey): PublicKey {
    const [schemaPDA] = PublicKey.findProgramAddressSync([Buffer.from("survey"), study.toBuffer()], programId);
    return schemaPDA;
}

// Vault token account
export function getVaultTokenAccountPDA(rewardVault: PublicKey): PublicKey {
    const [vaultTokenAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault_token"), rewardVault.toBuffer()],
        programId
    );
    return vaultTokenAccount;
}

// Study creation
export function createStudyParams(studyId: InstanceType<typeof BN>, title: string, description: string, maxParticipants: number, rewardAmount: InstanceType<typeof BN>) {
    const now = new BN(Math.floor(Date.now() / 1000));
    const enrollmentStart = now.add(new BN(1));
    const enrollmentEnd = enrollmentStart.add(new BN(604800)); // 7 days
    const dataCollectionEnd = enrollmentEnd.add(new BN(604800)); // 7 days
    
    return {
        studyId,
        title,
        description,
        enrollmentStart,
        enrollmentEnd,
        dataCollectionEnd,
        maxParticipants,
        rewardAmount
    };
}

// Borsh schema for EligibilityInfo
const EligibilityInfoSchema = borsh.struct([
  borsh.option(borsh.u8(), 'min_age'),
  borsh.option(borsh.u8(), 'max_age'),
  borsh.option(borsh.str(), 'gender'),
  borsh.option(borsh.str(), 'location'),
]);

// Eligibility criteria
export function createEligibilityCriteria(options: {
    minAge?: number;
    maxAge?: number;
    gender?: string;
    location?: string;
}) {
    return {
        min_age: options.minAge || null,
        max_age: options.maxAge || null,
        gender: options.gender || null,
        location: options.location || null
    };
}

// Participant info
export function createParticipantInfo(options: {
    age: number;
    gender: string;
    location: string;
}) {
    return {
        min_age: options.age,        // Store actual age
        max_age: null,               // Not used
        gender: options.gender,
        location: options.location
    };
}

// Borsh serialization for eligibility criteria
export function serializeEligibilityCriteria(criteria: ReturnType<typeof createEligibilityCriteria>): Buffer {
    const buffer = Buffer.alloc(1000); // Allocate space
    const len = EligibilityInfoSchema.encode(criteria, buffer);
    return buffer.slice(0, len);
}

// Borsh serialization for participant info
export function serializeParticipantInfo(participantInfo: ReturnType<typeof createParticipantInfo>): Buffer {
    const buffer = Buffer.alloc(1000); // Allocate space
    const len = EligibilityInfoSchema.encode(participantInfo, buffer);
    return buffer.slice(0, len);
}

// Transaction confirmation
export async function confirmTransaction(connection: anchor.web3.Connection, signature: string): Promise<string> {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({ signature, ...block });
    return signature;
}

// Log transaction
export function logTransaction(signature: string, connection: anchor.web3.Connection): string {
    const explorerUrl = `https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`;
    console.log(`Transaction: ${explorerUrl}`);
    return signature;
}

// Mock MPL Core functions for localnet testing
export function mockMplCoreAsset(): PublicKey {
    // Generate a mock asset public key for testing
    return Keypair.generate().publicKey;
}

export function mockMplCoreMint(): PublicKey {
    // Generate a mock mint public key for testing
    return Keypair.generate().publicKey;
}

export function mockMplCoreMetadata(): PublicKey {
    // Generate a mock metadata public key for testing
    return Keypair.generate().publicKey;
}

export function mockMplCoreCollection(): PublicKey {
    // Generate a mock collection public key for testing
    return Keypair.generate().publicKey;
}

// Mock NFT creation function that simulates MPL Core behavior
export async function mockCreateNFT(
    program: any,
    accounts: any,
    signers: any[]
): Promise<string> {
    // Simulate successful NFT creation
    console.log("✓ Mock NFT created successfully (localnet simulation)");
    console.log("✓ This would create a real NFT on devnet/mainnet with MPL Core");
    
    // Return a mock transaction signature
    return "mock_nft_transaction_signature_" + Date.now();
}

// Mock NFT verification function
export function mockVerifyNFT(asset: PublicKey): boolean {
    // Simulate NFT verification
    console.log("✓ Mock NFT verification successful (localnet simulation)");
    return true;
} 