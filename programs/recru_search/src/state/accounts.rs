use anchor_lang::prelude::*;

// Study status enum 
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum StudyStatus {
    Draft,
    Published,
    Active,
    Closed,
}

// Global admin account 
#[account]
#[derive(InitSpace)]
pub struct AdminAccount {
    pub protocol_admin: Pubkey,
    pub protocol_fee_bps: u16,
    pub min_study_duration: u64,
    pub max_study_duration: u64,
    pub total_studies: u64,
    pub total_participants: u64,
    pub total_rewards_distributed: u64,
    pub bump: u8,
}

// Study account 
#[account]
#[derive(InitSpace)]
pub struct StudyAccount {
    pub study_id: u64,
    #[max_len(100)]
    pub title: String,
    #[max_len(500)]
    pub description: String,
    pub researcher: Pubkey,
    pub enrollment_start: i64,
    pub enrollment_end: i64,
    pub data_collection_end: i64,
    pub max_participants: u32,
    pub enrolled_count: u32,
    pub reward_amount_per_participant: u64,
    pub status: StudyStatus,
    pub completed_count: u32,
    pub total_rewards_distributed: u64,
    pub created_at: i64,

    pub has_eligibility_criteria: bool,
    #[max_len(500)]
    pub eligibility_criteria: Vec<u8>,
    pub bump: u8,
}

// Consent account 
#[account]
#[derive(InitSpace)]
pub struct ConsentAccount {
    pub study: Pubkey,
    pub participant: Pubkey,
    #[max_len(500)]
    pub eligibility_proof: Vec<u8>,
    pub timestamp: i64,
    pub is_revoked: bool,
    pub revocation_timestamp: Option<i64>,
    pub nft_mint: Option<Pubkey>,
    pub bump: u8,
}

// Submission account 
#[account]
#[derive(InitSpace)]
pub struct SubmissionAccount {
    pub study: Pubkey,
    pub participant: Pubkey,
    pub encrypted_data_hash: [u8; 32],
    #[max_len(100)]
    pub ipfs_cid: String,
    pub submission_timestamp: i64,
    pub is_verified: bool,
    pub reward_distributed: bool,
    pub completion_nft_mint: Option<Pubkey>,
    pub bump: u8,
}

// Reward vault account 
#[account]
#[derive(InitSpace)]
pub struct RewardVault {
    pub study: Pubkey,
    pub reward_token_mint: Pubkey,
    pub total_deposited: u64,
    pub total_distributed: u64,
    pub bump: u8,
}

// Survey schema account 
#[account]
#[derive(InitSpace)]
pub struct SurveySchema {
    pub study: Pubkey,
    #[max_len(100)]
    pub title: String,
    #[max_len(100)]
    pub schema_ipfs_cid: String,
    pub requires_encryption: bool,
    pub bump: u8,
}

// Data collection statistics account
#[account]
#[derive(InitSpace)]
pub struct DataCollectionStats {
    pub study: Pubkey,
    pub researcher: Pubkey,
    pub total_responses: u32,
    pub complete_responses: u32,
    pub bump: u8,
}