use anchor_lang::prelude::*;

// Study status enum - defines the lifecycle states of research studies
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum StudyStatus {
    Draft,
    Published,
    Active,
    Closed,
}

// Global admin account - stores RecruSearch config
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

// Study account - stores research study config and state
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
    #[max_len(1000)]
    pub eligibility_criteria: Vec<u8>,
    pub bump: u8,
}

// Consent account - tracks participant enrollment and consent status
#[account]
#[derive(InitSpace)]
pub struct ConsentAccount {
    pub study: Pubkey,
    pub participant: Pubkey,
    #[max_len(1000)]
    pub eligibility_proof: Vec<u8>,
    pub timestamp: i64,
    pub is_revoked: bool,
    pub revocation_timestamp: Option<i64>,
    pub nft_mint: Option<Pubkey>,
    pub bump: u8,
}

// Submission account - stores encrypted data submission metadata
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

// Reward vault account - manages token rewards for study participants
#[account]
#[derive(InitSpace)]
pub struct RewardVault {
    pub study: Pubkey,
    pub reward_token_mint: Pubkey,
    pub total_deposited: u64,
    pub total_distributed: u64,
    pub bump: u8,
}

// Survey schema account - defines data collection structure and metadata
#[account]
#[derive(InitSpace)]
pub struct SurveySchema {
    pub study: Pubkey,
    #[max_len(100)]
    pub title: String,
    pub schema_version: u32,
    pub question_count: u32,
    pub estimated_duration_minutes: u32,
    #[max_len(100)]
    pub schema_ipfs_cid: String,
    pub requires_encryption: bool,
    pub supports_file_uploads: bool,
    pub total_responses: u32,
    pub average_completion_time: u32,
    #[max_len(100)]
    pub export_ipfs_cid: String,
    pub bump: u8,
}

// Data collection statistics account - tracks survey response metrics
#[account]
#[derive(InitSpace)]
pub struct DataCollectionStats {
    pub study: Pubkey,
    pub researcher: Pubkey,
    pub total_responses: u32,
    pub complete_responses: u32,
    pub validated_responses: u32,
    pub encrypted_responses: u32,
    pub total_files_uploaded: u32,
    pub total_file_size_mb: u32,
    pub average_completion_time_seconds: u32,
    pub first_response_timestamp: i64,
    pub last_response_timestamp: i64,
    pub last_updated: i64,
    pub bump: u8,
}