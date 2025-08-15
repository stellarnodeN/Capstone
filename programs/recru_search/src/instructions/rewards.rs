use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
    token::{transfer_checked, TransferChecked},
};
use crate::state::*;

// transfers tokens to participants for study completion

#[derive(Accounts)]
pub struct DistributeReward<'info> {
    // Study account for reward validation
    #[account(
        mut,
        seeds = [b"study", study.researcher.as_ref(), study.study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    // Reward vault account - holds study rewards
    #[account(
        mut,
        seeds = [b"vault", study.key().as_ref()],
        bump = reward_vault.bump,
        constraint = reward_vault.study == study.key() @ RecruSearchError::InvalidParameterValue
    )]
    pub reward_vault: Account<'info, RewardVault>,

    // Vault token account - source of reward tokens
    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = reward_vault,
        token::token_program = token_program,
        seeds = [b"vault_token", reward_vault.key().as_ref()],
        bump
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    // Consent account - verifies participant enrollment
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

    // Submission account - verifies data submission and prevents double claims
    #[account(
        mut,
        seeds = [
            b"submission",
            study.key().as_ref(),
            participant.key().as_ref()
        ],
        bump = submission.bump,
        constraint = !submission.reward_distributed @ RecruSearchError::RewardAlreadyClaimed,
        constraint = submission.participant == participant.key() @ RecruSearchError::UnauthorizedParticipant
    )]
    pub submission: Account<'info, SubmissionAccount>,

    // Reward token mint
    #[account(mut)]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    // Participant token account - destination for rewards
    #[account(
        init_if_needed,
        payer = participant,
        associated_token::mint = reward_mint,
        associated_token::authority = participant
    )]
    pub participant_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: This is the participant account that will receive the reward
    #[account(mut)]
    pub participant: UncheckedAccount<'info>,

    // Researcher authorizing reward distribution
    #[account(mut)]
    pub researcher: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// Reward vault creation - sets up token vault for study rewards

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct CreateRewardVault<'info> {
    // Study account for vault association
    #[account(
        seeds = [b"study", researcher.key().as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump
    )]
    pub study: Account<'info, StudyAccount>,

    // Reward vault account - manages study rewards
    #[account(
        init,
        payer = researcher,
        space = 8 + RewardVault::INIT_SPACE,
        seeds = [b"vault", study.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, RewardVault>,

    // Vault token account - holds reward tokens
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

    // Reward token mint
    pub reward_token_mint: InterfaceAccount<'info, Mint>,

    // Researcher token account - source of initial deposit
    #[account(
        init_if_needed,
        payer = researcher,
        associated_token::mint = reward_token_mint,
        associated_token::authority = researcher,
        associated_token::token_program = token_program,
    )]
    pub researcher_token_account: InterfaceAccount<'info, TokenAccount>,

    // Researcher creating the vault
    #[account(mut)]
    pub researcher: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateRewardVault<'info> {
    // Creates reward vault and deposits initial tokens
    pub fn create_reward_vault(
        &mut self,
        study_id: u64,
        initial_deposit: u64,
        bumps: &CreateRewardVaultBumps,
    ) -> Result<()> {
        let study = &self.study;
        let vault = &mut self.reward_vault;

        // Validate sufficient initial deposit
        let total_reward_needed = study.reward_amount_per_participant * study.max_participants as u64;
        require!(
            initial_deposit >= total_reward_needed,
            RecruSearchError::InsufficientFunds
        );

        require!(
            self.researcher_token_account.amount >= initial_deposit,
            RecruSearchError::InsufficientFunds
        );

        // Initialize vault account
        vault.study = study.key();
        vault.reward_token_mint = self.reward_token_mint.key();
        vault.total_deposited = initial_deposit;
        vault.total_distributed = 0;
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

        // Log vault creation details
        msg!("Reward vault created successfully");
        msg!("Study ID: {}", study_id);
        msg!("Initial deposit: {} tokens", initial_deposit);
        msg!("Vault: {}", vault.key());

        // Emit reward vault created event
        emit!(RewardVaultCreated {
            study_id,
            researcher: self.researcher.key(),
            reward_mint: self.reward_token_mint.key(),
            initial_deposit,
        });

        Ok(())
    }
}

// Helper function for vault signer seeds
fn vault_signer_seeds(study_key: &Pubkey, vault_bump: u8) -> ([u8; 5], Vec<u8>, [u8; 1]) {
    (b"vault".clone(), study_key.to_bytes().to_vec(), [vault_bump])
}

impl<'info> DistributeReward<'info> {
    // Distributes reward tokens to participant after verification
    pub fn distribute_reward(&mut self, _bumps: &DistributeRewardBumps) -> Result<()> {
        let study = &self.study;
        let submission = &mut self.submission;
        let vault = &mut self.reward_vault;

        let clock = Clock::get()?;
        
        // Validate study is in active state
        require!(
            study.status == StudyStatus::Active,
            RecruSearchError::InvalidStudyState
        );

        // Enforce minimum time before claiming (24 hours)
        let min_time_before_claim = 24 * 60 * 60; // 24 hours
        require!(
            clock.unix_timestamp >= submission.submission_timestamp + min_time_before_claim,
            RecruSearchError::InvalidDataCollectionPeriod
        );

        // Validate sufficient vault balance
        let vault_token_balance = self.vault_token_account.amount;
        require!(
            vault_token_balance >= study.reward_amount_per_participant,
            RecruSearchError::InsufficientFunds
        );

        let reward_amount = study.reward_amount_per_participant;
        
        let (prefix, study_bytes, bump) = vault_signer_seeds(&study.key(), vault.bump);
        let signer_seeds: &[&[u8]] = &[&prefix, &study_bytes, &bump];
        let signer_seeds = &[signer_seeds];
        
        // Transfer tokens from vault to participant
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

        vault.total_distributed = vault.total_distributed.saturating_add(reward_amount);
        submission.reward_distributed = true;

        let study = &mut self.study;
        study.total_rewards_distributed = study.total_rewards_distributed.saturating_add(reward_amount);

        msg!("Reward distributed successfully from vault");
        msg!("Amount: {} tokens", reward_amount);
        msg!("Participant: {}", self.participant.key());
        msg!("Study: {}", study.study_id);
        msg!("Vault total distributed: {}", vault.total_distributed);
        msg!("Study total rewards distributed: {}", study.total_rewards_distributed);

        // Emit reward distributed event
        emit!(RewardDistributed {
            study_id: study.study_id,
            participant: self.participant.key(),
            amount: reward_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}