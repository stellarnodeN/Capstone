use anchor_lang::prelude::*;

// Global configuration account for the entire RecruSearch protocol
// This stores protocol-wide settings and is owned by the protocol administrator
#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub protocol_admin: Pubkey,           // Wallet address of the protocol administrator
    pub protocol_fee_basis_points: u16,   // Protocol fee (in basis points, e.g., 100 = 1%)
    pub min_study_duration: u32,          // Minimum required study duration in seconds
    pub max_study_duration: u32,          // Maximum allowed study duration in seconds
    pub paused: bool,                     // Emergency pause switch for the entire protocol
    pub bump: u8,                         // PDA bump seed for account creation
} 