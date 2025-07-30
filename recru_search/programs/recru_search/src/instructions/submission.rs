use anchor_lang::prelude::*;
use crate::state::{
    study::{StudyAccount, StudyStatus},
    submission::SubmissionAccount,
    nft::ConsentAccount,
};
use crate::error::RecruSearchError;
use crate::constants::*;

#[derive(Accounts)]
pub struct SubmitData<'info> {
    #[account(
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.status == StudyStatus::Published || study.status == StudyStatus::Active @ RecruSearchError::InvalidStudyState
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        seeds = [
            CONSENT_SEED.as_bytes(),
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
            SUBMISSION_SEED.as_bytes(),
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
        submission.study_id = study.study_id;
        submission.consent_nft = self.consent.key();
        submission.encrypted_data_hash = encrypted_data_hash;
        submission.ipfs_cid = ipfs_cid.clone();
        submission.submission_timestamp = clock.unix_timestamp;
        submission.reward_claimed = false;
        submission.is_verified = false;
        submission.completion_nft = None;
        submission.bump = bumps.submission;

        msg!("Data submitted successfully");
        msg!("Participant: {}", self.participant.key());
        msg!("Study: {}", study.study_id);
        msg!("IPFS CID: {}", ipfs_cid);
        msg!("Submission timestamp: {}", clock.unix_timestamp);

        Ok(())
    }
}