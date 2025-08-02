use anchor_lang::prelude::*;

// Legacy constant - keeping for compatibility
#[constant]
pub const SEED: &str = "anchor";

// Protocol configuration constants (using proper types for #[constant])
pub const MAX_TITLE_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_IPFS_CID_LENGTH: usize = 100;

#[constant]
pub const MIN_STUDY_DURATION: i64 = 86400; // 1 day in seconds

#[constant]
pub const MAX_STUDY_DURATION: i64 = 31536000; // 1 year in seconds

#[constant]
pub const MIN_ENROLLMENT_WINDOW: i64 = 3600; // 1 hour in seconds

pub const MAX_PARTICIPANTS_PER_STUDY: u32 = 10000;

// Protocol fee configuration (basis points)
#[constant]
pub const DEFAULT_PROTOCOL_FEE_BPS: u16 = 250; // 2.5%

#[constant] 
pub const MAX_PROTOCOL_FEE_BPS: u16 = 1000; // 10%

// Seed prefixes for PDAs
#[constant]
pub const ADMIN_SEED: &str = "admin";

#[constant]
pub const STUDY_SEED: &str = "study";

#[constant]
pub const CONSENT_SEED: &str = "consent";

#[constant]
pub const SUBMISSION_SEED: &str = "submission";

#[constant]
pub const VAULT_SEED: &str = "vault";

#[constant]
pub const COMPLETION_SEED: &str = "completion";

#[constant]
pub const REWARD_SEED: &str = "reward";

#[constant]
pub const REWARD_VAULT_SEED: &str = "vault";

#[constant]
pub const VAULT_TOKEN_SEED: &str = "vault_token";

#[constant]
pub const SURVEY_SCHEMA_SEED: &str = "schema";

// Time-based constants
#[constant]
pub const SECONDS_PER_DAY: i64 = 86400;

#[constant]
pub const GRACE_PERIOD_SECONDS: i64 = 3600; // 1 hour grace period for submissions
