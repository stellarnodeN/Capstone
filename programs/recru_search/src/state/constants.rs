use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

// Content length limits for study metadata
pub const MAX_TITLE_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_IPFS_CID_LENGTH: usize = 100;

// IPFS CID validation constants
pub const MIN_IPFS_CID_LENGTH: usize = 46;  // Minimum CID v0 length
pub const IPFS_CID_V0_PREFIX: &str = "Qm";  // CID v0 prefix
pub const IPFS_CID_V1_PREFIX: &str = "bafy"; // CID v1 prefix (most common)

// Study duration constraints
#[constant]
pub const MIN_STUDY_DURATION: i64 = 86400; // 1 day

#[constant]
pub const MAX_STUDY_DURATION: i64 = 31536000; // 1 year

#[constant]
pub const MIN_ENROLLMENT_WINDOW: i64 = 3600; // 1 hour

// Study participation limits
pub const MAX_PARTICIPANTS_PER_STUDY: u32 = 10000;

// Protocol fee configuration
#[constant]
pub const DEFAULT_PROTOCOL_FEE_BPS: u16 = 250; // 2.5%

#[constant] 
pub const MAX_PROTOCOL_FEE_BPS: u16 = 1000; // 10%

// PDA seeds for account derivation
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
pub const SURVEY_SCHEMA_SEED: &str = "survey";

#[constant]
pub const DATA_STATS_SEED: &str = "data_stats";

// Time-based constants
#[constant]
pub const SECONDS_PER_DAY: i64 = 86400;

#[constant]
pub const GRACE_PERIOD_SECONDS: i64 = 3600; // 1 hour

// NFT metadata constants for completion and consent tokens
#[constant]
pub const DEFAULT_COMPLETION_NFT_METADATA_URI: &str = "ipfs://bafkreihzn56xpmslsfakm3sjpnxquhdmgrip5snn3leyqbyzewuxzw2ofa";

#[constant]
pub const COMPLETION_NFT_SYMBOL: &str = "RSCOMPLETE";

#[constant]
pub const CONSENT_NFT_SYMBOL: &str = "RSCONSENT";
