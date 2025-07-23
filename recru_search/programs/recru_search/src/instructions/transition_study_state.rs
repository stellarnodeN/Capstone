use anchor_lang::prelude::*;
use crate::state::{StudyAccount, StudyStatus};
use crate::error::RecruSearchError;

// Seed prefixes for account PDAs
const STUDY_SEED_PREFIX: &str = "study";

// Account validation struct for automatic study state transitions
// This allows anyone to trigger state transitions when time conditions are met
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct TransitionStudyState<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED_PREFIX.as_bytes(), study_account.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump,
    )]
    pub study_account: Account<'info, StudyAccount>,
}

impl<'info> TransitionStudyState<'info> {
    pub fn transition_study_state(&mut self, study_id: u64) -> Result<()> {
        let study_account = &mut self.study_account;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Log current state for debugging
        msg!("Current study status: {:?}", study_account.status);
        msg!("Current time: {}", current_time);
        msg!("Enrollment end: {}", study_account.enrollment_end);
        msg!("Data collection end: {}", study_account.data_collection_end);

        // Determine appropriate state transition based on current status and time
        match study_account.status {
            StudyStatus::Published => {
                // Transition from Published to Active when enrollment period ends
                if current_time >= study_account.enrollment_end {
                    study_account.status = StudyStatus::Active;
                    msg!(
                        "Study {} transitioned from Published to Active at timestamp {}",
                        study_id,
                        current_time
                    );
                } else {
                    msg!(
                        "Study {} still in enrollment period. {} seconds remaining.",
                        study_id,
                        study_account.enrollment_end - current_time
                    );
                }
            }
            StudyStatus::Active => {
                // Transition from Active to Closed when data collection period ends
                if current_time >= study_account.data_collection_end {
                    study_account.status = StudyStatus::Closed;
                    msg!(
                        "Study {} transitioned from Active to Closed at timestamp {}",
                        study_id,
                        current_time
                    );
                } else {
                    msg!(
                        "Study {} still in data collection period. {} seconds remaining.",
                        study_id,
                        study_account.data_collection_end - current_time
                    );
                }
            }
            StudyStatus::Draft => {
                // Check if study should auto-publish (if enrollment start time has passed)
                if current_time >= study_account.enrollment_start {
                    study_account.status = StudyStatus::Published;
                    msg!(
                        "Study {} auto-transitioned from Draft to Published at timestamp {}",
                        study_id,
                        current_time
                    );
                } else {
                    msg!(
                        "Study {} still in draft phase. {} seconds until enrollment starts.",
                        study_id,
                        study_account.enrollment_start - current_time
                    );
                }
            }
            StudyStatus::Closed => {
                // No automatic transitions from Closed state
                // Archival must be done manually by researcher
                msg!("Study {} is already closed. No automatic transitions available.", study_id);
                return Err(RecruSearchError::InvalidStatusTransition.into());
            }
            StudyStatus::Archived => {
                // No transitions from Archived state
                msg!("Study {} is archived. No transitions allowed.", study_id);
                return Err(RecruSearchError::InvalidStatusTransition.into());
            }
        }

        Ok(())
    }
} 