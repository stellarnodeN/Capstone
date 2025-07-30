use anchor_lang::prelude::*;

/// Simple survey schema for MVP - researchers define basic data collection requirements
#[account]
#[derive(InitSpace)]
pub struct SurveySchema {
    pub study_id: u64,
    pub researcher: Pubkey,
    
    #[max_len(100)]
    pub survey_title: String,
    
    #[max_len(300)]
    pub survey_description: String,
    
    pub question_count: u32,
    pub estimated_duration_minutes: u32,
    pub schema_version: u32,
    
    // Off-chain reference for detailed survey content
    #[max_len(100)]
    pub schema_ipfs_cid: String,
    
    // Basic settings
    pub requires_encryption: bool,
    pub supports_file_uploads: bool,
    pub anonymous_responses: bool,
    
    // Lifecycle
    pub created_at: i64,
    pub is_active: bool,
    pub response_count: u32,
    
    pub bump: u8,
}

/// Basic question types for MVP
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum QuestionType {
    ShortText,          // Single line text
    LongText,           // Multi-line text
    MultipleChoice,     // Single selection
    Checkbox,           // Multiple selections
    Scale,              // 1-5 rating scale
    FileUpload,         // File attachment
    YesNo,             // Simple boolean
}

/// Basic data collection statistics
#[account]
#[derive(InitSpace)]
pub struct DataCollectionStats {
    pub study_id: u64,
    pub researcher: Pubkey,
    
    // Response stats
    pub total_responses: u32,
    pub complete_responses: u32,
    pub average_completion_time_seconds: u32,
    pub validated_responses: u32,
    pub encrypted_responses: u32,
    pub anonymized_responses: u32,
    pub gdpr_deletion_requests: u32,
    
    // File stats (simplified - handled off-chain)
    pub total_files_uploaded: u32,
    pub total_file_size_mb: u64,
    
    // Timestamps
    pub first_response_timestamp: i64,
    pub last_response_timestamp: i64,
    pub last_updated: i64,
    
    pub bump: u8,
} 