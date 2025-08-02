use anchor_lang::prelude::*;

// Protocol Events
#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub fee_bps: u16,
    pub min_duration: u64,
    pub max_duration: u64,
}

// Study Events
#[event]
pub struct StudyCreated {
    pub study_id: u64,
    pub title: String,
    pub researcher: Pubkey,
    pub max_participants: u32,
    pub reward_amount: u64,
}

#[event]
pub struct StudyPublished {
    pub study_id: u64,
    pub researcher: Pubkey,
}

#[event]
pub struct StudyClosed {
    pub study_id: u64,
    pub researcher: Pubkey,
    pub total_participants: u32,
    pub total_submissions: u32,
}

// Consent Events
#[event]
pub struct ConsentNFTMinted {
    pub study_id: u64,
    pub participant: Pubkey,
    pub consent_nft_mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ConsentRevoked {
    pub study_id: u64,
    pub participant: Pubkey,
    pub timestamp: i64,
}

// Data Submission Events
#[event]
pub struct DataSubmitted {
    pub study_id: u64,
    pub participant: Pubkey,
    pub ipfs_cid: String,
    pub timestamp: i64,
}

// Reward Events
#[event]
pub struct RewardVaultCreated {
    pub study_id: u64,
    pub researcher: Pubkey,
    pub reward_mint: Pubkey,
    pub initial_deposit: u64,
}

#[event]
pub struct RewardDistributed {
    pub study_id: u64,
    pub participant: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

// Survey Schema Events
#[event]
pub struct SurveySchemaCreated {
    pub study_id: u64,
    pub researcher: Pubkey,
    pub schema_version: u32,
    pub question_count: u32,
    pub estimated_duration: u32,
}

// Completion NFT Events
#[event]
pub struct CompletionNFTMinted {
    pub study_id: u64,
    pub participant: Pubkey,
    pub completion_nft_mint: Pubkey,
    pub timestamp: i64,
}

// Error Events
#[event]
pub struct StudyError {
    pub study_id: u64,
    pub error_code: u32,
    pub error_message: String,
    pub timestamp: i64,
}

// Statistics Events
#[event]
pub struct StudyStatistics {
    pub study_id: u64,
    pub total_participants: u32,
    pub total_submissions: u32,
    pub total_rewards_distributed: u64,
    pub average_completion_time: u32,
    pub timestamp: i64,
} 