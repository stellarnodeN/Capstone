use anchor_lang::prelude::*;



/// Study status enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum StudyStatus {
    Draft,
    Published,
    Active,
    Closed,
}

/// Protocol administration account
#[account]
#[derive(InitSpace)]
pub struct AdminAccount {
    pub protocol_admin: Pubkey,        // Protocol administrator
    pub protocol_fee_bps: u16,         // Protocol fee in basis points (0.01%)
    pub min_study_duration: u64,       // Minimum study duration in seconds
    pub max_study_duration: u64,       // Maximum study duration in seconds
    pub total_studies: u64,            // Total number of studies created
    pub total_participants: u64,       // Total number of participants across all studies
    pub total_rewards_distributed: u64, // Total rewards distributed
    pub bump: u8,                      // PDA bump seed
}



/// Study management account
#[account]
#[derive(InitSpace)]
pub struct StudyAccount {
    pub study_id: u64,                 // Unique study identifier
    #[max_len(100)]
    pub title: String,                 // Study title
    #[max_len(500)]
    pub description: String,           // Study description
    pub researcher: Pubkey,            // Study creator/researcher
    pub enrollment_start: i64,         // Enrollment start timestamp
    pub enrollment_end: i64,           // Enrollment end timestamp
    pub data_collection_end: i64,      // Data collection end timestamp
    pub max_participants: u32,         // Maximum number of participants
    pub enrolled_count: u32,           // Current number of participants (renamed from current_participants)
    pub reward_amount_per_participant: u64, // Reward amount per participant (renamed from reward_amount)
    pub status: StudyStatus,           // Study status (Draft, Published, Active, Closed)
    pub completed_count: u32,          // Total data submissions received (renamed from total_submissions)
    pub total_rewards_distributed: u64, // Total rewards distributed for this study
    pub created_at: i64,               // When study was created
    pub requires_zk_proof: bool,       // Whether study requires ZK proof
    pub has_eligibility_criteria: bool, // Whether study has eligibility criteria set
    #[max_len(1000)]
    pub eligibility_criteria: Vec<u8>, // Serialized eligibility criteria
    pub bump: u8,                      // PDA bump seed
}



/// Consent NFT tracking account
#[account]
#[derive(InitSpace)]
pub struct ConsentAccount {
    pub study: Pubkey,                 // Associated study account
    pub participant: Pubkey,           // Participant's wallet address
    pub consent_nft_mint: Pubkey,      // Consent NFT mint address
    #[max_len(1000)]
    pub eligibility_proof: Vec<u8>,    // Proof of eligibility
    pub timestamp: i64,                // When consent was given (renamed from consent_timestamp)
    pub is_revoked: bool,              // Whether consent is revoked (renamed from is_active, inverted logic)
    pub revocation_timestamp: Option<i64>, // When consent was revoked
    pub nft_mint: Option<Pubkey>,      // NFT mint address (renamed from consent_nft_mint)
    pub bump: u8,                      // PDA bump seed
}



/// Data submission tracking account
#[account]
#[derive(InitSpace)]
pub struct SubmissionAccount {
    pub study: Pubkey,                 // Associated study account
    pub participant: Pubkey,           // Participant's wallet address
    pub encrypted_data_hash: [u8; 32], // Hash of the encrypted data for verification
    #[max_len(100)]
    pub ipfs_cid: String,              // IPFS content identifier where encrypted data is stored
    pub submission_timestamp: i64,     // When data was submitted
    pub is_verified: bool,             // Whether submission has been verified
    pub reward_distributed: bool,      // Whether reward has been distributed
    pub completion_nft_mint: Option<Pubkey>, // Completion NFT mint (if minted)
    pub bump: u8,                      // PDA bump seed
}



/// Reward vault account for managing study rewards
#[account]
#[derive(InitSpace)]
pub struct RewardVault {
    pub study: Pubkey,                 // Associated study account
    pub reward_token_mint: Pubkey,     // Token mint for rewards
    pub total_deposited: u64,          // Total amount deposited by researcher
    pub total_distributed: u64,        // Total amount distributed to participants
    pub vault_authority: Pubkey,       // PDA authority for the vault
    pub bump: u8,                      // PDA bump seed
}



/// Survey schema and statistics account
#[account]
#[derive(InitSpace)]
pub struct SurveySchema {
    pub study: Pubkey,                 // Associated study account
    pub schema_version: u32,           // Schema version number
    pub question_count: u32,           // Number of questions in survey
    pub estimated_duration_minutes: u32, // Estimated completion time
    #[max_len(100)]
    pub schema_ipfs_cid: String,       // IPFS CID for detailed schema
    pub requires_encryption: bool,     // Whether data must be encrypted
    pub supports_file_uploads: bool,   // Whether file uploads are supported
    pub anonymous_responses: bool,     // Whether responses are anonymous
    pub total_responses: u32,          // Total number of responses received
    pub average_completion_time: u32,  // Average completion time in minutes
    #[max_len(100)]
    pub export_ipfs_cid: String,       // IPFS CID for exported data (if available)
    pub bump: u8,                      // PDA bump seed
}

/// Data collection statistics account
#[account]
#[derive(InitSpace)]
pub struct DataCollectionStats {
    pub study: Pubkey,                 // Associated study account
    pub researcher: Pubkey,            // Study researcher
    pub total_responses: u32,          // Total responses received
    pub complete_responses: u32,       // Complete responses
    pub validated_responses: u32,      // Validated responses
    pub encrypted_responses: u32,      // Encrypted responses
    pub anonymized_responses: u32,     // Anonymized responses
    pub gdpr_deletion_requests: u32,   // GDPR deletion requests
    pub total_files_uploaded: u32,     // Total files uploaded
    pub total_file_size_mb: u32,       // Total file size in MB
    pub average_completion_time_seconds: u32, // Average completion time
    pub first_response_timestamp: i64, // First response timestamp
    pub last_response_timestamp: i64,  // Last response timestamp
    pub last_updated: i64,             // Last update timestamp
    pub bump: u8,                      // PDA bump seed
}