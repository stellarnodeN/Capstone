use anchor_lang::prelude::*;

// emitted when RecruSearch is first set up
#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub fee_bps: u16,
    pub min_duration: u64,
    pub max_duration: u64,
}

// track study creation, publication, and closure
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

// track participant enrollment and withdrawal
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

//  track encrypted data uploads
#[event]
pub struct DataSubmitted {
    pub study_id: u64,
    pub participant: Pubkey,
    pub ipfs_cid: String,
    pub timestamp: i64,
}

//  track vault creation and token distribution
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

//  track data collection setup
#[event]
pub struct SurveySchemaCreated {
    pub study_id: u64,
    pub researcher: Pubkey,
}

//  track study completion rewards
#[event]
pub struct CompletionNFTMinted {
    pub study_id: u64,
    pub participant: Pubkey,
    pub completion_nft_mint: Pubkey,
    pub timestamp: i64,
}

// log study-related errors for monitoring
#[event]
pub struct StudyError {
    pub study_id: u64,
    pub error_code: u32,
    pub error_message: String,
    pub timestamp: i64,
}

// track study performance metrics
#[event]
pub struct StudyStatistics {
    pub study_id: u64,
    pub total_participants: u32,
    pub total_submissions: u32,
    pub total_rewards_distributed: u64,
    pub average_completion_time: u32,
    pub timestamp: i64,
} 