use anchor_lang::prelude::*;

// Account that represents a participant's consent to join a research study
// This is minted as an NFT to provide verifiable proof of informed consent
#[account]
#[derive(InitSpace)]
pub struct ConsentNFTAccount {
    pub study_id: u64,                    // Which study this consent is for
    pub participant: Pubkey,              // Wallet address of the participant
    pub study_account: Pubkey,            // Reference to the main study account
    pub consent_timestamp: i64,           // When the participant gave consent (Unix timestamp)
    pub eligibility_proof_hash: [u8; 32], // Hash of zero-knowledge proof of eligibility (if required)
    pub consent_document_version: [u8; 32], // Hash version of the consent form they agreed to
    
    // Enhanced metadata fields for Metaplex NFT compatibility
    #[max_len(100)]
    pub study_title: String,              // Study name for NFT display
    
    #[max_len(50)]
    pub study_type: String,               // Research category (e.g., "Mental Health", "Clinical Trial")
    
    #[max_len(200)]
    pub metadata_uri: String,             // Complete metadata JSON URI (IPFS/Arweave)
    
    #[max_len(200)]
    pub image_uri: String,                // NFT image URI (IPFS/Arweave)
    
    #[max_len(20)]
    pub privacy_level: String,            // "Zero-Knowledge", "Pseudonymous", "Anonymous"
    
    pub is_revoked: bool,                 // Whether the participant revoked their consent
    pub bump: u8,                         // PDA bump seed for account creation
} 