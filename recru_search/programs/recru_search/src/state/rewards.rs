use anchor_lang::prelude::*;

/// Individual reward account tracking a participant's reward for completing a study
#[account]
#[derive(InitSpace)]
pub struct RewardAccount {
    pub study: Pubkey,                // Associated study
    pub participant: Pubkey,          // Participant to reward
    pub submission: Pubkey,           // Associated submission
    pub amount: u64,                  // Reward amount
    pub token_mint: Pubkey,           // SPL token mint
    pub vault: Pubkey,                // Vault holding rewards
    pub is_claimed: bool,             // Whether reward is claimed
    pub completion_nft_mint: Option<Pubkey>, // Optional completion NFT
    pub bump: u8,
}

/// Vault account that manages the token pool for storing participant rewards
/// This holds the SPL tokens that will be distributed to participants when they complete studies
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
