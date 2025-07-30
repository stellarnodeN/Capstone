use anchor_lang::prelude::*;
use crate::state::{
    study::{StudyAccount, StudyStatus},
};
use crate::error::RecruSearchError;
use crate::constants::*;

/// Create a new research study
#[derive(Accounts)]
#[instruction(
    study_id: u64,
    title: String, 
    description: String,
    enrollment_start: i64,
    enrollment_end: i64,
    data_collection_end: i64,
    max_participants: u32
)]
pub struct CreateStudy<'info> {
    #[account(
        init,
        payer = researcher,
        space = 8 + StudyAccount::INIT_SPACE,
        seeds = [STUDY_SEED.as_bytes(), researcher.key().as_ref(), study_id.to_le_bytes().as_ref()],
        bump,
        // Move validation to constraints for efficiency
        constraint = title.len() <= MAX_TITLE_LENGTH @ RecruSearchError::TitleTooLong,
        constraint = description.len() <= MAX_DESCRIPTION_LENGTH @ RecruSearchError::DescriptionTooLong,
        constraint = max_participants > 0 && max_participants <= MAX_PARTICIPANTS_PER_STUDY @ RecruSearchError::InvalidMaxParticipants,
        constraint = enrollment_end > enrollment_start @ RecruSearchError::InvalidEnrollmentEnd,
        constraint = data_collection_end > enrollment_end @ RecruSearchError::InvalidDataCollectionEnd,
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>, // Add clock for time validation
}

/// Publish study to make it available for participants
#[derive(Accounts)]
pub struct PublishStudy<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED.as_bytes(), researcher.key().as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status == StudyStatus::Draft @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

/// Close study to prevent new participants or submissions
#[derive(Accounts)]
pub struct CloseStudy<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED.as_bytes(), researcher.key().as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status != StudyStatus::Closed @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

/// Transition study state automatically based on time
#[derive(Accounts)]
pub struct TransitionStudyState<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump
    )]
    pub study: Account<'info, StudyAccount>,
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
        reward_amount: u64,
        bumps: &CreateStudyBumps,
    ) -> Result<()> {
        let study = &mut self.study;
        let clock = Clock::get()?;

        // Time-based validation (requires runtime clock access)
        require!(enrollment_start > clock.unix_timestamp, RecruSearchError::InvalidEnrollmentStart);
        
        // Validate enrollment window duration
        let enrollment_duration = enrollment_end - enrollment_start;
        require!(
            enrollment_duration >= MIN_ENROLLMENT_WINDOW,
            RecruSearchError::InvalidEnrollmentPeriod
        );

        // Validate study duration
        let total_duration = data_collection_end - enrollment_start;
        require!(
            total_duration >= MIN_STUDY_DURATION && total_duration <= MAX_STUDY_DURATION,
            RecruSearchError::InvalidDataCollectionPeriod
        );

        // Initialize study fields directly (moved from state/study.rs)
        study.study_id = study_id;
        study.researcher = self.researcher.key();
        study.title = title.clone();
        study.description = description;
        study.enrollment_start = enrollment_start;
        study.enrollment_end = enrollment_end;
        study.data_collection_end = data_collection_end;
        study.max_participants = max_participants;
        study.reward_amount_per_participant = reward_amount;
        study.enrolled_count = 0;
        study.completed_count = 0;
        study.status = StudyStatus::Draft;
        study.bump = bumps.study;
        study.requires_zk_proof = false;

        msg!("Study created with ID: {}", study_id);
        msg!("Title: {}", title);
        msg!("Researcher: {}", self.researcher.key());
        msg!("Max participants: {}", max_participants);
        msg!("Reward amount: {} lamports", reward_amount);

        Ok(())
    }
}

impl<'info> PublishStudy<'info> {
    pub fn publish_study(&mut self) -> Result<()> {
        let study = &mut self.study;
        let clock = Clock::get()?;
        
        // Update study status and timestamp
        study.status = StudyStatus::Published;
        
        msg!("Study published: {}", study.study_id);
        msg!("Now accepting participants");
        
        Ok(())
    }
}

impl<'info> CloseStudy<'info> {
    pub fn close_study(&mut self) -> Result<()> {
        let study = &mut self.study;
        let clock = Clock::get()?;
        
        // Close the study
        study.status = StudyStatus::Closed;
        
        msg!("Study closed: {}", study.study_id);
        msg!("No longer accepting new participants or data submissions");
        
        Ok(())
    }
}

impl<'info> TransitionStudyState<'info> {
    pub fn transition_study_state(&mut self) -> Result<()> {
        let study = &mut self.study;
        let clock = Clock::get()?;
        
        // Automatic state transitions based on time
        let current_time = clock.unix_timestamp;
        
        match study.status {
            StudyStatus::Published => {
                            if current_time >= study.data_collection_end {
                study.status = StudyStatus::Active;
                msg!("Study transitioned to Active state");
            }
            },
            StudyStatus::Active => {
                // Could add logic to transition to Closed when all data is collected
                // For now, this is manual via close_study
            },
            _ => {
                return Err(RecruSearchError::InvalidStudyState.into());
            }
        }
        
        Ok(())
    }
}