use anchor_lang::prelude::*;
use crate::state::{StudyAccount, StudyStatus, RecruSearchError};

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct SetEligibilityCriteria<'info> {
    #[account(
        mut,
        seeds = [b"study", study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status == StudyStatus::Draft @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct VerifyEligibility<'info> {
    #[account(
        seeds = [b"study", study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::StudyNotPublished
    )]
    pub study: Account<'info, StudyAccount>,

    /// CHECK: This is the participant account that will be verified for eligibility
    pub participant: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct VerifyEligibilityWithZK<'info> {
    #[account(
        seeds = [b"study", study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::StudyNotPublished,
        constraint = study.requires_zk_proof @ RecruSearchError::ZKProofValidationFailed
    )]
    pub study: Account<'info, StudyAccount>,

    /// CHECK: This is the participant account that will be verified for eligibility with ZK proof
    pub participant: UncheckedAccount<'info>,
}

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EligibilityZKProof {
    pub proof: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub verification_key: Vec<u8>,
}

impl<'info> SetEligibilityCriteria<'info> {
    pub fn set_eligibility_criteria(
        &mut self,
        study_id: u64,
        criteria_bytes: Vec<u8>,
    ) -> Result<()> {
        let study = &mut self.study;

        let criteria: EligibilityCriteria = EligibilityCriteria::try_from_slice(&criteria_bytes)
            .map_err(|_| RecruSearchError::InvalidParameterValue)?;

        study.eligibility_criteria = criteria_bytes;
        study.has_eligibility_criteria = true;

        let requires_zk = criteria.min_age.is_some() || 
                         criteria.max_age.is_some() ||
                         criteria.medical_conditions.is_some();
        study.requires_zk_proof = requires_zk;

        msg!("Eligibility criteria set for study: {}", study_id);
        msg!("Requires ZK proof: {}", requires_zk);
        msg!("Criteria stored successfully");

        Ok(())
    }
}

impl<'info> VerifyEligibility<'info> {
    pub fn verify_eligibility(
        &mut self,
        study_id: u64,
        participant_info: ParticipantInfo,
    ) -> Result<bool> {
        let study = &self.study;
        
        if !study.has_eligibility_criteria {
            msg!("Study has no eligibility criteria - all participants eligible");
            return Ok(true);
        }

        let criteria: EligibilityCriteria = EligibilityCriteria::try_from_slice(&study.eligibility_criteria)
            .map_err(|_| RecruSearchError::InvalidParameterValue)?;

        msg!("Verifying eligibility for study: {}", study_id);

        if let Some(min_age) = criteria.min_age {
            if participant_info.age < min_age {
                msg!("Participant age {} is below minimum required age {}", participant_info.age, min_age);
                return Ok(false);
            }
        }

        if let Some(max_age) = criteria.max_age {
            if participant_info.age > max_age {
                msg!("Participant age {} is above maximum allowed age {}", participant_info.age, max_age);
                return Ok(false);
            }
        }

        if let Some(required_gender) = &criteria.gender {
            if participant_info.gender.to_lowercase() != required_gender.to_lowercase() {
                msg!("Participant gender '{}' does not match required gender '{}'", 
                     participant_info.gender, required_gender);
                return Ok(false);
            }
        }

        if let Some(required_location) = &criteria.location {
            if participant_info.location.to_lowercase() != required_location.to_lowercase() {
                msg!("Participant location '{}' does not match required location '{}'", 
                     participant_info.location, required_location);
                return Ok(false);
            }
        }

        if let Some(required_education) = &criteria.education_level {
            if participant_info.education_level.to_lowercase() != required_education.to_lowercase() {
                msg!("Participant education '{}' does not match required education '{}'", 
                     participant_info.education_level, required_education);
                return Ok(false);
            }
        }

        if let Some(required_employment) = &criteria.employment_status {
            if participant_info.employment_status.to_lowercase() != required_employment.to_lowercase() {
                msg!("Participant employment '{}' does not match required employment '{}'", 
                     participant_info.employment_status, required_employment);
                return Ok(false);
            }
        }

        if let Some(excluded_conditions) = &criteria.medical_conditions {
            for condition in excluded_conditions {
                if participant_info.medical_conditions.iter().any(|c| c.to_lowercase() == condition.to_lowercase()) {
                    msg!("Participant has excluded medical condition: {}", condition);
                    return Ok(false);
                }
            }
        }

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

        msg!("Participant meets all eligibility criteria for study: {}", study_id);
        Ok(true)
    }
}

impl<'info> VerifyEligibilityWithZK<'info> {
    pub fn verify_eligibility_with_zk(
        &mut self,
        study_id: u64,
        participant_info: ParticipantInfo,
        zk_proof: EligibilityZKProof,
    ) -> Result<bool> {
        let study = &self.study;
        
        // TODO: Implement actual ZK proof verification
        require!(zk_proof.proof.len() > 0, RecruSearchError::ZKProofValidationFailed);
        require!(zk_proof.public_inputs.len() > 0, RecruSearchError::ZKProofValidationFailed);
        require!(zk_proof.verification_key.len() > 0, RecruSearchError::ZKProofValidationFailed);

        msg!("ZK proof eligibility check for study: {} (Title: {})", study_id, study.title);

        require!(study.has_eligibility_criteria, RecruSearchError::NoEligibilityCriteria);

        let is_eligible = self.verify_zk_proof_placeholder(&zk_proof, &participant_info)?;

        if is_eligible {
            msg!("ZK proof verification successful for study: {}", study_id);
        } else {
            msg!("ZK proof verification failed for study: {}", study_id);
        }

        Ok(is_eligible)
    }

    fn verify_zk_proof_placeholder(
        &self,
        _proof: &EligibilityZKProof,
        _participant_info: &ParticipantInfo,
    ) -> Result<bool> {
        // TODO: Implement actual ZK proof verification logic
        msg!("ZK proof verification placeholder - assuming valid");
        Ok(true)
    }
}