use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::{BurnV1, BurnV1InstructionArgs},
};
use crate::state::{
    study::{StudyAccount, StudyStatus},
    consent_nft::ConsentNFTAccount,
};
use crate::error::RecruSearchError;

// Seed prefixes for account PDAs
const STUDY_SEED_PREFIX: &str = "study";
const CONSENT_SEED_PREFIX: &str = "consent";

// Account validation struct for revoking consent
// This allows participants to withdraw from studies before data submission
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct RevokeConsent<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED_PREFIX.as_bytes(), study_account.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump
    )]
    pub study_account: Account<'info, StudyAccount>,

    #[account(
        mut,
        seeds = [CONSENT_SEED_PREFIX.as_bytes(), study_account.key().as_ref(), participant.key().as_ref()],
        bump = consent_nft_account.bump,
        constraint = consent_nft_account.participant == participant.key() @ RecruSearchError::UnauthorizedAccess,
        constraint = !consent_nft_account.is_revoked @ RecruSearchError::ConsentAlreadyRevoked
    )]
    pub consent_nft_account: Account<'info, ConsentNFTAccount>,

    /// Core NFT Asset account to be burned
    /// CHECK: This account will be validated by the Core program
    #[account(mut)]
    pub core_asset: UncheckedAccount<'info>,

    #[account(mut)]
    pub participant: Signer<'info>,

    pub system_program: Program<'info, System>,
    
    /// CHECK: Metaplex Core Program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> RevokeConsent<'info> {
    pub fn revoke_consent(&mut self, study_id: u64) -> Result<()> {
        let study_account = &mut self.study_account;
        let consent_nft_account = &mut self.consent_nft_account;
        let clock = Clock::get()?;

        // Validate study is still in a state where consent can be revoked
        // Participants can revoke during Published or Active phases, but not after Closed
        require!(
            matches!(study_account.status, StudyStatus::Published | StudyStatus::Active),
            RecruSearchError::InvalidStatusTransition
        );

        // Check if participant has already submitted data
        // TODO: This would require checking if a SubmissionAccount exists
        // For now, we'll allow revocation during any phase before closure

        // Mark consent as revoked
        consent_nft_account.is_revoked = true;

        // Burn the Core NFT Asset to make revocation permanent and visible
        let burn_ix = BurnV1 {
            asset: self.core_asset.key(),
            collection: None,
            payer: self.participant.key(),
            authority: None,
            system_program: Some(self.system_program.key()),
            log_wrapper: None,
        }.instruction(BurnV1InstructionArgs {
            compression_proof: None,
        });

        anchor_lang::solana_program::program::invoke(
            &burn_ix,
            &[
                self.core_asset.to_account_info(),
                self.participant.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        // Decrement study enrollment count
        study_account.enrolled_count = study_account.enrolled_count
            .checked_sub(1)
            .ok_or(RecruSearchError::MathOverflow)?;

        msg!(
            "Consent revoked for participant {} in study '{}' (ID: {}) at timestamp {}",
            self.participant.key(),
            consent_nft_account.study_title,
            study_id,
            clock.unix_timestamp
        );

        Ok(())
    }
} 