use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::CreateV2CpiBuilder,
};
use crate::state::{StudyAccount, StudyStatus, SubmissionAccount, ConsentAccount, RecruSearchError};

#[derive(Accounts)]
pub struct SubmitData<'info> {
    #[account(
        seeds = [b"study", study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published || study.status == StudyStatus::Active @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

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

    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintCompletionNFT<'info> {
    #[account(
        seeds = [b"study", study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Active || study.status == StudyStatus::Closed @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

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

    /// CHECK: Asset account (the completion NFT itself)
    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub system_program: Program<'info, System>,

    /// CHECK: Metaplex Core Program for NFT creation
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> SubmitData<'info> {
    pub fn submit_data(
        &mut self,
        encrypted_data_hash: [u8; 32],
        ipfs_cid: String,
        bumps: &SubmitDataBumps,
    ) -> Result<()> {
        let study = &self.study;
        let clock = Clock::get()?;

        // Validate IPFS CID length
        require!(ipfs_cid.len() > 0 && ipfs_cid.len() <= 100, RecruSearchError::InvalidIPFSCID);

        // Validate study is still accepting submissions
        require!(
            clock.unix_timestamp <= study.data_collection_end,
            RecruSearchError::InvalidDataCollectionPeriod
        );

        // Initialize submission
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

        msg!("Data submitted successfully");
        msg!("Participant: {}", self.participant.key());
        msg!("Study: {}", study.study_id);
        msg!("IPFS CID: {}", ipfs_cid);
        msg!("Submission timestamp: {}", clock.unix_timestamp);

        Ok(())
    }
}

impl<'info> MintCompletionNFT<'info> {
    pub fn mint_completion_nft(&mut self) -> Result<()> {
        let study = &self.study;
        let submission = &mut self.submission;

        // Create Completion NFT using Metaplex Core
        let metadata_uri = format!("https://api.recrusearch.com/completion/{}/{}", study.study_id, submission.participant);
        let completion_nft_name = format!("RecruSearch Completion #{}", study.study_id);
        
        // Use CreateV2CpiBuilder as per Metaplex documentation
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

        // Update submission with completion NFT reference
        submission.completion_nft_mint = Some(self.asset.key());

        msg!("SUCCESS: Completion NFT minted for participant: {}", self.participant.key());
        msg!("Completion NFT mint: {}", self.asset.key());
        msg!("Study ID: {}", study.study_id);
        msg!("Submission timestamp: {}", submission.submission_timestamp);

        Ok(())
    }
}