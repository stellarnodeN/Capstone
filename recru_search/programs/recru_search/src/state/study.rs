use anchor_lang::prelude::*;

// Enum to track the current state of a research study
// Each study moves through these states in a specific order
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum StudyStatus {
    Draft,          // Study created but not yet published (can be edited)
    Published,      // Study is live and accepting participants
    DataCollection, // Study is in data collection phase
    Active,         // Study in data collection phase (alias for DataCollection)
    Closed,         // Study completed, no new participants allowed
    Archived,       // Study archived for long-term storage
}

impl Default for StudyStatus {
    fn default() -> Self {
        StudyStatus::Draft  // New studies start as drafts
    }
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