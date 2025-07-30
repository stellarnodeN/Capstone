use anchor_lang::prelude::*;

/// This handles both consent tracking and NFT metadata in one efficient struct
#[account]
#[derive(InitSpace)]
pub struct ConsentAccount {
    pub study: Pubkey,           // Reference to study account
    pub participant: Pubkey,     // Participant wallet
    pub timestamp: i64,          // Consent timestamp
    pub is_revoked: bool,        // Revocation status
    pub revocation_timestamp: Option<i64>, // When consent was revoked
    
    #[max_len(32)]
    pub eligibility_proof: Vec<u8>, // ZK proof data
    pub nft_mint: Option<Pubkey>,    // Reference to consent NFT mint
    pub bump: u8,                // PDA bump
}


#[account]
#[derive(InitSpace)]
pub struct ConsentNFTAccount {
    pub study_id: u64,
    pub participant: Pubkey,
    pub study_account: Pubkey,
    pub consent_timestamp: i64,
    pub eligibility_proof_hash: [u8; 32],
    pub consent_document_version: [u8; 32],
    
    #[max_len(100)]
    pub study_title: String,
    
    #[max_len(50)]
    pub study_type: String,
    
    #[max_len(200)]
    pub metadata_uri: String,
    
    #[max_len(200)]
    pub image_uri: String,
    
    #[max_len(20)]
    pub privacy_level: String,
    
    pub is_revoked: bool,
    pub bump: u8,
}