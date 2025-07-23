use anchor_lang::prelude::*;
use crate::state::study::{StudyAccount, StudyStatus};
use crate::error::RecruSearchError;

// Seed prefix for study account PDAs
const STUDY_SEED_PREFIX: &str = "study";

// Account validation struct for closing research studies
// This defines what accounts must be provided when a researcher closes their study
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct CloseStudy<'info> {
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

impl<'info> CloseStudy<'info> {
    pub fn close_study(&mut self, study_id: u64) -> Result<()> {
        let study_account = &mut self.study_account;
        let clock = Clock::get()?;

        // Validate current status allows closing
        require!(
            matches!(
                study_account.status,
                StudyStatus::Published | StudyStatus::DataCollection
            ),
            RecruSearchError::InvalidStatusTransition
        );

        // Validate data collection period has ended
        require!(
            clock.unix_timestamp >= study_account.data_collection_end,
            RecruSearchError::DataCollectionStillActive
        );

        // Transition to Closed status
        study_account.status = StudyStatus::Closed;

        msg!(
            "Study closed: ID {}, Title: {}, Final participant count: {}",
            study_id,
            study_account.title,
            study_account.enrolled_count
        );

        Ok(())
    }
} 