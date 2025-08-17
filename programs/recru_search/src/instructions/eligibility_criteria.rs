use anchor_lang::prelude::*;
use crate::state::{StudyAccount, RecruSearchError, MAX_ELIGIBILITY_CRITERIA_SIZE, MIN_AGE_LIMIT, MAX_AGE_LIMIT};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EligibilityInfo {
    pub min_age: Option<u8>,        
    pub max_age: Option<u8>,        
    pub gender: Option<String>,      
    pub location: Option<String>,    
}

// Study account constraint for eligibility criteria
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct SetEligibilityCriteria<'info> {
    // Study account to set eligibility criteria 
    #[account(
        mut,
        seeds = [b"study", study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> SetEligibilityCriteria<'info> {
    pub fn set_eligibility_criteria(
        &mut self,
        study_id: u64,
        criteria_bytes: Vec<u8>,
    ) -> Result<()> {
        let study = &mut self.study;
        require!(
            criteria_bytes.len() <= MAX_ELIGIBILITY_CRITERIA_SIZE,
            RecruSearchError::InvalidParameterValue
        );

        let criteria: EligibilityInfo = EligibilityInfo::try_from_slice(&criteria_bytes)
            .map_err(|_| RecruSearchError::InvalidParameterValue)?;

        if let Some(min_age) = criteria.min_age {
            require!(min_age >= MIN_AGE_LIMIT, RecruSearchError::InvalidParameterValue);
        }
        if let Some(max_age) = criteria.max_age {
            require!(max_age <= MAX_AGE_LIMIT, RecruSearchError::InvalidParameterValue);
        }
        if let (Some(min_age), Some(max_age)) = (criteria.min_age, criteria.max_age) {
            require!(min_age <= max_age, RecruSearchError::InvalidParameterValue);
        }

        // Store validated criteria
        study.eligibility_criteria = criteria_bytes;
        study.has_eligibility_criteria = true;

        msg!("Eligibility criteria set for study: {}", study_id);
        msg!("Criteria stored successfully");

        Ok(())
    }
}

// Verify participant eligibility against study criteria
pub fn verify_participant_eligibility(
    study_eligibility_criteria: &[u8],
    participant_info: &EligibilityInfo,
) -> Result<bool> {
    
    let criteria: EligibilityInfo = EligibilityInfo::try_from_slice(study_eligibility_criteria)
        .map_err(|_| RecruSearchError::InvalidParameterValue)?;
    
    verify_eligibility_against_criteria(&criteria, participant_info)
}

// Check if participant info meets study criteria
fn verify_eligibility_against_criteria(
    criteria: &EligibilityInfo,
    participant_info: &EligibilityInfo,
) -> Result<bool> {
    // Check age requirements
    if let Some(min_age) = criteria.min_age {
        if let Some(participant_age) = participant_info.min_age {
            if participant_age < min_age {
                msg!("Eligibility verification failed - participant age {} is below minimum {}", participant_age, min_age);
                return Ok(false);
            }
        } else {
            msg!("Eligibility verification failed - participant age not provided");
            return Ok(false);
        }
    }

    if let Some(max_age) = criteria.max_age {
        if let Some(participant_age) = participant_info.min_age {
            if participant_age > max_age {
                msg!("Eligibility verification failed - participant age {} is above maximum {}", participant_age, max_age);
                return Ok(false);
            }
        } else {
            msg!("Eligibility verification failed - participant age not provided");
            return Ok(false);
        }
    }

    // Check gender requirement (exact match, case-insensitive)
    if let Some(required_gender) = &criteria.gender {
        if let Some(participant_gender) = &participant_info.gender {
            if participant_gender.to_lowercase() != required_gender.to_lowercase() {
                msg!("Participant gender '{}' does not match required gender '{}'", 
                     participant_gender, required_gender);
                return Ok(false);
            }
        } else {
            msg!("Eligibility verification failed - participant gender not provided");
            return Ok(false);
        }
    }

    // Check location requirement (exact match, case-insensitive)
    if let Some(required_location) = &criteria.location {
        if let Some(participant_location) = &participant_info.location {
            if participant_location.to_lowercase() != required_location.to_lowercase() {
                msg!("Participant location '{}' does not match required location '{}'", 
                     participant_location, required_location);
                return Ok(false);
            }
        } else {
            msg!("Eligibility verification failed - participant location not provided");
            return Ok(false);
        }
    }

    msg!("Participant meets all eligibility criteria");
    Ok(true)
}