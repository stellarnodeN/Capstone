use anchor_lang::prelude::*;
use crate::state::*;

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
    // Study account - stores all study data and state
    #[account(
        init,
        payer = researcher,
        space = 8 + StudyAccount::INIT_SPACE,
        seeds = [b"study", researcher.key().as_ref(), study_id.to_le_bytes().as_ref()],
        bump,
        constraint = title.len() <= MAX_TITLE_LENGTH @ RecruSearchError::TitleTooLong,
        constraint = description.len() <= MAX_DESCRIPTION_LENGTH @ RecruSearchError::DescriptionTooLong,
        constraint = max_participants > 0 && max_participants <= MAX_PARTICIPANTS_PER_STUDY @ RecruSearchError::InvalidMaxParticipants,
        constraint = enrollment_end > enrollment_start @ RecruSearchError::InvalidEnrollmentEnd,
        constraint = data_collection_end > enrollment_end @ RecruSearchError::InvalidDataCollectionEnd,
    )]
    pub study: Account<'info, StudyAccount>,

    // Only the researcher can create the study
    #[account(mut)]
    pub researcher: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

// Study publishing - makes a draft study available for participant enrollment

#[derive(Accounts)]
pub struct PublishStudy<'info> {
    // Study account to be published
    #[account(
        mut,
        seeds = [b"study", researcher.key().as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status == StudyStatus::Draft @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    // Only the study researcher can publish
    #[account(mut)]
    pub researcher: Signer<'info>,
}

// permanently closes a study to new enrollments

#[derive(Accounts)]
pub struct CloseStudy<'info> {
    // Study account to be closed
    #[account(
        mut,
        seeds = [b"study", researcher.key().as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status != StudyStatus::Closed @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    // Only the study researcher can close
    #[account(mut)]
    pub researcher: Signer<'info>,
}

// Study state transition -handles automatic state changes based on time

#[derive(Accounts)]
pub struct TransitionStudyState<'info> {
    // Study account for state transition
    #[account(
        mut,
        seeds = [b"study", study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump
    )]
    pub study: Account<'info, StudyAccount>,
}

impl<'info> CreateStudy<'info> {
    // Creates a new study with validated parameters and initial state
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

        // Validate enrollment start time
        require!(enrollment_start > clock.unix_timestamp, RecruSearchError::InvalidEnrollmentStart);
        
        // Validate enrollment period duration
        let enrollment_duration = enrollment_end - enrollment_start;
        require!(
            enrollment_duration >= MIN_ENROLLMENT_WINDOW,
            RecruSearchError::InvalidEnrollmentPeriod
        );

        // Validate total study duration
        let total_duration = data_collection_end - enrollment_start;
        require!(
            total_duration >= MIN_STUDY_DURATION && total_duration <= MAX_STUDY_DURATION,
            RecruSearchError::InvalidDataCollectionPeriod
        );

        // Initialize study account 
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
        study.created_at = clock.unix_timestamp;

        // Initialize eligibility criteria fields
        study.has_eligibility_criteria = false;
        study.eligibility_criteria = Vec::new();
        study.bump = bumps.study;
        study.total_rewards_distributed = 0;

        // Log study creation details
        msg!("Study created with ID: {}", study_id);
        msg!("Title: {}", title);
        msg!("Researcher: {}", self.researcher.key());
        msg!("Max participants: {}", max_participants);
        msg!("Reward amount: {} lamports", reward_amount);

        // Emit study created event
        emit!(StudyCreated {
            study_id,
            title: title.clone(),
            researcher: self.researcher.key(),
            max_participants,
            reward_amount,
        });

        Ok(())
    }
}

impl<'info> PublishStudy<'info> {
    // Publishes a draft study to make it available for enrollment
    pub fn publish_study(&mut self) -> Result<()> {
        let study = &mut self.study;
        let clock = Clock::get()?;
        
        // Change status to published
        study.status = StudyStatus::Published;
        
        // Log publication details
        msg!("Study published: {} at timestamp: {}", study.study_id, clock.unix_timestamp);
        msg!("Study published: {}", study.study_id);
        msg!("Now accepting participants");
        
        // Emit study published event
        emit!(StudyPublished {
            study_id: study.study_id,
            researcher: self.researcher.key(),
        });
        
        Ok(())
    }
}

impl<'info> CloseStudy<'info> {
    // Permanently closes a study to new enrollments and data submissions
    pub fn close_study(&mut self) -> Result<()> {
        let study = &mut self.study;
        let clock = Clock::get()?;
        
        // Change status to closed
        study.status = StudyStatus::Closed;
        
        // Log closure details
        msg!("Study closed: {} at timestamp: {}", study.study_id, clock.unix_timestamp);
        msg!("Study closed: {}", study.study_id);
        msg!("No longer accepting new participants or data submissions");
        
        // Emit study closed event 
        emit!(StudyClosed {
            study_id: study.study_id,
            researcher: self.researcher.key(),
            total_participants: study.enrolled_count,
            total_submissions: study.completed_count,
        });
        
        Ok(())
    }
}

impl<'info> TransitionStudyState<'info> {
    // Handles automatic state transitions based on time conditions
    pub fn transition_study_state(&mut self) -> Result<()> {
        let study = &mut self.study;
        let clock = Clock::get()?;
        
        let current_time = clock.unix_timestamp;
        
        // Check for automatic transitions based on current state and time
        match study.status {
            StudyStatus::Published => {
                // Auto-transition to Active when data collection period starts
                if current_time >= study.data_collection_end {
                    study.status = StudyStatus::Active;
                    msg!("Study transitioned to Active state");
                }
            },
            StudyStatus::Active => {
                // Manual transition to Closed via close_study
            },
            _ => {
                return Err(RecruSearchError::InvalidStudyState.into());
            }
        }
        
        Ok(())
    }
}