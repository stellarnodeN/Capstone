use anchor_lang::prelude::*;

// Account that manages the token vault for storing participant rewards
// This holds the SPL tokens that will be distributed to participants when they complete studies
#[account]
#[derive(InitSpace)]
pub struct RewardVault {
    pub study_account: Pubkey,            // Which study this vault belongs to
    pub vault_token_account: Pubkey,      // Token account that holds the actual tokens
    pub reward_mint: Pubkey,              // The type of token being distributed (e.g., USDC, custom token)
    pub total_deposited: u64,             // Total amount of tokens deposited by the researcher
    pub participants_rewarded: u32,       // Number of participants who have received rewards
    pub created_at: i64,                  // When this vault was created (Unix timestamp)
    pub bump: u8,                         // PDA bump seed for account creation
} 