use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

// Content length limits for study metadata
pub const MAX_TITLE_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;

// Study duration constraints (simplified)
#[constant]
pub const MIN_STUDY_DURATION: i64 = 86400; // 1 day
#[constant]
pub const MAX_STUDY_DURATION: i64 = 31536000; // 1 year
#[constant]
pub const MIN_ENROLLMENT_WINDOW: i64 = 3600; // 1 hour

// Study participation limits
pub const MAX_PARTICIPANTS_PER_STUDY: u32 = 10000;

// Protocol fee constants
pub const DEFAULT_PROTOCOL_FEE_BPS: u16 = 250; // 2.5%
pub const MAX_PROTOCOL_FEE_BPS: u16 = 1000; // 10%

// NFT symbols
pub const CONSENT_NFT_SYMBOL: &str = "RCONSENT";
pub const COMPLETION_NFT_SYMBOL: &str = "RCOMPLETE";

// Template images for NFTs (standard images with dynamic metadata)
pub const CONSENT_NFT_TEMPLATE_IMAGE: &str = "ipfs://bafkreiaich32x7g4cajovenhlnvn3jfedf3vkh4pqiyfa6g2e26zi7chkm";
pub const COMPLETION_NFT_TEMPLATE_IMAGE: &str = "ipfs://bafkreiaich32x7g4cajovenhlnvn3jfedf3vkh4pqiyfa6g2e26zi7chkm";

// Basic eligibility constraints
pub const MIN_AGE_LIMIT: u8 = 18;
pub const MAX_AGE_LIMIT: u8 = 100;
pub const MAX_ELIGIBILITY_CRITERIA_SIZE: usize = 500; // Reduced from 1000
