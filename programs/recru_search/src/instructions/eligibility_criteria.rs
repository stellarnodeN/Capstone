use anchor_lang::prelude::*;
use crate::state::{StudyAccount, StudyStatus, RecruSearchError};

// Eligibility criteria management - allows researchers to set participant requirements

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct SetEligibilityCriteria<'info> {
    // Study account to set criteria for
    #[account(
        mut,
        seeds = [b"study", study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status == StudyStatus::Draft @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    // Only the study researcher can set criteria
    #[account(mut)]
    pub researcher: Signer<'info>,
}

// Eligibility verification - checks if participants meet study requirements

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct VerifyEligibility<'info> {
    // Study account to verify against
    #[account(
        seeds = [b"study", study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::StudyNotPublished
    )]
    pub study: Account<'info, StudyAccount>,

    /// CHECK: Participant account for eligibility verification
    /// We validate this is the correct participant for the eligibility check
    pub participant: Signer<'info>,
}

// Eligibility criteria structure - defines participant requirements

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EligibilityCriteria {
    pub min_age: Option<u8>,
    pub max_age: Option<u8>,
    pub gender: Option<String>,
    pub location: Option<String>,
    pub education_level: Option<String>,
    pub employment_status: Option<String>,
    pub medical_conditions: Option<Vec<String>>,
    pub custom_requirements: Option<Vec<String>>,
}

// Participant information structure - contains participant demographics

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ParticipantInfo {
    pub age: u8,
    pub gender: String,
    pub location: String,
    pub education_level: String,
    pub employment_status: String,
    pub medical_conditions: Vec<String>,
    pub additional_info: Option<Vec<String>>,
}

impl<'info> SetEligibilityCriteria<'info> {
    // Sets eligibility criteria for a study - only in draft state
    pub fn set_eligibility_criteria(
        &mut self,
        study_id: u64,
        criteria_bytes: Vec<u8>,
    ) -> Result<()> {
        let study = &mut self.study;

        // Validate criteria format by parsing it
        let _criteria: EligibilityCriteria = EligibilityCriteria::try_from_slice(&criteria_bytes)
            .map_err(|_| RecruSearchError::InvalidParameterValue)?;

        // Store validated criteria
        study.eligibility_criteria = criteria_bytes;
        study.has_eligibility_criteria = true;

        msg!("Eligibility criteria set for study: {}", study_id);
        msg!("Criteria stored successfully");

        Ok(())
    }
}

impl<'info> VerifyEligibility<'info> {
    // Verifies if participant meets study eligibility criteria
    pub fn verify_eligibility(
        &mut self,
        study_id: u64,
        participant_info: ParticipantInfo,
    ) -> Result<bool> {
        let study = &self.study;
        let participant = &self.participant;
        
        msg!("Verifying eligibility for participant {} in study: {}", participant.key(), study_id);
        
        // Skip verification if no criteria set
        if !study.has_eligibility_criteria {
            msg!("Study has no eligibility criteria - all participants eligible");
            return Ok(true);
        }

        // Parse stored criteria
        let criteria: EligibilityCriteria = EligibilityCriteria::try_from_slice(&study.eligibility_criteria)
            .map_err(|_| RecruSearchError::InvalidParameterValue)?;

        msg!("Verifying eligibility for study: {}", study_id);

        // Check age requirements
        if let Some(min_age) = criteria.min_age {
            if participant_info.age < min_age {
                msg!("Eligibility verification failed - age requirement not met");
                return Ok(false);
            }
        }

        if let Some(max_age) = criteria.max_age {
            if participant_info.age > max_age {
                msg!("Eligibility verification failed - age requirement not met");
                return Ok(false);
            }
        }

        // Check gender requirement
        if let Some(required_gender) = &criteria.gender {
            if participant_info.gender.to_lowercase() != required_gender.to_lowercase() {
                msg!("Participant gender '{}' does not match required gender '{}'", 
                     participant_info.gender, required_gender);
                return Ok(false);
            }
        }

        // Check location requirement
        if let Some(required_location) = &criteria.location {
            if participant_info.location.to_lowercase() != required_location.to_lowercase() {
                msg!("Participant location '{}' does not match required location '{}'", 
                     participant_info.location, required_location);
                return Ok(false);
            }
        }

        // Check education requirement
        if let Some(required_education) = &criteria.education_level {
            if participant_info.education_level.to_lowercase() != required_education.to_lowercase() {
                msg!("Participant education '{}' does not match required education '{}'", 
                     participant_info.education_level, required_education);
                return Ok(false);
            }
        }

        // Check employment requirement
        if let Some(required_employment) = &criteria.employment_status {
            if participant_info.employment_status.to_lowercase() != required_employment.to_lowercase() {
                msg!("Participant employment '{}' does not match required employment '{}'", 
                     participant_info.employment_status, required_employment);
                return Ok(false);
            }
        }

        // Check medical conditions exclusion
        if let Some(excluded_conditions) = &criteria.medical_conditions {
            for condition in excluded_conditions {
                if participant_info.medical_conditions.iter().any(|c| c.to_lowercase() == condition.to_lowercase()) {
                    msg!("Eligibility verification failed - medical criteria not met");
                    return Ok(false);
                }
            }
        }

        // Check custom requirements
        if let Some(custom_requirements) = &criteria.custom_requirements {
            for requirement in custom_requirements {
                if !participant_info.additional_info.as_ref()
                    .map(|info| info.iter().any(|item| item.to_lowercase() == requirement.to_lowercase()))
                    .unwrap_or(false) {
                    msg!("Participant does not meet custom requirement: {}", requirement);
                    return Ok(false);
                }
            }
        }

        msg!("Participant {} meets all eligibility criteria for study: {}", participant.key(), study_id);
        Ok(true)
    }
}



