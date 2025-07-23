use anchor_lang::prelude::*;
use crate::state::study::{StudyAccount, StudyStatus};
use crate::error::RecruSearchError;

// Seed prefix for study account PDAs
const STUDY_SEED_PREFIX: &str = "study";

// Account validation struct for creating new research studies
// This defines what accounts must be provided when a researcher creates a study
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct CreateStudy<'info> {
    #[account(
        init,
        payer = researcher,
        space = 8 + StudyAccount::INIT_SPACE,
        seeds = [STUDY_SEED_PREFIX.as_bytes(), researcher.key().as_ref(), study_id.to_le_bytes().as_ref()],
        bump
    )]
    pub study_account: Account<'info, StudyAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateStudy<'info> {
    pub fn create_study(
        &mut self,
        study_id: u64,
        title: String,
        description: String,
        enrollment_start: i64,
        enrollment_end: i64,
        data_collection_end: i64,
        max_participants: u32,
        reward_amount_per_participant: u64,
        bumps: &CreateStudyBumps,
    ) -> Result<()> {
        let study_account = &mut self.study_account;
        let researcher = &self.researcher;
        let clock = Clock::get()?;

        // Validate input parameters
        require!(
            title.len() <= 100,
            RecruSearchError::TitleTooLong
        );
        
        require!(
            description.len() <= 500,
            RecruSearchError::DescriptionTooLong
        );
        
        require!(
            enrollment_start >= clock.unix_timestamp,
            RecruSearchError::InvalidEnrollmentPeriod
        );
        
        require!(
            enrollment_end > enrollment_start,
            RecruSearchError::InvalidEnrollmentPeriod
        );
        
        require!(
            data_collection_end > enrollment_end,
            RecruSearchError::InvalidDataCollectionPeriod
        );
        
        require!(
            max_participants > 0,
            RecruSearchError::InvalidMaxParticipants
        );

        // Initialize study account
        study_account.study_id = study_id;
        study_account.researcher = researcher.key();
        study_account.title = title;
        study_account.description = description;
        study_account.status = StudyStatus::Draft;
        study_account.enrollment_start = enrollment_start;
        study_account.enrollment_end = enrollment_end;
        study_account.data_collection_end = data_collection_end;
        study_account.max_participants = max_participants;
        study_account.enrolled_count = 0;
        study_account.reward_amount_per_participant = reward_amount_per_participant;
        study_account.created_at = clock.unix_timestamp;
        study_account.requires_zk_proof = false; // Default to false
        study_account.consent_document_hash = [0u8; 32]; // Placeholder
        study_account.bump = bumps.study_account;

        msg!(
            "Study created: ID {}, Title: {}, Researcher: {}",
            study_id,
            study_account.title,
            researcher.key()
        );

        Ok(())
    }
} 