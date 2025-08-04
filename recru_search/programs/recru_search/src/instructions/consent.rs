use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::{CreateV2CpiBuilder, BurnV1CpiBuilder},
};
use crate::state::{StudyAccount, StudyStatus, ConsentAccount, SubmissionAccount, RecruSearchError};
use crate::instructions::eligibility_criteria::{EligibilityCriteria, ParticipantInfo};

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct MintConsentNFT<'info> {
    #[account(
        mut,
        seeds = [b"study", study.researcher.key().as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::StudyNotPublished,
        constraint = study.enrolled_count < study.max_participants @ RecruSearchError::StudyFull
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        init,
        payer = participant,
        space = 8 + ConsentAccount::INIT_SPACE,
        seeds = [
            b"consent",
            study.key().as_ref(),
            participant.key().as_ref()
        ],
        bump
    )]
    pub consent: Account<'info, ConsentAccount>,

    /// CHECK: This is the asset account that will be used to mint the NFT
    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub system_program: Program<'info, System>,

    /// CHECK: This is the MPL Core program ID which is verified by the address constraint
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RevokeConsent<'info> {
    #[account(
        mut,
        seeds = [
            b"consent",
            consent.study.as_ref(),
            participant.key().as_ref()
        ],
        bump = consent.bump,
        constraint = consent.participant == participant.key() @ RecruSearchError::UnauthorizedParticipant,
        constraint = !consent.is_revoked @ RecruSearchError::ConsentRevoked
    )]
    pub consent: Account<'info, ConsentAccount>,

    /// CHECK: This is the asset account that will be used to burn the NFT
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    #[account(mut)]
    pub participant: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        seeds = [
            b"submission",
            consent.study.as_ref(),
            participant.key().as_ref()
        ],
        bump
    )]
    pub submission: Option<Account<'info, SubmissionAccount>>,

    /// CHECK: This is the MPL Core program ID which is verified by the address constraint
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> MintConsentNFT<'info> {
    pub fn mint_consent_nft(&mut self, _study_id: u64, eligibility_proof: Vec<u8>) -> Result<()> {
        require!(eligibility_proof.len() > 0, RecruSearchError::InvalidEligibilityProof);
        
        let study = &self.study;
        let clock = Clock::get()?;
        
        require!(
            clock.unix_timestamp >= study.enrollment_start && 
            clock.unix_timestamp <= study.enrollment_end,
            RecruSearchError::InvalidEnrollmentPeriod
        );

        if study.has_eligibility_criteria {
            let participant_info: ParticipantInfo = ParticipantInfo::try_from_slice(&eligibility_proof)
                .map_err(|_| RecruSearchError::InvalidEligibilityProof)?;
            
            let is_eligible = self.verify_participant_eligibility(&participant_info)?;
            require!(is_eligible, RecruSearchError::ParticipantNotEligible);
            
            msg!("Participant eligibility verified successfully");
        } else {
            msg!("Study has no eligibility criteria - skipping verification");
        }

        let consent = &mut self.consent;
        consent.participant = self.participant.key();
        consent.study = study.key();
        consent.timestamp = clock.unix_timestamp;
        consent.is_revoked = false;
        consent.revocation_timestamp = None;
        consent.eligibility_proof = eligibility_proof;
        consent.nft_mint = Some(self.asset.key());

        // Increment study enrollment count
        let study = &mut self.study;
        study.enrolled_count = study.enrolled_count.saturating_add(1);

        let metadata_uri = "ipfs://bafkreibvcklkbj43f7ljb7ade2jcldybdxgb4vu7kdhbyedkslyq5g4sf4".to_string();
        let consent_nft_name = format!("RecruSearch Consent #{}", study.study_id);
        let consent_nft_symbol = "RSCONSENT";
        let consent_nft_description = format!("Consent NFT for RecruSearch study #{} - This NFT represents the participant's consent to participate in this research study.", study.study_id);
        
        msg!("Creating Consent NFT with symbol: {} and description: {}", consent_nft_symbol, consent_nft_description);
        
        CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(None)
            .authority(None)
            .payer(&self.participant.to_account_info())
            .owner(Some(&self.participant.to_account_info()))
            .update_authority(Some(&self.participant.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name(consent_nft_name)
            .uri(metadata_uri)
            .invoke()?;

        msg!("SUCCESS: Consent NFT minted for participant: {}", self.participant.key());
        msg!("Consent NFT mint: {}", self.asset.key());
        msg!("Study ID: {}", study.study_id);

        Ok(())
    }

    fn verify_participant_eligibility(&self, participant_info: &ParticipantInfo) -> Result<bool> {
        let study = &self.study;
        
        if !study.has_eligibility_criteria {
            return Ok(true);
        }

        let criteria: EligibilityCriteria = EligibilityCriteria::try_from_slice(&study.eligibility_criteria)
            .map_err(|_| RecruSearchError::InvalidParameterValue)?;

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

        Ok(true)
    }
}

impl<'info> RevokeConsent<'info> {
    pub fn revoke_consent(&mut self) -> Result<()> {
        if let Some(_submission) = &self.submission {
            msg!("ERROR: Cannot revoke consent after data submission");
            return Err(RecruSearchError::AlreadySubmitted.into());
        }

        let consent = &mut self.consent;
        consent.is_revoked = true;
        consent.revocation_timestamp = Some(Clock::get()?.unix_timestamp);

        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .authority(Some(&self.participant.to_account_info()))
            .invoke()?;

        msg!("SUCCESS: Consent revoked and NFT burned for participant: {}", self.participant.key());
        msg!("Burned NFT: {}", self.asset.key());
        
        Ok(())
    }
}