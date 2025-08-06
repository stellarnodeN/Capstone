use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::CreateV2CpiBuilder,
};
use crate::state::{StudyAccount, StudyStatus, SubmissionAccount, ConsentAccount, RecruSearchError};
use crate::state::constants::{
    MIN_IPFS_CID_LENGTH, MAX_IPFS_CID_LENGTH, IPFS_CID_V0_PREFIX, IPFS_CID_V1_PREFIX,
    DEFAULT_COMPLETION_NFT_METADATA_URI, COMPLETION_NFT_SYMBOL
};
use crate::state::events::{DataSubmitted, CompletionNFTMinted};

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

    ///  This is the asset account that will be used to mint the completion NFT
    #[account(mut)]
    pub asset: Signer<'info>,

    // Participant receiving completion NFT
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

        // Validate IPFS CID format and length
        require!(
            ipfs_cid.len() >= MIN_IPFS_CID_LENGTH && ipfs_cid.len() <= MAX_IPFS_CID_LENGTH,
            RecruSearchError::InvalidIPFSCID
        );
        require!(
            ipfs_cid.starts_with(IPFS_CID_V0_PREFIX) || ipfs_cid.starts_with(IPFS_CID_V1_PREFIX),
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
        let submission = &mut self.submission;

        // Create completion NFT metadata
        let metadata_uri = DEFAULT_COMPLETION_NFT_METADATA_URI.to_string();
        let completion_nft_name = format!("RecruSearch Completion #{}", study.study_id);
        let completion_nft_symbol = COMPLETION_NFT_SYMBOL;
        let completion_nft_description = format!("Completion NFT for RecruSearch study #{} - This NFT represents successful completion of the research study.", study.study_id);
        
        msg!("Creating Completion NFT with symbol: {} and description: {}", completion_nft_symbol, completion_nft_description);
        
        // Mint the completion NFT
        CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(None)
            .authority(None)
            .payer(&self.participant.to_account_info())
            .owner(Some(&self.participant.to_account_info()))
            .update_authority(Some(&self.participant.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name(completion_nft_name)
            .uri(metadata_uri)
            .invoke()?;

        // Update submission with NFT mint
        submission.completion_nft_mint = Some(self.asset.key());

        // Update study completion count
        let study = &mut self.study;
        study.completed_count = study.completed_count.saturating_add(1);

        // Log completion details
        msg!("SUCCESS: Completion NFT minted for participant: {}", self.participant.key());
        msg!("Completion NFT mint: {}", self.asset.key());
        msg!("Study ID: {}", study.study_id);
        msg!("Submission timestamp: {}", submission.submission_timestamp);

        // Emit completion NFT minted event
        emit!(CompletionNFTMinted {
            study_id: study.study_id,
            participant: self.participant.key(),
            completion_nft_mint: self.asset.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}