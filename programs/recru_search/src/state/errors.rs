use anchor_lang::prelude::*;

#[error_code]
pub enum RecruSearchError {
    // Study validation errors - validate study parameters and constraints
    #[msg("Study title exceeds maximum length of 100 characters")]
    TitleTooLong = 6000,
    #[msg("Study description exceeds maximum length of 500 characters")]
    DescriptionTooLong = 6001,
    #[msg("Enrollment start time must be in the future")]
    InvalidEnrollmentStart = 6002,
    #[msg("Enrollment end time must be after enrollment start time")]
    InvalidEnrollmentEnd = 6003,
    #[msg("Enrollment period must be at least 1 hour")]
    InvalidEnrollmentPeriod = 6004,
    #[msg("Data collection end time must be after enrollment end time")]
    InvalidDataCollectionEnd = 6005,
    #[msg("Data collection period must be between 1 day and 1 year")]
    InvalidDataCollectionPeriod = 6006,
    #[msg("Maximum participants must be between 1 and 10,000")]
    InvalidMaxParticipants = 6007,
    #[msg("Invalid parameter value provided")]
    InvalidParameterValue = 6008,

    // Access control errors - validate user permissions and authorization
    #[msg("Only the study researcher can perform this action")]
    UnauthorizedResearcher = 6100,
    #[msg("Only the enrolled participant can perform this action")]
    UnauthorizedParticipant = 6101,
    #[msg("Insufficient permissions to perform this action")]
    UnauthorizedAccess = 6102,

    // State transition errors - validate study and consent state changes
    #[msg("Study is not in the required state for this operation")]
    InvalidStudyState = 6200,
    #[msg("Invalid state transition for the study")]
    InvalidStatusTransition = 6201,
    #[msg("Study must be published before participants can enroll")]
    StudyNotPublished = 6202,
    #[msg("Study has reached maximum participant capacity")]
    StudyFull = 6203,
    #[msg("Study has already been closed and cannot be modified")]
    StudyAlreadyClosed = 6204,
    #[msg("Consent is not active or has been revoked")]
    ConsentNotActive = 6205,

    // Data validation errors - validate input data and IPFS content
    #[msg("Data format is invalid or corrupted")]
    InvalidDataFormat = 6300,
    #[msg("IPFS CID format is invalid or unsupported")]
    InvalidIPFSCID = 6301,
    #[msg("Eligibility proof format is invalid or missing required data")]
    InvalidEligibilityProof = 6302,
    #[msg("Participant does not meet the study's eligibility criteria")]
    ParticipantNotEligible = 6303,
    #[msg("Study has no eligibility criteria set for verification")]
    NoEligibilityCriteria = 6304,

    // Participant action errors - validate participant enrollment and submission
    #[msg("Consent has been revoked and cannot be used")]
    ConsentRevoked = 6400,
    #[msg("Data has already been submitted for this study")]
    AlreadySubmitted = 6401,

    // Token and reward errors - validate token transfers and reward distribution
    #[msg("Insufficient token balance for this operation")]
    InsufficientFunds = 6500,
    #[msg("Reward has already been claimed by this participant")]
    RewardAlreadyClaimed = 6501,
    #[msg("Reward has already been distributed for this submission")]
    RewardAlreadyDistributed = 6502,
    #[msg("Reward has not been distributed yet")]
    RewardNotDistributed = 6503,
    #[msg("Protocol fee exceeds maximum allowed rate of 10%")]
    ExcessiveProtocolFee = 6504,

    // Processing errors - validate data processing and arithmetic operations
    #[msg("Data anonymization process failed")]
    AnonymizationFailed = 6600,
    #[msg("Arithmetic overflow or underflow occurred")]
    ArithmeticError = 6601,
}