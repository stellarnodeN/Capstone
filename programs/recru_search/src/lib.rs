#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;

declare_id!("HL4vrf5EV4eeaWyDLdzRgdjxxLiPfxiBvpWqjtKBPBNR");

#[program]
pub mod recru_search {
    use super::*;

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>, protocol_fee_basis_points: Option<u16>, min_study_duration: Option<u32>, max_study_duration: Option<u32>) -> Result<()> {
        ctx.accounts.initialize_protocol(protocol_fee_basis_points, min_study_duration, max_study_duration, &ctx.bumps)
    }

    pub fn create_study(ctx: Context<CreateStudy>, study_id: u64, title: String, description: String, enrollment_start: i64, enrollment_end: i64, data_collection_end: i64, max_participants: u32, reward_amount: u64) -> Result<()> {
        ctx.accounts.create_study(study_id, title, description, enrollment_start, enrollment_end, data_collection_end, max_participants, reward_amount, &ctx.bumps)
    }

    pub fn publish_study(ctx: Context<PublishStudy>) -> Result<()> {
        ctx.accounts.publish_study()
    }

    pub fn close_study(ctx: Context<CloseStudy>) -> Result<()> {
        ctx.accounts.close_study()
    }

    pub fn transition_study_state(ctx: Context<TransitionStudyState>) -> Result<()> {
        ctx.accounts.transition_study_state()
    }

    pub fn set_eligibility_criteria(ctx: Context<SetEligibilityCriteria>, study_id: u64, criteria: Vec<u8>) -> Result<()> {
        ctx.accounts.set_eligibility_criteria(study_id, criteria)
    }

    pub fn verify_eligibility(ctx: Context<VerifyEligibility>, study_id: u64, participant_info: Vec<u8>) -> Result<bool> {
        let info: eligibility_criteria::ParticipantInfo = eligibility_criteria::ParticipantInfo::try_from_slice(&participant_info).map_err(|_| RecruSearchError::InvalidParameterValue)?;
        ctx.accounts.verify_eligibility(study_id, info)
    }



    pub fn mint_consent_nft(ctx: Context<MintConsentNFT>, study_id: u64, eligibility_proof: Vec<u8>) -> Result<()> {
        ctx.accounts.mint_consent_nft(study_id, eligibility_proof)
    }

    pub fn revoke_consent(ctx: Context<RevokeConsent>) -> Result<()> {
        ctx.accounts.revoke_consent()
    }

    pub fn submit_data(ctx: Context<SubmitData>, encrypted_data_hash: [u8; 32], ipfs_cid: String) -> Result<()> {
        ctx.accounts.submit_data(encrypted_data_hash, ipfs_cid, &ctx.bumps)
    }

    pub fn mint_completion_nft(ctx: Context<MintCompletionNFT>) -> Result<()> {
        ctx.accounts.mint_completion_nft()
    }

    pub fn create_reward_vault(ctx: Context<CreateRewardVault>, study_id: u64, initial_deposit: u64) -> Result<()> {
        ctx.accounts.create_reward_vault(study_id, initial_deposit, &ctx.bumps)
    }

    pub fn distribute_reward(ctx: Context<DistributeReward>) -> Result<()> {
        ctx.accounts.distribute_reward(&ctx.bumps)
    }

    pub fn create_survey_schema(ctx: Context<CreateSurveySchema>, study_id: u64, survey_title: String, survey_description: String, question_count: u32, estimated_duration_minutes: u32, schema_ipfs_cid: String, requires_encryption: bool, supports_file_uploads: bool) -> Result<()> {
        ctx.accounts.create_survey_schema(study_id, survey_title, survey_description, question_count, estimated_duration_minutes, schema_ipfs_cid, requires_encryption, supports_file_uploads, &ctx.bumps)
    }

    pub fn finalize_survey_schema(ctx: Context<FinalizeSurveySchema>, study_id: u64) -> Result<()> {
        ctx.accounts.finalize_survey_schema(study_id)
    }

    pub fn export_survey_data(ctx: Context<ExportSurveyData>, study_id: u64, export_format: data_management::ExportFormat, include_files: bool) -> Result<data_management::ExportManifest> {
        ctx.accounts.export_survey_data(study_id, export_format, include_files)
    }






}