use anchor_lang::prelude::*;
use crate::state::study::{StudyAccount, StudyStatus};
use crate::error::RecruSearchError;

// Seed prefix for study account PDAs
const STUDY_SEED_PREFIX: &str = "study";

// Account validation struct for publishing research studies
// This defines what accounts must be provided when a researcher publishes their study
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct PublishStudy<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED_PREFIX.as_bytes(), researcher.key().as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump,
        constraint = study_account.researcher == researcher.key() @ RecruSearchError::UnauthorizedAccess
    )]
    pub study_account: Account<'info, StudyAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> PublishStudy<'info> {
    pub fn publish_study(&mut self, study_id: u64) -> Result<()> {
        let study_account = &mut self.study_account;
        let clock = Clock::get()?;

        // Validate current status is Draft
        require!(
            matches!(study_account.status, StudyStatus::Draft),
            RecruSearchError::InvalidStatusTransition
        );

        // Validate enrollment start is in the future (or now)
        require!(
            study_account.enrollment_start >= clock.unix_timestamp,
            RecruSearchError::InvalidEnrollmentPeriod
        );

        // Validate all required fields are set
        require!(
            study_account.max_participants > 0,
            RecruSearchError::InvalidMaxParticipants
        );

        // Transition status to Published
        study_account.status = StudyStatus::Published;

        msg!(
            "Study published: ID {}, Title: {}",
            study_id,
            study_account.title
        );

        Ok(())
    }
} 