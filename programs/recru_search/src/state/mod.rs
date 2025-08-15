// State module - exports all account structures, constants, errors, and events

pub mod accounts;
pub mod constants;
pub mod errors;
pub mod events;

// Re-export all items for convenient access
pub use accounts::*;
pub use constants::*;
pub use events::*;

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
    MAX_TITLE_LENGTH,
    MAX_DESCRIPTION_LENGTH,
    MAX_PARTICIPANTS_PER_STUDY,
    DEFAULT_PROTOCOL_FEE_BPS,
    MAX_PROTOCOL_FEE_BPS,
    MIN_STUDY_DURATION,
    MAX_STUDY_DURATION,
    CONSENT_NFT_TEMPLATE_IMAGE,
    COMPLETION_NFT_TEMPLATE_IMAGE,
    CONSENT_NFT_SYMBOL,
    COMPLETION_NFT_SYMBOL,
};

// Export error types for instruction validation
pub use errors::RecruSearchError;
