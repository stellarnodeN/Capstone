// State module - exports all account structures, constants, errors, and events

pub mod accounts;
pub mod constants;
pub mod errors;
pub mod events;      

// Re-export all items for convenient access
pub use accounts::*;
pub use constants::*;

// Export account structures for program instructions
pub use accounts::{
    AdminAccount,
    StudyAccount,
    ConsentAccount,
    SubmissionAccount,
    RewardVault,
    SurveySchema,
    DataCollectionStats,
};

// Export constants for validation and configuration
pub use constants::{
    ADMIN_SEED,
    STUDY_SEED,
    CONSENT_SEED,
    SUBMISSION_SEED,
    VAULT_SEED,
    COMPLETION_SEED,
    REWARD_SEED,
    REWARD_VAULT_SEED,
    VAULT_TOKEN_SEED,
    SURVEY_SCHEMA_SEED,
    DATA_STATS_SEED,
    MAX_TITLE_LENGTH,
    MAX_DESCRIPTION_LENGTH,
    MAX_PARTICIPANTS_PER_STUDY,
    DEFAULT_PROTOCOL_FEE_BPS,
    MAX_PROTOCOL_FEE_BPS,
    MIN_STUDY_DURATION,
    MAX_STUDY_DURATION,
};

// Export error types for instruction validation
pub use errors::RecruSearchError;
