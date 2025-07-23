use anchor_lang::prelude::*;
use crate::state::{
    study::{StudyAccount, StudyStatus},
    consent_nft::ConsentNFTAccount,
    submission::SubmissionAccount,
};
use crate::error::RecruSearchError;

// Seed prefixes for account PDAs
const STUDY_SEED_PREFIX: &str = "study";
const CONSENT_SEED_PREFIX: &str = "consent";
const SUBMISSION_SEED_PREFIX: &str = "submission";

// Account validation struct for submitting encrypted research data
// This defines what accounts must be provided when a participant submits their data
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct SubmitEncryptedData<'info> {
    #[account(
        seeds = [STUDY_SEED_PREFIX.as_bytes(), study_account.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump
    )]
    pub study_account: Account<'info, StudyAccount>,

    #[account(
        seeds = [CONSENT_SEED_PREFIX.as_bytes(), study_account.key().as_ref(), participant.key().as_ref()],
        bump = consent_nft_account.bump,
        constraint = !consent_nft_account.is_revoked @ RecruSearchError::InvalidOrRevokedConsent
    )]
    pub consent_nft_account: Account<'info, ConsentNFTAccount>,

    #[account(
        init,
        payer = participant,
        space = 8 + SubmissionAccount::INIT_SPACE,
        seeds = [SUBMISSION_SEED_PREFIX.as_bytes(), study_account.key().as_ref(), participant.key().as_ref()],
        bump
    )]
    pub submission_account: Account<'info, SubmissionAccount>,

    #[account(mut)]
    pub participant: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> SubmitEncryptedData<'info> {
    pub fn submit_encrypted_data(
        &mut self,
        study_id: u64,
        encrypted_data_hash: [u8; 32],      // Hash of the encrypted data for verification
        ipfs_cid: String,                   // IPFS content identifier where data is stored
        bumps: &SubmitEncryptedDataBumps,
    ) -> Result<()> {
        let submission_account = &mut self.submission_account;
        let study_account = &self.study_account;
        let clock = Clock::get()?;

        // Validate that data collection is still open for this study
        require!(
            matches!(study_account.status, StudyStatus::Published | StudyStatus::Active) 
                && clock.unix_timestamp <= study_account.data_collection_end,
            RecruSearchError::DataCollectionClosed
        );

        // Validate IPFS CID length (standard IPFS CIDv1 is around 59 characters)
        require!(
            ipfs_cid.len() >= 10 && ipfs_cid.len() <= 100,
            RecruSearchError::InvalidDataFormat
        );

        // Validate encrypted data hash is not empty (must contain actual data)
        require!(
            encrypted_data_hash != [0u8; 32],
            RecruSearchError::InvalidDataFormat
        );

        // Initialize submission account with the provided data
        submission_account.study_id = study_id;
        submission_account.participant = self.participant.key();
        submission_account.consent_nft = self.consent_nft_account.key();
        submission_account.encrypted_data_hash = encrypted_data_hash;
        submission_account.ipfs_cid = ipfs_cid;
        submission_account.submission_timestamp = clock.unix_timestamp;
        submission_account.reward_claimed = false; // Will be set to true when reward is distributed
        submission_account.completion_nft = None; // Will be set when completion NFT is minted
        submission_account.bump = bumps.submission_account;

        msg!(
            "Encrypted data submitted for participant {} in study {}",
            self.participant.key(),
            study_id
        );

        Ok(())
    }
} 