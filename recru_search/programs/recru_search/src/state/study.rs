use anchor_lang::prelude::*;

// Enum to track the current state of a research study
// Each study moves through these states in a specific order: Draft → Published → Active → Closed → Archived
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum StudyStatus {
    Draft,      // Study created but not yet published (can be edited)
    Published,  // Study is published and accepting participants (enrollment phase)
    Active,     // Study is in active data collection phase
    Closed,     // Study completed, no new participants or submissions allowed
    Archived,   // Study archived for long-term storage and compliance
}

// Main account that stores all study information and configuration
// This is the central hub for each research study
#[account]
#[derive(InitSpace)]
pub struct StudyAccount {
    pub study_id: u64,                    // Unique identifier for this study
    pub researcher: Pubkey,               // Wallet address of the researcher who owns this study
    
    #[max_len(100)]
    pub title: String,                    // Human-readable study name (max 100 chars)
    
    #[max_len(500)]
    pub description: String,              // Detailed study description (max 500 chars)
    
    pub consent_document_hash: [u8; 32],  // Hash of the consent form participants must agree to
    pub eligibility_merkle_root: [u8; 32], // Root hash for verifying participant eligibility (ZK proofs)
    pub requires_zk_proof: bool,          // Whether participants need zero-knowledge proof of eligibility
    
    // Eligibility Criteria (User Story #2 - High Priority)
    pub min_age: Option<u8>,              // Minimum age requirement (None = no limit)
    pub max_age: Option<u8>,              // Maximum age requirement (None = no limit)
    pub requires_wallet_verification: bool, // Whether wallet needs to be verified
    pub min_wallet_age_days: Option<u32>, // Minimum wallet age in days
    #[max_len(10)]
    pub excluded_previous_studies: Vec<u64>, // Study IDs participant cannot have joined before
    #[max_len(200)]
    pub custom_eligibility_logic: String, // Custom eligibility criteria (serialized JSON)
    pub eligibility_expires_at: Option<i64>, // When eligibility verification expires
    
    // Time-based study management - all times are Unix timestamps
    pub enrollment_start: i64,            // When participants can start joining
    pub enrollment_end: i64,              // Last moment for new participants to join
    pub data_collection_end: i64,         // Final deadline for data submission
    
    pub status: StudyStatus,              // Current lifecycle state of the study
    
    // Participant management
    pub max_participants: u32,            // Maximum number of people who can join
    pub reward_amount_per_participant: u64, // How much each participant gets paid (in tokens)
    pub enrolled_count: u32,              // Current number of participants who have consented
    pub completed_count: u32,             // Number of participants who submitted data
    
    pub reward_vault: Pubkey,             // Address of the token vault holding participant rewards
    pub created_at: i64,                  // When this study was first created
    pub bump: u8,                         // PDA bump seed for account creation
}