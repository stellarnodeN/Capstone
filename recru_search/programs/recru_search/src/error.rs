use anchor_lang::prelude::*;

#[error_code]
pub enum RecruSearchError {
    // Study Validation (6000-6099)
    #[msg("Study title is too long")]
    TitleTooLong = 6000,
    #[msg("Study description is too long")]
    DescriptionTooLong = 6001,
    #[msg("Invalid enrollment start time")]
    InvalidEnrollmentStart = 6002,
    #[msg("Invalid enrollment end time")]
    InvalidEnrollmentEnd = 6003,
    #[msg("Invalid enrollment period")]
    InvalidEnrollmentPeriod = 6004,
    #[msg("Invalid data collection end time")]
    InvalidDataCollectionEnd = 6005,
    #[msg("Invalid data collection period")]
    InvalidDataCollectionPeriod = 6006,
    #[msg("Invalid maximum participants")]
    InvalidMaxParticipants = 6007,
    #[msg("Invalid parameter value")]
    InvalidParameterValue = 6008,

    // Access Control (6100-6199)
    #[msg("Unauthorized researcher")]
    UnauthorizedResearcher = 6100,
    #[msg("Unauthorized participant")]
    UnauthorizedParticipant = 6101,
    #[msg("Unauthorized access")]
    UnauthorizedAccess = 6102,

    // State/Status Issues (6200-6299)
    #[msg("Invalid study state")]
    InvalidStudyState = 6200,
    #[msg("Invalid status transition")]
    InvalidStatusTransition = 6201,
    #[msg("Study not published")]
    StudyNotPublished = 6202,
    #[msg("Study is full")]
    StudyFull = 6203,

    // Data Validation (6300-6399)
    #[msg("Invalid data format")]
    InvalidDataFormat = 6300,
    #[msg("Invalid IPFS CID")]
    InvalidIPFSCID = 6301,
    #[msg("Invalid eligibility proof")]
    InvalidEligibilityProof = 6302,
    #[msg("ZK proof validation failed")]
    ZKProofValidationFailed = 6303,

    // Participant Issues (6400-6499)
    #[msg("Consent revoked")]
    ConsentRevoked = 6400,
    #[msg("Already submitted")]
    AlreadySubmitted = 6401,

    // Financial/Token Issues (6500-6599)
    #[msg("Insufficient funds")]
    InsufficientFunds = 6500,
    #[msg("Reward already claimed")]
    RewardAlreadyClaimed = 6501,
    #[msg("Excessive protocol fee")]
    ExcessiveProtocolFee = 6502,

    // Processing Issues (6600-6699)
    #[msg("Anonymization failed")]
    AnonymizationFailed = 6600,
    #[msg("Arithmetic error")]
    ArithmeticError = 6601,
}