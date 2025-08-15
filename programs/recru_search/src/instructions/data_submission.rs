use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::CreateV1CpiBuilder,
    types::{Attribute, Attributes, DataState, PluginAuthorityPair},
};

use crate::state::*;

// Data submission - allows participants to submit encrypted research data

#[derive(Accounts)]
pub struct SubmitData<'info> {
    // Study account for data submission
    #[account(
        seeds = [b"study", study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published || study.status == StudyStatus::Active @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    // Consent account - verifies participant enrollment
    #[account(
        seeds = [
            b"consent",
            study.key().as_ref(),
            participant.key().as_ref()
        ],
        bump = consent.bump,
        constraint = !consent.is_revoked @ RecruSearchError::ConsentRevoked,
        constraint = consent.participant == participant.key() @ RecruSearchError::UnauthorizedParticipant
    )]
    pub consent: Account<'info, ConsentAccount>,

    // Submission account - stores encrypted data metadata
    #[account(
        init,
        payer = participant,
        space = 8 + SubmissionAccount::INIT_SPACE,
        seeds = [
            b"submission",
            study.key().as_ref(),
            participant.key().as_ref()
        ],
        bump
    )]
    pub submission: Account<'info, SubmissionAccount>,

    // Participant submitting data
    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// Completion NFT minting - rewards participants for study completion

#[derive(Accounts)]
pub struct MintCompletionNFT<'info> {
    // Study account for completion tracking
    #[account(
        mut,
        seeds = [b"study", study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Active || study.status == StudyStatus::Closed @ RecruSearchError::InvalidStudyState,
        constraint = study.completed_count < study.max_participants @ RecruSearchError::StudyFull
    )]
    pub study: Account<'info, StudyAccount>,

    // Submission account - verifies data was submitted
    #[account(
        mut,
        seeds = [
            b"submission",
            study.key().as_ref(),
            participant.key().as_ref()
        ],
        bump = submission.bump,
        constraint = !submission.reward_distributed @ RecruSearchError::InvalidParameterValue,
        constraint = submission.completion_nft_mint.is_none() @ RecruSearchError::AlreadySubmitted
    )]
    pub submission: Account<'info, SubmissionAccount>,

    /// CHECK: This is the asset account that will be used to mint the completion NFT
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,


    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub system_program: Program<'info, System>,

    /// CHECK: This is the MPL Core program ID which is verified by the address constraint
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> SubmitData<'info> {
    // Submits encrypted research data with IPFS CID
    pub fn submit_data(
        &mut self,
        encrypted_data_hash: [u8; 32],
        ipfs_cid: String,
        bumps: &SubmitDataBumps,
    ) -> Result<()> {
        let study = &self.study;
        let clock = Clock::get()?;

        // Basic IPFS CID validation (length only)
        require!(
            ipfs_cid.len() >= 10 && ipfs_cid.len() <= 100,
            RecruSearchError::InvalidIPFSCID
        );

        // Validate data collection period
        require!(
            clock.unix_timestamp <= study.data_collection_end,
            RecruSearchError::InvalidDataCollectionPeriod
        );

        // Initialize submission account
        let submission = &mut self.submission;
        submission.participant = self.participant.key();
        submission.study = study.key();
        submission.encrypted_data_hash = encrypted_data_hash;
        submission.ipfs_cid = ipfs_cid.clone();
        submission.submission_timestamp = clock.unix_timestamp;
        submission.reward_distributed = false;
        submission.is_verified = false;
        submission.completion_nft_mint = None;
        submission.bump = bumps.submission;

        // Log submission details
        msg!("Data submitted successfully");
        msg!("Participant: {}", self.participant.key());
        msg!("Study: {}", study.study_id);
        msg!("IPFS CID: {}", ipfs_cid);
        msg!("Submission timestamp: {}", clock.unix_timestamp);

        // Emit data submitted event
        emit!(DataSubmitted {
            study_id: study.study_id,
            participant: self.participant.key(),
            ipfs_cid: ipfs_cid.clone(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}

impl<'info> MintCompletionNFT<'info> {
    // Mints completion NFT as reward for study participation
    pub fn mint_completion_nft(&mut self) -> Result<()> {
        let study = &self.study;
        
        // Extract submission data before mutable borrow
        let submission_timestamp = self.submission.submission_timestamp;
        
        // Use simple static metadata URI for template image
        let metadata_uri = COMPLETION_NFT_TEMPLATE_IMAGE.to_string();
        
        msg!("Creating Completion NFT with MPL Core attributes");
        
        // Mint the completion NFT with MPL Core attributes
        CreateV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(None)
            .authority(Some(&self.participant.to_account_info()))
            .payer(&self.participant.to_account_info())
            .owner(Some(&self.participant.to_account_info()))
            .update_authority(Some(&self.participant.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .data_state(DataState::AccountState)
            .name(format!("RecruSearch Completion #{}", study.study_id))
            .uri(metadata_uri)
            .plugins(vec![PluginAuthorityPair {
                plugin: mpl_core::types::Plugin::Attributes(Attributes { 
                    attribute_list: vec![
                        Attribute { 
                            key: "Study ID".to_string(), 
                            value: study.study_id.to_string() 
                        },
                        Attribute { 
                            key: "Study Title".to_string(), 
                            value: study.title.clone()
                        },
                        Attribute { 
                            key: "Completion Date".to_string(), 
                            value: Clock::get()?.unix_timestamp.to_string()
                        },
                        Attribute { 
                            key: "Type".to_string(), 
                            value: "Completion NFT".to_string() 
                        },
                        Attribute { 
                            key: "Platform".to_string(), 
                            value: "RecruSearch".to_string() 
                        },
                        Attribute { 
                            key: "Researcher".to_string(), 
                            value: study.researcher.to_string()
                        },
                        Attribute { 
                            key: "Submission Timestamp".to_string(), 
                            value: submission_timestamp.to_string()
                        },
                        Attribute { 
                            key: "Achievement".to_string(), 
                            value: "Research Participant".to_string()
                        }
                    ]
                }), 
                authority: None
            }])
            .invoke()?;

        // Update submission with NFT mint
        let submission = &mut self.submission;
        submission.completion_nft_mint = Some(self.asset.key());

        let study_id = study.study_id;
        let study = &mut self.study;
        study.completed_count = study.completed_count.saturating_add(1);

       
        msg!("SUCCESS: Completion NFT minted for participant: {}", self.participant.key());
        msg!("Completion NFT mint: {}", self.asset.key());
        msg!("Study ID: {}", study_id);
        msg!("Submission timestamp: {}", submission_timestamp);

        // Emit completion NFT minted event
        emit!(CompletionNFTMinted {
            study_id: study_id,
            participant: self.participant.key(),
            completion_nft_mint: self.asset.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}