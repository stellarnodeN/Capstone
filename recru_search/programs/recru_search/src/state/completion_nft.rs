use anchor_lang::prelude::*;

// Account that represents a participant's completion certificate for a research study
// This is minted as an NFT when the participant completes the study and receives rewards
#[account]
#[derive(InitSpace)]
pub struct CompletionNFTAccount {
    pub study_id: u64,                    // Which study this certificate is for
    pub participant: Pubkey,              // Wallet address of the participant
    pub submission_account: Pubkey,       // Reference to the data submission account
    pub completion_timestamp: i64,        // When the participant completed the study (Unix timestamp)
    pub reward_amount: u64,               // Amount of reward the participant received
    
    // Enhanced metadata fields for Metaplex NFT compatibility
    #[max_len(100)]
    pub study_title: String,              // Study name for certificate display
    
    #[max_len(50)]
    pub study_type: String,               // Research category
    
    #[max_len(20)]
    pub study_duration_days: String,      // Study duration for display on certificate
    
    #[max_len(200)]
    pub metadata_uri: String,             // Complete metadata JSON URI (IPFS/Arweave)
    
    #[max_len(200)]
    pub image_uri: String,                // NFT certificate image URI
    
    pub bump: u8,                         // PDA bump seed for account creation
} 