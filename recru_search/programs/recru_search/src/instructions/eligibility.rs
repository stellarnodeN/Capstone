use anchor_lang::prelude::*;
use crate::state::study::{StudyAccount, StudyStatus};
use crate::error::RecruSearchError;
use crate::constants::*;

/// Set eligibility criteria for a study (User Story #2)
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct SetEligibilityCriteria<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status == StudyStatus::Draft @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

/// Verify participant eligibility for a study
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct VerifyEligibility<'info> {
    #[account(
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::StudyNotPublished
    )]
    pub study: Account<'info, StudyAccount>,

    /// CHECK: Participant being verified for eligibility
    pub participant: UncheckedAccount<'info>,
}

/// Check eligibility with ZK proof verification
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct VerifyEligibilityWithZK<'info> {
    #[account(
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::StudyNotPublished,
        constraint = study.requires_zk_proof @ RecruSearchError::ZKProofValidationFailed
    )]
    pub study: Account<'info, StudyAccount>,

    /// CHECK: Participant being verified for eligibility
    pub participant: UncheckedAccount<'info>,
}

// Eligibility criteria structure
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

// Participant information for eligibility checking
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

// ZK proof structure for privacy-preserving eligibility verification
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
        criteria: EligibilityCriteria,
    ) -> Result<()> {
        let study = &mut self.study;

        // If any criteria require ZK proofs, mark the study accordingly
        let requires_zk = criteria.min_age.is_some() || 
                         criteria.max_age.is_some() ||
                         criteria.medical_conditions.is_some();
        study.requires_zk_proof = requires_zk;

        msg!("Eligibility criteria set for study: {}", study_id);
        msg!("Requires ZK proof: {}", requires_zk);

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
        
        // For MVP, do basic eligibility check
        // TODO: Store eligibility criteria separately or expand StudyAccount
        msg!("Basic eligibility check for study: {}", study_id);

        // TODO: Implement full eligibility checking logic
        // For now, check basic age requirement if available
        if participant_info.age < 18 {
            msg!("Participant must be at least 18 years old");
            return Ok(false);
        }

        msg!("Participant is eligible for study: {}", study_id);
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
        // For now, do basic validation
        require!(zk_proof.proof.len() > 0, RecruSearchError::ZKProofValidationFailed);
        require!(zk_proof.public_inputs.len() > 0, RecruSearchError::ZKProofValidationFailed);
        require!(zk_proof.verification_key.len() > 0, RecruSearchError::ZKProofValidationFailed);

        // For MVP, simplified ZK proof verification
        msg!("ZK proof eligibility check for study: {}", study_id);

        // For sensitive criteria (age, medical conditions), use ZK proof verification
        // This is a placeholder - actual implementation would verify the ZK proof
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
        // This would typically involve:
        // 1. Verifying the proof against the verification key
        // 2. Checking that public inputs match the criteria
        // 3. Ensuring the proof proves knowledge of satisfying private inputs
        
        msg!("ZK proof verification placeholder - assuming valid");
        Ok(true)
    }
}