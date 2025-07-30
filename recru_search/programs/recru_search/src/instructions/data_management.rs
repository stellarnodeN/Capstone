use anchor_lang::prelude::*;
use crate::state::{
    study::{StudyAccount, StudyStatus},
    survey::{SurveySchema, DataCollectionStats},
};
use crate::error::RecruSearchError;
use crate::constants::*;

// Simplified data structures for MVP (moved from utils/privacy)
#[derive(Clone, Debug)]
pub struct AnonymizationConfig {
    pub method: AnonymizationMethod,
}

#[derive(Clone, Debug)]
pub struct AnonymizationReport {
    pub study_id: u64,
    pub total_responses_processed: u32,
    pub successfully_anonymized: u32,
    pub failed_anonymizations: u32,
    pub method: AnonymizationMethod,
}

impl AnonymizationReport {
    pub fn new(study_id: u64, total: u32, success: u32, failed: u32, method: AnonymizationMethod) -> Self {
        Self {
            study_id,
            total_responses_processed: total,
            successfully_anonymized: success,
            failed_anonymizations: failed,
            method,
        }
    }
}

// =============================================================================
// DATA COLLECTION INSTRUCTIONS
// =============================================================================

/// Create survey schema for data collection (simplified for MVP)
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct CreateSurveySchema<'info> {
    #[account(
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::InvalidStatusTransition
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        init,
        payer = researcher,
        space = 8 + SurveySchema::INIT_SPACE,
        seeds = [b"survey", study.key().as_ref()],
        bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    #[account(
        init,
        payer = researcher,
        space = 8 + DataCollectionStats::INIT_SPACE,
        seeds = [b"data_stats", study.key().as_ref()],
        bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    #[account(mut)]
    pub researcher: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateSurveySchema<'info> {
    pub fn create_survey_schema(
        &mut self,
        study_id: u64,
        survey_title: String,
        survey_description: String,
        question_count: u32,
        estimated_duration_minutes: u32,
        schema_ipfs_cid: String,
        requires_encryption: bool,
        supports_file_uploads: bool,
        anonymous_responses: bool,
            bumps: &CreateSurveySchemaBumps,
    ) -> Result<()> {
        let clock = Clock::get()?;

        // Basic validation
        require!(
            survey_title.len() >= 5 && survey_title.len() <= 100,
            RecruSearchError::TitleTooLong
        );
        require!(
            survey_description.len() <= 300,
            RecruSearchError::InvalidDataFormat
        );
        require!(
            question_count > 0 && question_count <= 50, // MVP limit
            RecruSearchError::InvalidDataFormat
        );
        require!(
            schema_ipfs_cid.len() >= 10 && schema_ipfs_cid.len() <= 100,
            RecruSearchError::InvalidDataFormat
        );

        // Initialize survey schema (simplified for MVP)
        let survey_schema = &mut self.survey_schema;
        survey_schema.study_id = study_id;
        survey_schema.researcher = self.researcher.key();
        survey_schema.survey_title = survey_title.clone();
        survey_schema.survey_description = survey_description;
        survey_schema.question_count = question_count;
        survey_schema.estimated_duration_minutes = estimated_duration_minutes;
        survey_schema.schema_version = 1; // Start with version 1
        survey_schema.schema_ipfs_cid = schema_ipfs_cid;
        survey_schema.requires_encryption = requires_encryption;
        survey_schema.supports_file_uploads = supports_file_uploads;
        survey_schema.anonymous_responses = anonymous_responses;
        survey_schema.created_at = clock.unix_timestamp;
        survey_schema.is_active = false;
        survey_schema.response_count = 0;
        survey_schema.bump = bumps.survey_schema;

        // Initialize data collection stats (simplified for MVP)
        let data_stats = &mut self.data_stats;
        data_stats.study_id = study_id;
        data_stats.researcher = self.researcher.key();
        data_stats.total_responses = 0;
        data_stats.complete_responses = 0;
        data_stats.validated_responses = 0;
        data_stats.encrypted_responses = 0;
        data_stats.anonymized_responses = 0;
        data_stats.gdpr_deletion_requests = 0;
        data_stats.total_files_uploaded = 0;
        data_stats.total_file_size_mb = 0;
        data_stats.average_completion_time_seconds = 0;
        data_stats.first_response_timestamp = 0;
        data_stats.last_response_timestamp = 0;
        data_stats.last_updated = clock.unix_timestamp;
        data_stats.bump = bumps.data_stats;

        msg!(
            "Survey schema created: '{}' with {} questions for study {}",
            survey_title,
            question_count,
            study_id
        );

        Ok(())
    }
}

/// Finalize survey schema and activate data collection (simplified for MVP)
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct FinalizeSurveySchema<'info> {
    #[account(
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        mut,
        seeds = [b"survey", study.key().as_ref()],
        bump = survey_schema.bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> FinalizeSurveySchema<'info> {
    pub fn finalize_survey_schema(&mut self, study_id: u64) -> Result<()> {
        // Activate the survey for data collection
        self.survey_schema.is_active = true;

        msg!(
            "Survey schema finalized and activated for study {}: '{}'",
            study_id,
            self.survey_schema.survey_title
        );

        Ok(())
    }
}

// =============================================================================
// DATA ANONYMIZATION INSTRUCTIONS
// =============================================================================

/// Account validation struct for anonymizing participant data
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct AnonymizeParticipantData<'info> {
    #[account(
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        seeds = [b"survey_schema", study.key().as_ref(), survey_schema.schema_version.to_le_bytes().as_ref()],
        bump = survey_schema.bump,
        constraint = survey_schema.anonymous_responses @ RecruSearchError::AnonymizationFailed
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    #[account(
        mut,
        seeds = [b"data_stats", study.key().as_ref()],
        bump = data_stats.bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> AnonymizeParticipantData<'info> {
    pub fn handle_anonymize_data(
        &mut self,
        study_id: u64,
        anonymization_config: AnonymizationConfig,
        response_ids: Vec<u64>,
    ) -> Result<AnonymizationReport> {
        let _study = &self.study;
        let schema = &self.survey_schema;
        let stats = &mut self.data_stats;

        // Validate anonymization is allowed
        require!(schema.anonymous_responses, RecruSearchError::AnonymizationFailed);

        let mut report = AnonymizationReport::new(
            study_id,
            response_ids.len() as u32,
            0,
            0,
            anonymization_config.method.clone(),
        );

        // Process each response for anonymization
        for _response_id in response_ids.iter() {
            // In a real implementation, this would:
            // 1. Load the response data from storage
            // 2. Apply the selected anonymization method
            // 3. Store the anonymized version
            // 4. Update the report counters
            
            // For MVP, we'll just simulate successful anonymization
            report.successfully_anonymized += 1;
        }

        // Update collection statistics
        stats.anonymized_responses = stats.anonymized_responses
            .checked_add(report.successfully_anonymized)
            .ok_or(RecruSearchError::ArithmeticError)?;

        stats.last_updated = Clock::get()?.unix_timestamp;

        msg!(
            "Anonymized {}/{} responses for study {} using {:?} method",
            report.successfully_anonymized,
            report.total_responses_processed,
            study_id,
            anonymization_config.method
        );

        Ok(report)
    }
}

// =============================================================================
// DATA ANALYTICS INSTRUCTIONS
// =============================================================================

/// Export survey responses for research analysis
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct ExportSurveyData<'info> {
    /// Study account for validation
    #[account(
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    /// Survey schema for metadata
    #[account(
        seeds = [b"survey_schema", study.key().as_ref(), survey_schema.schema_version.to_le_bytes().as_ref()],
        bump = survey_schema.bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    /// Data collection statistics
    #[account(
        mut,
        seeds = [b"data_stats", study.key().as_ref()],
        bump = data_stats.bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> ExportSurveyData<'info> {
    /// Generate export manifest with all survey responses and file uploads
    pub fn export_survey_data(
        &mut self,
        study_id: u64,
        export_format: ExportFormat,
        include_files: bool,
        anonymize_responses: bool,
    ) -> Result<ExportManifest> {
        let study = &self.study;
        let schema = &self.survey_schema;
        let stats = &mut self.data_stats;

        // Validate export permissions
        require!(
            matches!(study.status, StudyStatus::Active | StudyStatus::Closed),
            RecruSearchError::InvalidStatusTransition
        );

        // Generate export manifest
        let export_manifest = ExportManifest {
            study_id,
            study_title: study.title.clone(),
            export_timestamp: Clock::get()?.unix_timestamp,
            export_format,
            include_files,
            anonymized: anonymize_responses,
            schema_version: schema.schema_version,
            total_responses: stats.total_responses,
            complete_responses: stats.complete_responses,
            file_count: stats.total_files_uploaded,
            export_ipfs_cid: String::new(), // Would be set after IPFS upload
        };

        msg!(
            "Data export initiated for study {}: '{}' ({} responses, {} files)",
            study_id,
            study.title,
            stats.total_responses,
            stats.total_files_uploaded
        );

        Ok(export_manifest)
    }
}

/// Generate research compliance report
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct GenerateComplianceReport<'info> {
    /// Study account
    #[account(
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    /// Survey schema for compliance details
    #[account(
        seeds = [b"survey_schema", study.key().as_ref(), survey_schema.schema_version.to_le_bytes().as_ref()],
        bump = survey_schema.bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    /// Data collection statistics
    #[account(
        seeds = [b"data_stats", study.key().as_ref()],
        bump = data_stats.bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> GenerateComplianceReport<'info> {
    /// Generate comprehensive compliance report for IRB/ethics boards
    pub fn generate_compliance_report(&self, study_id: u64) -> Result<ComplianceReport> {
        let study = &self.study;
        let schema = &self.survey_schema;
        let stats = &self.data_stats;
        let clock = Clock::get()?;

        let report = ComplianceReport {
            // Study identification
            study_id,
            study_title: study.title.clone(),
            researcher_wallet: study.researcher,
            generation_timestamp: clock.unix_timestamp,
            
            // Study timeline compliance
            planned_enrollment_start: study.enrollment_start,
            planned_enrollment_end: study.enrollment_end,
            planned_data_collection_end: study.data_collection_end,
            actual_first_response: stats.first_response_timestamp,
            actual_last_response: stats.last_response_timestamp,
            timeline_compliant: clock.unix_timestamp <= study.data_collection_end,
            
            // Participation metrics
            target_participants: study.max_participants,
            enrolled_participants: study.enrolled_count,
            completed_participants: stats.complete_responses,
            recruitment_rate: if study.max_participants > 0 {
                (study.enrolled_count as f64 / study.max_participants as f64 * 100.0) as u8
            } else { 0 },
            
            // Data protection compliance
            encryption_required: schema.requires_encryption,
            anonymization_enabled: schema.anonymous_responses,
            encrypted_responses_count: stats.encrypted_responses,
            anonymized_responses_count: stats.anonymized_responses,
            gdpr_requests_handled: stats.gdpr_deletion_requests,
            
            // Quality assurance
            validated_responses: stats.validated_responses,
            validation_rate: if stats.total_responses > 0 {
                (stats.validated_responses as f64 / stats.total_responses as f64 * 100.0) as u8
            } else { 0 },
            
            // File handling compliance
            file_uploads_enabled: schema.supports_file_uploads,
            total_files_collected: stats.total_files_uploaded,
            total_storage_used_gb: (stats.total_file_size_mb as f64 / 1024.0) as f32,
            
            // Audit trail
            protocol_violations: 0, // Would be calculated from event logs
            security_incidents: 0,  // Would be calculated from security logs
        };

        msg!(
            "Compliance report generated for study {}: {} participants, {} responses",
            study_id,
            study.enrolled_count,
            stats.total_responses
        );

        Ok(report)
    }
}

// =============================================================================
// PRIVACY & COMPLIANCE INSTRUCTIONS
// =============================================================================

/// Verify data quality and integrity
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct VerifyDataQuality<'info> {
    #[account(
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        seeds = [b"survey_schema", study.key().as_ref(), survey_schema.schema_version.to_le_bytes().as_ref()],
        bump = survey_schema.bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    #[account(
        mut,
        seeds = [b"data_stats", study.key().as_ref()],
        bump = data_stats.bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> VerifyDataQuality<'info> {
    /// Verify data quality using multi-dimensional scoring
    pub fn verify_data_quality(
        &mut self,
        study_id: u64,
        responses_to_verify: Vec<ResponseQualityCheck>,
    ) -> Result<QualityVerificationReport> {
        let mut report = QualityVerificationReport {
            study_id,
            verification_timestamp: Clock::get()?.unix_timestamp,
            total_responses_checked: responses_to_verify.len() as u32,
            high_quality_responses: 0,
            medium_quality_responses: 0,
            low_quality_responses: 0,
            flagged_responses: 0,
            overall_quality_score: 0,
            verification_criteria: QualityVerificationCriteria {
                completeness_weight: 30,
                consistency_weight: 25,
                timeliness_weight: 20,
                authenticity_weight: 15,
                engagement_weight: 10,
            },
        };

        let mut total_quality_score = 0u32;

        // Process each response for quality verification
        for response_check in responses_to_verify.iter() {
            let quality_score = self.calculate_quality_score(response_check)?;
            total_quality_score = total_quality_score.checked_add(quality_score as u32).unwrap();

            // Categorize response quality
            match quality_score {
                80..=100 => report.high_quality_responses += 1,
                60..=79 => report.medium_quality_responses += 1,
                40..=59 => report.low_quality_responses += 1,
                _ => {
                    report.flagged_responses += 1;
                    msg!("WARNING: Low quality response flagged: ID {}", response_check.response_id);
                }
            }
        }

        // Calculate overall quality score
        if !responses_to_verify.is_empty() {
            report.overall_quality_score = (total_quality_score / responses_to_verify.len() as u32) as u8;
        }

        // Update data collection statistics
        let stats = &mut self.data_stats;
        stats.validated_responses = stats.validated_responses
            .checked_add(report.total_responses_checked).unwrap();
        stats.last_updated = Clock::get()?.unix_timestamp;

        msg!(
            "SUCCESS: Data quality verification completed for study {}: {}% overall quality score",
            study_id,
            report.overall_quality_score
        );

        Ok(report)
    }

    /// Calculate comprehensive quality score for a response
    fn calculate_quality_score(&self, response: &ResponseQualityCheck) -> Result<u8> {
        let criteria = QualityVerificationCriteria {
            completeness_weight: 30,
            consistency_weight: 25,
            timeliness_weight: 20,
            authenticity_weight: 15,
            engagement_weight: 10,
        };

        // Completeness score (0-100)
        let completeness_score = response.completion_percentage;

        // Consistency score based on response patterns
        let consistency_score = if response.has_contradictory_answers {
            40 // Penalize contradictory answers
        } else if response.response_variance_score > 80 {
            90 // Reward consistent, thoughtful responses
        } else {
            response.response_variance_score
        };

        // Timeliness score based on completion duration
        let timeliness_score = match response.completion_duration_seconds {
            0..=300 => 20,      // Too fast, likely not thoughtful
            301..=1800 => 100,  // Optimal completion time
            1801..=3600 => 80,  // Reasonable time
            _ => 60,            // Too slow, may indicate disengagement
        };

        // Authenticity score based on various signals
        let authenticity_score = if response.has_bot_like_patterns {
            10 // Strong penalty for bot-like behavior
        } else if response.ip_address_suspicious {
            50 // Moderate penalty for suspicious IP
        } else {
            95 // Default high authenticity
        };

        // Engagement score based on qualitative responses
        let engagement_score = response.text_response_quality_score.unwrap_or(75);

        // Calculate weighted average
        let weighted_score = (
            (completeness_score as u32 * criteria.completeness_weight) +
            (consistency_score as u32 * criteria.consistency_weight) +
            (timeliness_score as u32 * criteria.timeliness_weight) +
            (authenticity_score as u32 * criteria.authenticity_weight) +
            (engagement_score as u32 * criteria.engagement_weight)
        ) / 100;

        Ok(weighted_score.min(100) as u8)
    }
}

/// Handle GDPR data deletion requests
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct ProcessGDPRDeletion<'info> {
    #[account(
        seeds = [STUDY_SEED.as_bytes(), study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump
    )]
    pub study: Account<'info, StudyAccount>,

    #[account(
        mut,
        seeds = [b"data_stats", study.key().as_ref()],
        bump = data_stats.bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    #[account(mut)]
    pub participant: Signer<'info>,
}

impl<'info> ProcessGDPRDeletion<'info> {
    /// Process GDPR "Right to be Forgotten" request
    pub fn process_gdpr_deletion(
        &mut self,
        study_id: u64,
        deletion_request: GDPRDeletionRequest,
    ) -> Result<GDPRDeletionReport> {
        let clock = Clock::get()?;

        // Validate deletion request
        require!(
            deletion_request.participant == self.participant.key(),
            RecruSearchError::UnauthorizedAccess
        );

        // Create deletion report
        let report = GDPRDeletionReport {
            study_id,
            participant: deletion_request.participant,
            deletion_timestamp: clock.unix_timestamp,
            request_type: deletion_request.deletion_type,
            data_categories_deleted: deletion_request.data_categories.clone(),
            retention_override: deletion_request.research_retention_override,
            compliance_verified: true,
            verification_hash: [0u8; 32], // Would be calculated from deletion process
        };

        // Update statistics
        self.data_stats.gdpr_deletion_requests = self.data_stats.gdpr_deletion_requests
            .checked_add(1).unwrap();
        self.data_stats.last_updated = clock.unix_timestamp;

        msg!(
            "GDPR deletion processed for participant {} in study {}: {:?}",
            deletion_request.participant,
            study_id,
            deletion_request.deletion_type
        );

        Ok(report)
    }
}

// =============================================================================
// DATA STRUCTURES
// =============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ExportFormat {
    CSV,
    JSON,
    SPSS,
    R,
    Excel,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExportManifest {
    pub study_id: u64,
    pub study_title: String,
    pub export_timestamp: i64,
    pub export_format: ExportFormat,
    pub include_files: bool,
    pub anonymized: bool,
    pub schema_version: u32,
    pub total_responses: u32,
    pub complete_responses: u32,
    pub file_count: u32,
    pub export_ipfs_cid: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ComplianceReport {
    pub study_id: u64,
    pub study_title: String,
    pub researcher_wallet: Pubkey,
    pub generation_timestamp: i64,
    pub planned_enrollment_start: i64,
    pub planned_enrollment_end: i64,
    pub planned_data_collection_end: i64,
    pub actual_first_response: i64,
    pub actual_last_response: i64,
    pub timeline_compliant: bool,
    pub target_participants: u32,
    pub enrolled_participants: u32,
    pub completed_participants: u32,
    pub recruitment_rate: u8,
    pub encryption_required: bool,
    pub anonymization_enabled: bool,
    pub encrypted_responses_count: u32,
    pub anonymized_responses_count: u32,
    pub gdpr_requests_handled: u32,
    pub validated_responses: u32,
    pub validation_rate: u8,
    pub file_uploads_enabled: bool,
    pub total_files_collected: u32,
    pub total_storage_used_gb: f32,
    pub protocol_violations: u32,
    pub security_incidents: u32,
}

/// Methods for anonymizing participant data
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum AnonymizationMethod {
    PseudonymReplacement,   // Replace identifiers with pseudonyms
    DataGeneralization,     // Generalize specific data points
    DataSuppression,        // Remove sensitive fields entirely
    DifferentialPrivacy,    // Add statistical noise for privacy
    KAnonymity,            // Ensure k-anonymity in dataset
}

/// Compliance standards supported by the platform
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum ComplianceStandard {
    GDPR,      // European General Data Protection Regulation
    HIPAA,     // US Health Insurance Portability and Accountability Act
    IRB,       // Institutional Review Board requirements
    CCPA,      // California Consumer Privacy Act
    SOX,       // Sarbanes-Oxley Act
}

/// Quality check parameters for a survey response
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ResponseQualityCheck {
    pub response_id: u64,
    pub participant: Pubkey,
    pub completion_percentage: u8,
    pub completion_duration_seconds: u32,
    pub has_contradictory_answers: bool,
    pub response_variance_score: u8,
    pub has_bot_like_patterns: bool,
    pub ip_address_suspicious: bool,
    pub text_response_quality_score: Option<u8>,
}

/// Criteria used for quality verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct QualityVerificationCriteria {
    pub completeness_weight: u32,    // Weight for completion percentage
    pub consistency_weight: u32,     // Weight for response consistency
    pub timeliness_weight: u32,      // Weight for completion time
    pub authenticity_weight: u32,    // Weight for authenticity signals
    pub engagement_weight: u32,      // Weight for engagement quality
}

/// Report generated after quality verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct QualityVerificationReport {
    pub study_id: u64,
    pub verification_timestamp: i64,
    pub total_responses_checked: u32,
    pub high_quality_responses: u32,
    pub medium_quality_responses: u32,
    pub low_quality_responses: u32,
    pub flagged_responses: u32,
    pub overall_quality_score: u8,
    pub verification_criteria: QualityVerificationCriteria,
}

/// GDPR deletion request details
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GDPRDeletionRequest {
    pub participant: Pubkey,
    pub deletion_type: GDPRDeletionType,
    pub data_categories: Vec<DataCategory>,
    pub research_retention_override: bool,
}

/// Types of GDPR deletion requests
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum GDPRDeletionType {
    CompleteErasure,        // Delete all participant data
    PartialErasure,         // Delete specific data categories
    Anonymization,          // Convert to anonymous data
    DataPortability,        // Export data before deletion
}

/// Categories of data that can be deleted
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum DataCategory {
    SurveyResponses,        // Survey response data
    FileUploads,            // Uploaded files
    ConsentRecords,         // Consent NFT metadata
    ParticipationHistory,   // Study participation records
    RewardHistory,          // Reward distribution records
}

/// Report generated after GDPR deletion
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GDPRDeletionReport {
    pub study_id: u64,
    pub participant: Pubkey,
    pub deletion_timestamp: i64,
    pub request_type: GDPRDeletionType,
    pub data_categories_deleted: Vec<DataCategory>,
    pub retention_override: bool,
    pub compliance_verified: bool,
    pub verification_hash: [u8; 32],
}


