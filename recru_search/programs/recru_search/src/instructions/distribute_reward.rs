use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer, MintTo};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{
    study::StudyAccount,
    reward_vault::RewardVault,
    submission::SubmissionAccount,
    completion_nft::CompletionNFTAccount,
};
use crate::error::RecruSearchError;

// Seed prefixes for account PDAs
const STUDY_SEED_PREFIX: &str = "study";
const VAULT_SEED_PREFIX: &str = "vault";
const SUBMISSION_SEED_PREFIX: &str = "submission";
const COMPLETION_SEED_PREFIX: &str = "completion";

// Account validation struct for distributing rewards to participants
// This defines what accounts must be provided when paying participants and minting completion certificates
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct DistributeReward<'info> {
    #[account(
        seeds = [STUDY_SEED_PREFIX.as_bytes(), study_account.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump
    )]
    pub study_account: Account<'info, StudyAccount>,

    #[account(
        mut,
        seeds = [VAULT_SEED_PREFIX.as_bytes(), study_account.key().as_ref()],
        bump = reward_vault.bump
    )]
    pub reward_vault: Account<'info, RewardVault>,

    #[account(
        mut,
        seeds = [SUBMISSION_SEED_PREFIX.as_bytes(), study_account.key().as_ref(), participant.key().as_ref()],
        bump = submission_account.bump,
        constraint = !submission_account.reward_claimed @ RecruSearchError::RewardAlreadyClaimed
    )]
    pub submission_account: Account<'info, SubmissionAccount>,

    #[account(
        init,
        payer = participant,
        space = 8 + CompletionNFTAccount::INIT_SPACE,
        seeds = [COMPLETION_SEED_PREFIX.as_bytes(), study_account.key().as_ref(), participant.key().as_ref()],
        bump
    )]
    pub completion_nft_account: Account<'info, CompletionNFTAccount>,

    /// Token account holding the reward tokens (source of payment)
    #[account(
        mut,
        address = reward_vault.vault_token_account
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// Participant's token account where rewards will be sent
    #[account(
        mut,
        token::mint = reward_vault.reward_mint,
        token::authority = participant
    )]
    pub participant_token_account: Account<'info, TokenAccount>,

    /// The completion certificate NFT mint account
    #[account(
        init,
        payer = participant,
        mint::decimals = 0,
        mint::authority = completion_nft_account,
        mint::freeze_authority = completion_nft_account
    )]
    pub completion_nft_mint: Account<'info, Mint>,

    /// Participant's token account where the completion NFT will be minted
    #[account(
        init,
        payer = participant,
        associated_token::mint = completion_nft_mint,
        associated_token::authority = participant
    )]
    pub participant_completion_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub participant: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> DistributeReward<'info> {
    pub fn distribute_reward(
        &mut self,
        study_id: u64,
        study_title: String,                 // Study name for completion certificate
        study_type: String,                  // Research category for certificate  
        study_duration_days: String,         // Study duration for certificate display
        image_uri: String,                   // Image URI for certificate artwork
        bumps: &DistributeRewardBumps,
    ) -> Result<()> {
        let submission_account = &mut self.submission_account;
        let reward_vault = &mut self.reward_vault;
        let completion_nft_account = &mut self.completion_nft_account;
        let study_account = &self.study_account;
        let clock = Clock::get()?;

        // Validate that the vault has sufficient tokens for this reward
        let reward_amount = study_account.reward_amount_per_participant;
        // Note: We can't easily check token balance from account info, so we trust the transfer will fail if insufficient

        // Validate metadata input lengths to prevent oversized data
        require!(
            study_title.len() <= 100,
            RecruSearchError::InvalidDataFormat
        );
        require!(
            study_type.len() <= 50,
            RecruSearchError::InvalidDataFormat
        );
        require!(
            study_duration_days.len() <= 20,
            RecruSearchError::InvalidDataFormat
        );
        require!(
            image_uri.len() <= 200,
            RecruSearchError::InvalidDataFormat
        );

        // Transfer reward tokens from vault to participant
        let study_key = study_account.key();
        let vault_seeds = &[
            VAULT_SEED_PREFIX.as_bytes(),
            study_key.as_ref(),
            &[reward_vault.bump],
        ];
        let signer_seeds = &[&vault_seeds[..]];

        let transfer_ix = Transfer {
            from: self.vault_token_account.to_account_info(),
            to: self.participant_token_account.to_account_info(),
            authority: reward_vault.to_account_info(),
        };
        
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_ix,
                signer_seeds,
            ),
            reward_amount,
        )?;

        // Initialize completion NFT account with enhanced metadata
        completion_nft_account.study_id = study_id;
        completion_nft_account.participant = self.participant.key();
        completion_nft_account.submission_account = submission_account.key();
        completion_nft_account.completion_timestamp = clock.unix_timestamp;
        completion_nft_account.reward_amount = reward_amount;
        
        // Enhanced metadata fields for Metaplex compatibility
        completion_nft_account.study_title = study_title;
        completion_nft_account.study_type = study_type;
        completion_nft_account.study_duration_days = study_duration_days;
        completion_nft_account.image_uri = image_uri;
        
        // Metadata URI will be populated by frontend after uploading JSON to IPFS
        completion_nft_account.metadata_uri = "".to_string();
        
        completion_nft_account.bump = bumps.completion_nft_account;

        // Mint completion certificate NFT to participant
        let participant_key = self.participant.key();
        let completion_nft_seeds = &[
            COMPLETION_SEED_PREFIX.as_bytes(),
            study_key.as_ref(),
            participant_key.as_ref(),
            &[bumps.completion_nft_account],
        ];
        let completion_signer_seeds = &[&completion_nft_seeds[..]];

        let mint_ix = MintTo {
            mint: self.completion_nft_mint.to_account_info(),
            to: self.participant_completion_token_account.to_account_info(),
            authority: completion_nft_account.to_account_info(),
        };
        
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                mint_ix,
                completion_signer_seeds,
            ),
            1, // Mint exactly 1 completion certificate
        )?;

        // Update submission account to mark reward as claimed
        submission_account.reward_claimed = true;
        submission_account.completion_nft = Some(self.completion_nft_mint.key());

        // Update vault statistics
        reward_vault.participants_rewarded = reward_vault.participants_rewarded
            .checked_add(1)
            .ok_or(RecruSearchError::MathOverflow)?;

        msg!(
            "Reward distributed to participant {} for study '{}' (ID: {}) - Amount: {} tokens",
            self.participant.key(),
            completion_nft_account.study_title,
            study_id,
            reward_amount
        );

        Ok(())
    }
} 