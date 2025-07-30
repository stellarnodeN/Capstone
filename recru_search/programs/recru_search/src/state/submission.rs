use anchor_lang::prelude::*;

// Account that tracks a participant's encrypted data submission for a study
// This stores metadata about the submission while keeping the actual data encrypted off-chain
#[account]
#[derive(InitSpace)]
pub struct SubmissionAccount {
    pub study_id: u64,                    // Which study this data belongs to
    pub participant: Pubkey,              // Wallet address of the participant who submitted data
    pub consent_nft: Pubkey,              // Reference to the participant's consent NFT
    pub encrypted_data_hash: [u8; 32],    // Hash of the encrypted data for verification
    
    #[max_len(100)]
    pub ipfs_cid: String,                 // IPFS content identifier where encrypted data is stored
    
    pub submission_timestamp: i64,        // When the data was submitted (Unix timestamp)
    pub reward_claimed: bool,             // Whether the participant has received their reward yet
    pub is_verified: bool,                // Whether the submission has been verified by the researcher
    pub completion_nft: Option<Pubkey>,   // Reference to completion certificate NFT (if reward claimed)
    pub bump: u8,                         // PDA bump seed for account creation
} 