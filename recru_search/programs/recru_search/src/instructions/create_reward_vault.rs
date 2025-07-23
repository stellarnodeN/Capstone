use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use crate::state::{
    study::{StudyAccount, StudyStatus},
    reward_vault::RewardVault,
};
use crate::error::RecruSearchError;

// Seed prefixes for account PDAs
const STUDY_SEED_PREFIX: &str = "study";
const VAULT_SEED_PREFIX: &str = "vault";

// Account validation struct for creating reward vaults
// This defines what accounts must be provided when a researcher creates a token vault for participant rewards
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct CreateRewardVault<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED_PREFIX.as_bytes(), researcher.key().as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump,
        constraint = study_account.researcher == researcher.key() @ RecruSearchError::UnauthorizedAccess
    )]
    pub study_account: Account<'info, StudyAccount>,

    #[account(
        init,
        payer = researcher,
        space = 8 + RewardVault::INIT_SPACE,
        seeds = [VAULT_SEED_PREFIX.as_bytes(), study_account.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, RewardVault>,

    /// Token account that will hold the reward tokens (must be owned by the vault)
    #[account(
        mut,
        constraint = vault_token_account.mint == reward_mint.key() @ RecruSearchError::InvalidTokenMint,
        constraint = vault_token_account.owner == reward_vault.key() @ RecruSearchError::InvalidTokenAccount
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// The mint of the token to be used for rewards (e.g., USDC, custom token)
    pub reward_mint: Account<'info, Mint>,

    /// Researcher's token account (source of initial deposit)
    #[account(
        mut,
        constraint = researcher_token_account.mint == reward_mint.key() @ RecruSearchError::InvalidTokenMint,
        constraint = researcher_token_account.owner == researcher.key() @ RecruSearchError::InvalidTokenAccount
    )]
    pub researcher_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CreateRewardVault<'info> {
    pub fn create_reward_vault(
        &mut self,
        study_id: u64,
        initial_deposit: u64,
        bumps: &CreateRewardVaultBumps,
    ) -> Result<()> {
        let study_account = &mut self.study_account;
        let reward_vault = &mut self.reward_vault;
        let clock = Clock::get()?;

        // Validate study status allows vault creation
        require!(
            matches!(study_account.status, StudyStatus::Draft | StudyStatus::Published),
            RecruSearchError::InvalidStatusTransition
        );

        // Validate initial deposit covers the expected rewards
        let total_reward_needed = (study_account.max_participants as u64)
            .checked_mul(study_account.reward_amount_per_participant)
            .ok_or(RecruSearchError::MathOverflow)?;
        
        require!(
            initial_deposit >= total_reward_needed,
            RecruSearchError::InsufficientRewardDeposit
        );

        // Initialize reward vault
        reward_vault.study_account = study_account.key();
        reward_vault.vault_token_account = self.vault_token_account.key();
        reward_vault.reward_mint = self.reward_mint.key();
        reward_vault.total_deposited = initial_deposit;
        reward_vault.participants_rewarded = 0;
        reward_vault.created_at = clock.unix_timestamp;
        reward_vault.bump = bumps.reward_vault;

        // Transfer initial deposit from researcher to vault
        let transfer_ix = Transfer {
            from: self.researcher_token_account.to_account_info(),
            to: self.vault_token_account.to_account_info(),
            authority: self.researcher.to_account_info(),
        };
        
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                transfer_ix,
            ),
            initial_deposit,
        )?;

        msg!(
            "Reward vault created for study {}: {} tokens deposited",
            study_id,
            initial_deposit
        );

        Ok(())
    }
} 