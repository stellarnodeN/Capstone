use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::{CreateV1CpiBuilder, BurnV1CpiBuilder},
    types::{Attribute, Attributes, DataState, PluginAuthorityPair},
};
use crate::state::{StudyAccount, StudyStatus, ConsentAccount, SubmissionAccount, RecruSearchError, CONSENT_NFT_TEMPLATE_IMAGE};
use crate::instructions::eligibility_criteria::{EligibilityInfo, verify_participant_eligibility};
use crate::state::events::{ConsentNFTMinted,ConsentRevoked};

// Consent NFT minting - allows participants to enroll in studies

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct MintConsentNFT<'info> {
    // Study account to enroll in
    #[account(
        mut,
        seeds = [b"study", study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::StudyNotPublished,
        constraint = study.enrolled_count < study.max_participants @ RecruSearchError::StudyFull
    )]
    pub study: Account<'info, StudyAccount>,

    // Consent account - tracks participant enrollment
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
    pub asset: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub system_program: Program<'info, System>,

    /// CHECK: This is the MPL Core program ID which is verified by the address constraint
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}
// Consent revocation - allows participants to withdraw from studies
#[derive(Accounts)]
pub struct RevokeConsent<'info> {
    // Consent account to revoke
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

    // Study account for reference
    #[account(
        seeds = [b"study", study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump
    )]
    pub study: Account<'info, StudyAccount>,

    /// CHECK: This is the asset account that will be used to burn the NFT
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    // Participant revoking consent
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
    // Mints consent NFT and enrolls participant in study
    pub fn mint_consent_nft(&mut self, _study_id: u64, eligibility_proof: Vec<u8>) -> Result<()> {
        require!(eligibility_proof.len() > 0, RecruSearchError::InvalidEligibilityProof);
        
        let study = &self.study;
        let clock = Clock::get()?;
        
        // Validate enrollment period
        require!(
            clock.unix_timestamp >= study.enrollment_start && 
            clock.unix_timestamp <= study.enrollment_end,
            RecruSearchError::InvalidEnrollmentPeriod
        );
         // Verify eligibility if criteria are set
        if study.has_eligibility_criteria {
            let participant_info: EligibilityInfo = EligibilityInfo::try_from_slice(&eligibility_proof)
                .map_err(|_| RecruSearchError::InvalidEligibilityProof)?;
            
            
            let is_eligible = verify_participant_eligibility(&study.eligibility_criteria, &participant_info)?;
            require!(is_eligible, RecruSearchError::ParticipantNotEligible);
            
            msg!("Participant eligibility verified successfully");
        } else {
            msg!("Study has no eligibility criteria - skipping verification");
        }

        // Initialize consent account
        let consent = &mut self.consent;
        consent.participant = self.participant.key();
        consent.study = study.key();
        consent.timestamp = clock.unix_timestamp;
        consent.is_revoked = false;
        consent.revocation_timestamp = None;
        consent.eligibility_proof = eligibility_proof;
        consent.nft_mint = Some(self.asset.key());

        // Extract study data before borrowing mutably
        let study_id = study.study_id;
        let study_title = study.title.clone();
        let study_researcher = study.researcher;
        let study_has_eligibility = study.has_eligibility_criteria;
        let study = &mut self.study;
        study.enrolled_count = study.enrolled_count.saturating_add(1);
        
        // Use simple static metadata URI for template image
        let metadata_uri = CONSENT_NFT_TEMPLATE_IMAGE;
        
        msg!("Creating Consent NFT with MPL Core attributes");
        
        // Mint the consent NFT
        CreateV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(None)
            .authority(Some(&self.participant.to_account_info()))
            .payer(&self.participant.to_account_info())
            .owner(Some(&self.participant.to_account_info()))
            .update_authority(Some(&self.participant.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .data_state(DataState::AccountState)
            .name(format!("RecruSearch Consent #{}", study_id))
            .uri(metadata_uri.to_string())
            .plugins(vec![PluginAuthorityPair {
                plugin: mpl_core::types::Plugin::Attributes(Attributes { 
                    attribute_list: vec![
                        Attribute { 
                            key: "Study ID".to_string(), 
                            value: study.study_id.to_string() 
                        },
                        Attribute { 
                            key: "Study Title".to_string(), 
                            value: study_title.clone()
                        },
                        Attribute { 
                            key: "Consent Date".to_string(), 
                            value: clock.unix_timestamp.to_string()
                        },
                        Attribute { 
                            key: "Type".to_string(), 
                            value: "Consent NFT".to_string() 
                        },
                        Attribute { 
                            key: "Platform".to_string(), 
                            value: "RecruSearch".to_string() 
                        },
                        Attribute { 
                            key: "Researcher".to_string(), 
                            value: study_researcher.to_string()
                        },
                        Attribute { 
                            key: "Has Eligibility Criteria".to_string(), 
                            value: study_has_eligibility.to_string()
                        }
                    ]
                }), 
                authority: None
            }])
            .invoke()?;

        msg!("SUCCESS: Consent NFT minted for participant: {}", self.participant.key());
        msg!("Consent NFT mint: {}", self.asset.key());
        msg!("Study ID: {}", study_id);

        // Emit consent NFT minted event
        emit!(ConsentNFTMinted {
            study_id: study_id,
            participant: self.participant.key(),
            consent_nft_mint: self.asset.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}

impl<'info> RevokeConsent<'info> {
    // Revokes consent and marks NFT as revoked - prevents data submission
    pub fn revoke_consent(&mut self) -> Result<()> {
        // Prevent revocation after data submission
        if let Some(_submission) = &self.submission {
            msg!("ERROR: Cannot revoke consent after data submission");
            return Err(RecruSearchError::AlreadySubmitted.into());
        }

        let clock = Clock::get()?;

        // Mark consent as revoked
        let consent = &mut self.consent;
        consent.is_revoked = true;
        consent.revocation_timestamp = Some(clock.unix_timestamp);

        // Burn the consent NFT
        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .authority(Some(&self.participant.to_account_info()))
            .invoke()?;
        
        msg!("SUCCESS: Consent revoked and NFT burned for participant: {}", self.participant.key());
        msg!("Burned NFT: {}", self.asset.key());
        
        // Emit consent revoked event
        emit!(ConsentRevoked {
            study_id: self.study.study_id,
            participant: self.participant.key(),
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }
}