use anchor_lang::prelude::*;

#[error_code]
pub enum RecruSearchError {
    #[msg("Study title is too long (max 100 characters)")]
    TitleTooLong,
    #[msg("Study description is too long (max 500 characters)")]
    DescriptionTooLong,
    #[msg("Enrollment start time must be in the future")]
    InvalidEnrollmentStart,
    #[msg("Enrollment end time must be after start time")]
    InvalidEnrollmentEnd,
    #[msg("Invalid enrollment period configuration")]
    InvalidEnrollmentPeriod,
    #[msg("Data collection end time must be after enrollment end time")]
    InvalidDataCollectionEnd,
    #[msg("Invalid data collection period configuration")]
    InvalidDataCollectionPeriod,
    #[msg("Max participants must be greater than 0")]
    InvalidMaxParticipants,
    #[msg("Insufficient vault balance for reward distribution")]
    InsufficientVaultBalance,
    #[msg("Insufficient deposit to cover expected rewards")]
    InsufficientRewardDeposit,
    #[msg("Math operation resulted in overflow")]
    MathOverflow,
    
    // Publish study errors
    #[msg("Invalid status transition. Study must be in Draft status to publish")]
    InvalidStatusTransition,
    
    #[msg("Unauthorized researcher. Only the study creator can perform this action")]
    UnauthorizedResearcher,
    
    #[msg("Unauthorized access. Only authorized users can perform this action")]
    UnauthorizedAccess,
    
    #[msg("Invalid reward amount. Must be greater than 0")]
    InvalidRewardAmount,
    
    // Token and vault errors
    #[msg("Invalid token mint for this vault")]
    InvalidTokenMint,
    
    #[msg("Invalid token account configuration")]
    InvalidTokenAccount,
    
    // Consent NFT errors
    #[msg("Enrollment is closed for this study")]
    EnrollmentClosed,
    
    #[msg("Maximum number of participants reached")]
    MaxParticipantsReached,
    
    #[msg("Missing eligibility proof required for this study")]
    MissingEligibilityProof,
    
    // Data submission errors
    #[msg("Data collection is closed for this study")]
    DataCollectionClosed,
    
    #[msg("Data collection is still active - cannot close study yet")]
    DataCollectionStillActive,
    
    #[msg("Invalid consent NFT or consent has been revoked")]
    InvalidConsentNFT,
    
    #[msg("Invalid or revoked consent")]
    InvalidOrRevokedConsent,
    
    #[msg("Invalid data format or IPFS CID")]
    InvalidDataFormat,
    
    #[msg("Invalid data hash - cannot be empty")]
    InvalidDataHash,
    
    // Reward distribution errors
    #[msg("Reward has already been claimed for this submission")]
    RewardAlreadyClaimed,
    
    // Study closure errors
    #[msg("Study is already closed or archived")]
    StudyAlreadyClosed,
}
