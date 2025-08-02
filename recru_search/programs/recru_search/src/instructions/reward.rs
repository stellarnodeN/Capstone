use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
    token::{transfer_checked, TransferChecked},
};
use crate::state::{StudyAccount, StudyStatus, SubmissionAccount, ConsentAccount, RewardVault, RecruSearchError};


#[derive(Accounts)]
pub struct DistributeReward<'info> {
    #[account(
        mut,
        seeds = [b"study", study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        mut,
        seeds = [b"vault", study.key().as_ref()],
        bump = reward_vault.bump,
        constraint = reward_vault.study == study.key() @ RecruSearchError::InvalidParameterValue
    )]
    pub reward_vault: Account<'info, RewardVault>,

    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = reward_vault,
        token::token_program = token_program,
        seeds = [b"vault_token", reward_vault.key().as_ref()],
        bump,
        constraint = vault_token_account.key() == reward_vault.key() @ RecruSearchError::InvalidParameterValue
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [
            b"consent",
            study.key().as_ref(),
            participant.key().as_ref()
        ],
        bump = consent.bump,
        constraint = !consent.is_revoked @ RecruSearchError::ConsentRevoked
    )]
    pub consent: Account<'info, ConsentAccount>,

    #[account(
        mut,
        seeds = [
            b"submission",
            study.key().as_ref(),
            participant.key().as_ref()
        ],
        bump = submission.bump,
        constraint = !submission.reward_distributed @ RecruSearchError::RewardAlreadyClaimed
    )]
    pub submission: Account<'info, SubmissionAccount>,

    /// Reward token mint (e.g., USDC)
    #[account(mut)]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    /// Participant's token account (destination of reward payment)
    #[account(
        init_if_needed,
        payer = participant,
        associated_token::mint = reward_mint,
        associated_token::authority = participant
    )]
    pub participant_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Participant receiving the reward
    #[account(mut)]
    pub participant: UncheckedAccount<'info>,

    #[account(mut)]
    pub researcher: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

/// Create reward vault for a study
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct CreateRewardVault<'info> {
    #[account(
        seeds = [b"study", researcher.key().as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        init,
        payer = researcher,
        space = 8 + RewardVault::INIT_SPACE,
        seeds = [b"vault", study.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, RewardVault>,

    #[account(
        init,
        payer = researcher,
        token::mint = reward_token_mint,
        token::authority = reward_vault,
        token::token_program = token_program,
        seeds = [b"vault_token", reward_vault.key().as_ref()],
        bump
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub reward_token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = researcher,
        associated_token::mint = reward_token_mint,
        associated_token::authority = researcher,
        associated_token::token_program = token_program,
    )]
    pub researcher_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub researcher: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateRewardVault<'info> {
    pub fn create_reward_vault(
        &mut self,
        study_id: u64,
        initial_deposit: u64,
        bumps: &CreateRewardVaultBumps,
    ) -> Result<()> {
        let study = &self.study;
        let vault = &mut self.reward_vault;

        // Validate initial deposit
        let total_reward_needed = study.reward_amount_per_participant * study.max_participants as u64;
        require!(
            initial_deposit >= total_reward_needed,
            RecruSearchError::InsufficientFunds
        );

        // Check researcher has enough tokens
        require!(
            self.researcher_token_account.amount >= initial_deposit,
            RecruSearchError::InsufficientFunds
        );

        // Initialize vault
        vault.study = study.key();
        vault.reward_token_mint = self.reward_token_mint.key();
        vault.total_deposited = initial_deposit;
        vault.total_distributed = 0;
        vault.vault_authority = vault.key();
        vault.bump = bumps.reward_vault;

        // Transfer tokens from researcher to vault
        let cpi_accounts = TransferChecked {
            from: self.researcher_token_account.to_account_info(),
            mint: self.reward_token_mint.to_account_info(),
            to: self.vault_token_account.to_account_info(),
            authority: self.researcher.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(
            cpi_ctx,
            initial_deposit,
            self.reward_token_mint.decimals,
        )?;

        msg!("Reward vault created successfully");
        msg!("Study ID: {}", study_id);
        msg!("Initial deposit: {} tokens", initial_deposit);
        msg!("Vault: {}", vault.key());

        Ok(())
    }
}

// Helper function for vault PDA signer seeds
fn vault_signer_seeds(study_key: &Pubkey, vault_bump: u8) -> ([u8; 5], Vec<u8>, [u8; 1]) {
    (b"vault".clone(), study_key.to_bytes().to_vec(), [vault_bump])
}

impl<'info> DistributeReward<'info> {
    pub fn distribute_reward(&mut self, _bumps: &DistributeRewardBumps) -> Result<()> {
        let study = &self.study;
        let submission = &mut self.submission;
        let vault = &mut self.reward_vault;

        // Validate reward claim
        let clock = Clock::get()?;
        
        // Check if study allows reward claims
        require!(
            study.status == StudyStatus::Active || 
            study.status == StudyStatus::Closed,
            RecruSearchError::InvalidStudyState
        );

        // Check if sufficient time has passed since submission
        let min_time_before_claim = 24 * 60 * 60; // 24 hours in seconds
        require!(
            clock.unix_timestamp >= submission.submission_timestamp + min_time_before_claim,
            RecruSearchError::InvalidDataCollectionPeriod
        );

        // Check if vault has enough token balance
        let vault_token_balance = self.vault_token_account.amount;
        require!(
            vault_token_balance >= study.reward_amount_per_participant,
            RecruSearchError::InsufficientFunds
        );

        // Transfer reward tokens from vault to participant using PDA signing
        let reward_amount = study.reward_amount_per_participant;
        
        // Create signer seeds for vault PDA
        let (prefix, study_bytes, bump) = vault_signer_seeds(&study.key(), vault.bump);
        let signer_seeds: &[&[u8]] = &[&prefix, &study_bytes, &bump];
        let signer_seeds = &[signer_seeds];
        
        let cpi_accounts = TransferChecked {
            from: self.vault_token_account.to_account_info(),
            mint: self.reward_mint.to_account_info(),
            to: self.participant_token_account.to_account_info(),
            authority: vault.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(
            cpi_ctx,
            reward_amount,
            self.reward_mint.decimals,
        )?;

        // Update vault state
        vault.total_distributed = vault.total_distributed.saturating_add(reward_amount);

        // Mark reward as distributed
        submission.reward_distributed = true;

        msg!("Reward distributed successfully from vault");
        msg!("Amount: {} tokens", reward_amount);
        msg!("Participant: {}", self.participant.key());
        msg!("Study: {}", study.study_id);
        msg!("Vault total distributed: {}", vault.total_distributed);

        Ok(())
    }
}