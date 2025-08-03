// State module organization following Anchor best practices
pub mod accounts;    // All account data structures
pub mod constants;   // Program constants and configuration
pub mod errors;      // Custom error definitions
pub mod events;      // Event definitions for logging

// Re-export all state components
pub use accounts::*;
pub use constants::*;


// Re-export specific items for easier access
pub use accounts::{
    AdminAccount,
    StudyAccount,
    ConsentAccount,
    SubmissionAccount,
    RewardVault,
    SurveySchema,
    DataCollectionStats,
};

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

pub use errors::RecruSearchError;
