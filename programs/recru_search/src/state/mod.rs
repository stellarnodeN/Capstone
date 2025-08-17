pub mod accounts;
pub mod constants;
pub mod errors;
pub mod events;

pub use accounts::*;
pub use constants::*;
pub use events::*;

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

pub use errors::RecruSearchError;