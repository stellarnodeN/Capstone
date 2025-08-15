use anchor_lang::prelude::*;
use crate::state::*;

// defines data collection structure for studies

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

    // Researcher creating the schema
    #[account(mut)]
    pub researcher: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateSurveySchema<'info> {
    // Creates survey schema and initializes basic data collection tracking
    pub fn create_survey_schema(
        &mut self,
        study_id: u64,
        survey_title: String,
        schema_ipfs_cid: String,
        requires_encryption: bool,
        bumps: &CreateSurveySchemaBumps,
    ) -> Result<()> {
        
        require!(
            survey_title.len() >= 5 && survey_title.len() <= 100,
            RecruSearchError::TitleTooLong
        );

        // Basic IPFS CID validation (length only)
        require!(
            schema_ipfs_cid.len() >= 10 && schema_ipfs_cid.len() <= 100,
            RecruSearchError::InvalidIPFSCID
        );

       
        let survey_schema = &mut self.survey_schema;
        survey_schema.study = self.study.key();
        survey_schema.title = survey_title.clone();
        survey_schema.schema_ipfs_cid = schema_ipfs_cid;
        survey_schema.requires_encryption = requires_encryption;
        survey_schema.bump = bumps.survey_schema;

       
        let data_stats = &mut self.data_stats;
        data_stats.study = self.study.key();
        data_stats.researcher = self.researcher.key();
        data_stats.total_responses = 0;
        data_stats.complete_responses = 0;
        data_stats.bump = bumps.data_stats;

        msg!(
            "Survey schema created: '{}' for study {}",
            survey_title,
            study_id
        );

        // Emit survey schema created event
        emit!(SurveySchemaCreated {
            study_id,
            researcher: self.researcher.key(),
        });

        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct FinalizeSurveySchema<'info> {
    // Study account for validation
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

// Simple data export - generates basic export metadata

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
        seeds = [b"survey", study.key().as_ref()],
        bump = survey_schema.bump
    )]
    pub survey_schema: Account<'info, SurveySchema>,

    // Data collection statistics
    #[account(
        seeds = [b"data_stats", study.key().as_ref()],
        bump = data_stats.bump
    )]
    pub data_stats: Account<'info, DataCollectionStats>,

    // Researcher requesting export
    #[account(mut)]
    pub researcher: Signer<'info>,
}

impl<'info> ExportSurveyData<'info> {
    // Generates basic export metadata
    pub fn export_survey_data(
        &mut self,
        study_id: u64,
    ) -> Result<ExportManifest> {
        let study = &self.study;
        let stats = &self.data_stats;

       
        require!(
            matches!(study.status, StudyStatus::Active | StudyStatus::Closed),
            RecruSearchError::InvalidStatusTransition
        );

     
        let export_manifest = ExportManifest {
            study_id,
            study_title: study.title.clone(),
            total_responses: stats.total_responses,
            complete_responses: stats.complete_responses,
        };

      
        msg!(
            "Data export initiated for study {}: '{}' ({} responses)",
            study_id,
            study.title,
            stats.total_responses
        );

        Ok(export_manifest)
    }
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExportManifest {
    pub study_id: u64,
    pub study_title: String,
    pub total_responses: u32,
    pub complete_responses: u32,
}