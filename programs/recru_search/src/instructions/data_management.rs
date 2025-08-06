use anchor_lang::prelude::*;
use crate::state::{StudyAccount, StudyStatus, SurveySchema, DataCollectionStats, RecruSearchError};
use crate::state::constants::{MIN_IPFS_CID_LENGTH, MAX_IPFS_CID_LENGTH, IPFS_CID_V0_PREFIX, IPFS_CID_V1_PREFIX};
use crate::state::events::SurveySchemaCreated;

// Survey schema creation - defines data collection structure for studies

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct CreateSurveySchema<'info> {
    // Study account for schema association
    #[account(
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher,
        constraint = study.status == StudyStatus::Published @ RecruSearchError::InvalidStatusTransition
    )]
    pub study: Account<'info, StudyAccount>,

    // Survey schema account - stores survey definition
    #[account(
        init,
        payer = researcher,
        space = 8 + SurveySchema::INIT_SPACE,
        seeds = [b"survey", study.key().as_ref()],
        bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    // Data collection stats account - tracks response metrics
    #[account(
        init,
        payer = researcher,
        space = 8 + DataCollectionStats::INIT_SPACE,
        seeds = [b"data_stats", study.key().as_ref()],
        bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    // Researcher creating the schema
    #[account(mut)]
    pub researcher: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateSurveySchema<'info> {
    // Creates survey schema and initializes data collection tracking
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
        bumps: &CreateSurveySchemaBumps,
    ) -> Result<()> {
        let clock = Clock::get()?;

        // Validate survey parameters
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
        // Validate IPFS CID format
        require!(
            schema_ipfs_cid.len() >= MIN_IPFS_CID_LENGTH && schema_ipfs_cid.len() <= MAX_IPFS_CID_LENGTH,
            RecruSearchError::InvalidIPFSCID
        );
        require!(
            schema_ipfs_cid.starts_with(IPFS_CID_V0_PREFIX) || schema_ipfs_cid.starts_with(IPFS_CID_V1_PREFIX),
            RecruSearchError::InvalidIPFSCID
        );

        // Initialize survey schema account
        let survey_schema = &mut self.survey_schema;
        survey_schema.study = self.study.key();
        survey_schema.title = survey_title.clone();
        survey_schema.schema_version = 1;
        survey_schema.question_count = question_count;
        survey_schema.estimated_duration_minutes = estimated_duration_minutes;
        survey_schema.schema_ipfs_cid = schema_ipfs_cid;
        survey_schema.requires_encryption = requires_encryption;
        survey_schema.supports_file_uploads = supports_file_uploads;
        survey_schema.total_responses = 0;
        survey_schema.average_completion_time = 0;
        survey_schema.export_ipfs_cid = String::new();
        survey_schema.bump = bumps.survey_schema;

        // Initialize data collection statistics
        let data_stats = &mut self.data_stats;
        data_stats.study = self.study.key();
        data_stats.researcher = self.researcher.key();
        data_stats.total_responses = 0;
        data_stats.complete_responses = 0;
        data_stats.validated_responses = 0;
        data_stats.encrypted_responses = 0;
        data_stats.total_files_uploaded = 0;
        data_stats.total_file_size_mb = 0;
        data_stats.average_completion_time_seconds = 0;
        data_stats.first_response_timestamp = 0;
        data_stats.last_response_timestamp = 0;
        data_stats.last_updated = clock.unix_timestamp;
        data_stats.bump = bumps.data_stats;

        // Log schema creation
        msg!(
            "Survey schema created: '{}' with {} questions for study {}",
            survey_title,
            question_count,
            study_id
        );

        // Emit survey schema created event
        emit!(SurveySchemaCreated {
            study_id,
            researcher: self.researcher.key(),
            schema_version: 1,
            question_count,
            estimated_duration: estimated_duration_minutes,
        });

        Ok(())
    }
}

// Survey schema finalization - activates schema for data collection

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct FinalizeSurveySchema<'info> {
    // Study account for validation
    #[account(
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    // Survey schema to finalize
    #[account(
        mut,
        seeds = [b"survey", study.key().as_ref()],
        bump = survey_schema.bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    // Researcher finalizing the schema
    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> FinalizeSurveySchema<'info> {
    // Finalizes survey schema for active data collection
    pub fn finalize_survey_schema(&mut self, study_id: u64) -> Result<()> {
        
        msg!(
            "Survey schema finalized and activated for study {}: '{}'",
            study_id,
            self.survey_schema.title
        );

        Ok(())
    }
}

// Data export - generates research data exports for analysis

#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct ExportSurveyData<'info> {
    // Study account for validation
    #[account(
        seeds = [b"study", study.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study.bump,
        constraint = study.researcher == researcher.key() @ RecruSearchError::UnauthorizedResearcher
    )]
    pub study: Account<'info, StudyAccount>,

    // Survey schema for metadata
    #[account(
        seeds = [b"survey_schema", study.key().as_ref(), survey_schema.schema_version.to_le_bytes().as_ref()],
        bump = survey_schema.bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    // Data collection statistics
    #[account(
        mut,
        seeds = [b"data_stats", study.key().as_ref()],
        bump = data_stats.bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    // Researcher requesting export
    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> ExportSurveyData<'info> {
    // Generates export manifest with survey responses and file uploads
    pub fn export_survey_data(
        &mut self,
        study_id: u64,
        export_format: ExportFormat,
        include_files: bool,
    ) -> Result<ExportManifest> {
        let study = &self.study;
        let schema = &self.survey_schema;
        let stats = &mut self.data_stats;

        // Validate export permissions
        require!(
            matches!(study.status, StudyStatus::Active | StudyStatus::Closed),
            RecruSearchError::InvalidStatusTransition
        );

        // Generate export manifest with IPFS CID
        let export_timestamp = Clock::get()?.unix_timestamp;
        let export_ipfs_cid = format!(
            "ipfs://export-{}-{}-{}",
            study_id,
            export_timestamp,
            export_format as u8
        );
        
        // Create export manifest
        let export_manifest = ExportManifest {
            study_id,
            study_title: study.title.clone(),
            export_timestamp,
            export_format,
            include_files,
            anonymized: false, // Anonymization removed - wallet-based participation is inherently traceable
            schema_version: schema.schema_version,
            total_responses: stats.total_responses,
            complete_responses: stats.complete_responses,
            file_count: stats.total_files_uploaded,
            export_ipfs_cid,
        };

        // Log export details
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

// Export format options for data downloads
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum ExportFormat {
    CSV,
    JSON,
    SPSS,
    R,
    Excel,
}

// Export manifest structure - contains export metadata and statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
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