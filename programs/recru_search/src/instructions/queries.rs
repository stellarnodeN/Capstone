use anchor_lang::prelude::*;
use crate::state::{StudyAccount, StudyStatus, ConsentAccount, AdminAccount};

// Study information queries - provides study details and progress

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct GetStudyInfo<'info> {
    // Study account to query
    #[account(
        seeds = [b"study", study_account.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump
    )]
    pub study_account: Account<'info, StudyAccount>,
}

// Consent status queries - checks participant enrollment status

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct GetConsentStatus<'info> {
    // Study account for reference
    #[account(
        seeds = [b"study", study_account.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump
    )]
    pub study_account: Account<'info, StudyAccount>,

    // Consent account to check status
    #[account(
        seeds = [b"consent", study_account.key().as_ref(), participant.key().as_ref()],
        bump = consent_nft_account.bump
    )]
    pub consent_nft_account: Account<'info, ConsentAccount>,

    /// CHECK: This is the participant account to check consent status for
    pub participant: UncheckedAccount<'info>,
}

// Protocol statistics queries - provides global protocol information

#[derive(Accounts)]
pub struct GetProtocolStats<'info> {
    // Admin state account for protocol stats
    #[account(
        seeds = [b"admin"],
        bump = admin_state.bump
    )]
    pub admin_state: Account<'info, AdminAccount>,
}

impl<'info> GetStudyInfo<'info> {
    // Retrieves comprehensive study information and progress metrics
    pub fn get_study_info(&self, _study_id: u64) -> Result<StudyInfo> {
        let study = &self.study_account;
        let clock = Clock::get()?;

        // Calculate enrollment and time metrics
        let enrollment_progress = self.calculate_enrollment_progress(study, clock.unix_timestamp)?;
        let time_remaining = self.calculate_time_remaining(study, clock.unix_timestamp)?;

        Ok(StudyInfo {
            study_id: study.study_id,
            title: study.title.clone(),
            description: study.description.clone(),
            researcher: study.researcher,
            status: study.status.clone(),
            enrollment_progress,
            time_remaining,
            max_participants: study.max_participants,
            enrolled_count: study.enrolled_count,
            completed_count: study.completed_count,
            reward_amount: study.reward_amount_per_participant,
            created_at: study.created_at,
        })
    }

    // Calculates enrollment progress percentages and status
    fn calculate_enrollment_progress(&self, study: &StudyAccount, current_time: i64) -> Result<EnrollmentProgress> {
        // Calculate participant enrollment percentage
        let enrollment_percentage = if study.max_participants > 0 {
            (study.enrolled_count as f64 / study.max_participants as f64 * 100.0) as u8
        } else {
            0
        };

        // Calculate time progress percentage
        let time_progress = if study.enrollment_end > study.enrollment_start {
            let total_time = study.enrollment_end - study.enrollment_start;
            let elapsed_time = current_time - study.enrollment_start;
            ((elapsed_time as f64 / total_time as f64 * 100.0).min(100.0).max(0.0)) as u8
        } else {
            0
        };

        Ok(EnrollmentProgress {
            participants_percentage: enrollment_percentage,
            time_percentage: time_progress,
            is_enrollment_open: current_time >= study.enrollment_start && current_time <= study.enrollment_end,
        })
    }

    // Calculates time remaining for different study phases
    fn calculate_time_remaining(&self, study: &StudyAccount, current_time: i64) -> Result<TimeRemaining> {
        // Calculate seconds remaining for each phase
        let seconds_until_enrollment_start = (study.enrollment_start - current_time).max(0);
        let seconds_until_enrollment_end = (study.enrollment_end - current_time).max(0);
        let seconds_until_data_collection_end = (study.data_collection_end - current_time).max(0);

        Ok(TimeRemaining {
            until_enrollment_start: seconds_until_enrollment_start,
            until_enrollment_end: seconds_until_enrollment_end,
            until_data_collection_end: seconds_until_data_collection_end,
            current_phase: self.determine_current_phase(study, current_time)?,
        })
    }

    // Determines current study phase based on time and status
    fn determine_current_phase(&self, study: &StudyAccount, current_time: i64) -> Result<String> {
        if matches!(study.status, StudyStatus::Draft) {
            Ok("Draft".to_string())
        } else if current_time < study.enrollment_start {
            Ok("Pre-Enrollment".to_string())
        } else if current_time <= study.enrollment_end {
            Ok("Enrollment Active".to_string())
        } else if current_time <= study.data_collection_end {
            Ok("Data Collection".to_string())
        } else {
            Ok("Completed".to_string())
        }
    }
}

impl<'info> GetConsentStatus<'info> {
    // Retrieves participant consent status for a study
    pub fn get_consent_status(&self) -> Result<ConsentStatus> {
        let consent = &self.consent_nft_account;
        
        Ok(ConsentStatus {
            has_consented: true,
            is_revoked: consent.is_revoked,
            consent_timestamp: consent.timestamp,
            study_id: self.study_account.study_id,
            participant: consent.participant,
        })
    }
}

impl<'info> GetProtocolStats<'info> {
    // Retrieves global protocol statistics and configuration
    pub fn get_protocol_stats(&self) -> Result<ProtocolStats> {
        let admin_state = &self.admin_state;
        
        Ok(ProtocolStats {
            protocol_admin: admin_state.protocol_admin,
            protocol_fee_bps: admin_state.protocol_fee_bps,
            min_study_duration: admin_state.min_study_duration as u32,
            max_study_duration: admin_state.max_study_duration as u32,
            is_paused: false,
        })
    }
}

// Study information response structure
#[derive(Debug)]
pub struct StudyInfo {
    pub study_id: u64,
    pub title: String,
    pub description: String,
    pub researcher: Pubkey,
    pub status: StudyStatus,
    pub enrollment_progress: EnrollmentProgress,
    pub time_remaining: TimeRemaining,
    pub max_participants: u32,
    pub enrolled_count: u32,
    pub completed_count: u32,
    pub reward_amount: u64,
    pub created_at: i64,
}

// Enrollment progress metrics structure
#[derive(Debug)]
pub struct EnrollmentProgress {
    pub participants_percentage: u8,
    pub time_percentage: u8,
    pub is_enrollment_open: bool,
}

// Time remaining for study phases structure
#[derive(Debug)]
pub struct TimeRemaining {
    pub until_enrollment_start: i64,
    pub until_enrollment_end: i64,
    pub until_data_collection_end: i64,
    pub current_phase: String,
}

// Consent status response structure
#[derive(Debug)]
pub struct ConsentStatus {
    pub has_consented: bool,
    pub is_revoked: bool,
    pub consent_timestamp: i64,
    pub study_id: u64,
    pub participant: Pubkey,
}

// Protocol statistics response structure
#[derive(Debug)]
pub struct ProtocolStats {
    pub protocol_admin: Pubkey,
    pub protocol_fee_bps: u16,
    pub min_study_duration: u32,
    pub max_study_duration: u32,
    pub is_paused: bool,
} 