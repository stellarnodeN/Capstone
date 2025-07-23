#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("6QUDidj9eAByDn5kqSrge47qsHWxv2b6yTxZuqt7sDVU");

#[program]
pub mod recru_search {
    use super::*;

    pub fn create_study(
        ctx: Context<CreateStudy>,
        study_id: u64,
        title: String,
        description: String,
        enrollment_start: i64,
        enrollment_end: i64,
        data_collection_end: i64,
        max_participants: u32,
        reward_amount_per_participant: u64,
    ) -> Result<()> {
        ctx.accounts.create_study(
            study_id,
            title,
            description,
            enrollment_start,
            enrollment_end,
            data_collection_end,
            max_participants,
            reward_amount_per_participant,
            &ctx.bumps,
        )
    }

    pub fn publish_study(ctx: Context<PublishStudy>, study_id: u64) -> Result<()> {
        ctx.accounts.publish_study(study_id)
    }

    pub fn create_reward_vault(
        ctx: Context<CreateRewardVault>,
        study_id: u64,
        initial_deposit: u64,
    ) -> Result<()> {
        ctx.accounts.create_reward_vault(study_id, initial_deposit, &ctx.bumps)
    }

    pub fn mint_consent_nft(
        ctx: Context<MintConsentNFT>,
        study_id: u64,
        eligibility_proof: Option<[u8; 32]>,
        study_title: String,
        study_type: String,
        image_uri: String,
    ) -> Result<()> {
        ctx.accounts.mint_consent_nft(
            study_id,
            eligibility_proof,
            study_title,
            study_type,
            image_uri,
            &ctx.bumps,
        )
    }

    pub fn submit_encrypted_data(
        ctx: Context<SubmitEncryptedData>,
        study_id: u64,
        encrypted_data_hash: [u8; 32],
        ipfs_cid: String,
    ) -> Result<()> {
        ctx.accounts.submit_encrypted_data(study_id, encrypted_data_hash, ipfs_cid, &ctx.bumps)
    }

    pub fn distribute_reward(
        ctx: Context<DistributeReward>,
        study_id: u64,
        study_title: String,
        study_type: String,
        study_duration_days: String,
        image_uri: String,
    ) -> Result<()> {
        ctx.accounts.distribute_reward(
            study_id,
            study_title,
            study_type,
            study_duration_days,
            image_uri,
            &ctx.bumps,
        )
    }

    pub fn close_study(ctx: Context<CloseStudy>, study_id: u64) -> Result<()> {
        ctx.accounts.close_study(study_id)
    }

    pub fn transition_study_state(ctx: Context<TransitionStudyState>, study_id: u64) -> Result<()> {
        ctx.accounts.transition_study_state(study_id)
    }

    pub fn revoke_consent(ctx: Context<RevokeConsent>, study_id: u64) -> Result<()> {
        ctx.accounts.revoke_consent(study_id)
    }
}
