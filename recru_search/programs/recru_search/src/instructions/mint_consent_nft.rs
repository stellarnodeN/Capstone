use anchor_lang::prelude::*;

use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::{CreateV2, CreateV2InstructionArgs},
    types::DataState,
};
use crate::state::{
    study::{StudyAccount, StudyStatus},
    consent_nft::ConsentNFTAccount,
};
use crate::error::RecruSearchError;

// Seed prefixes for account PDAs
const STUDY_SEED_PREFIX: &str = "study";
const CONSENT_SEED_PREFIX: &str = "consent";

// Account validation struct for minting consent NFTs
// This defines what accounts must be provided when a participant joins a study
#[derive(Accounts)]
#[instruction(study_id: u64)]
pub struct MintConsentNFT<'info> {
    #[account(
        mut,
        seeds = [STUDY_SEED_PREFIX.as_bytes(), study_account.researcher.as_ref(), study_id.to_le_bytes().as_ref()],
        bump = study_account.bump
    )]
    pub study_account: Account<'info, StudyAccount>,

    #[account(
        init,
        payer = participant,
        space = 8 + ConsentNFTAccount::INIT_SPACE,
        seeds = [CONSENT_SEED_PREFIX.as_bytes(), study_account.key().as_ref(), participant.key().as_ref()],
        bump
    )]
    pub consent_nft_account: Account<'info, ConsentNFTAccount>,

    /// Core NFT Asset account (single account design)
    #[account(mut)]
    pub core_asset: Signer<'info>,

    #[account(mut)]
    pub participant: Signer<'info>,

    pub system_program: Program<'info, System>,
    
    /// CHECK: Metaplex Core Program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> MintConsentNFT<'info> {
    pub fn mint_consent_nft(
        &mut self,
        study_id: u64,
        eligibility_proof: Option<[u8; 32]>, // ZK proof for eligibility verification
        study_title: String,                 // Study name for NFT metadata
        study_type: String,                  // Research category for NFT metadata
        image_uri: String,                   // Image URI for NFT artwork
        bumps: &MintConsentNFTBumps,
    ) -> Result<()> {
        let study_account = &mut self.study_account;
        let consent_nft_account = &mut self.consent_nft_account;
        let clock = Clock::get()?;

        // Validate study status allows enrollment
        require!(
            matches!(study_account.status, StudyStatus::Published),
            RecruSearchError::InvalidStatusTransition
        );

        // Validate enrollment window is currently open
        require!(
            clock.unix_timestamp >= study_account.enrollment_start 
                && clock.unix_timestamp <= study_account.enrollment_end,
            RecruSearchError::EnrollmentClosed
        );

        // Validate enrollment hasn't reached max participants
        require!(
            study_account.enrolled_count < study_account.max_participants,
            RecruSearchError::MaxParticipantsReached
        );

        // Validate ZK proof if required by the study
        if study_account.requires_zk_proof {
            require!(
                eligibility_proof.is_some(),
                RecruSearchError::MissingEligibilityProof
            );
        }

        // Validate metadata input lengths to prevent oversized data
        require!(
            study_title.len() <= 100,
            RecruSearchError::InvalidDataFormat
        );
        require!(
            study_type.len() <= 50,
            RecruSearchError::InvalidDataFormat
        );
        require!(
            image_uri.len() <= 200,
            RecruSearchError::InvalidDataFormat
        );

        // Initialize consent NFT account with enhanced metadata
        consent_nft_account.study_id = study_id;
        consent_nft_account.participant = self.participant.key();
        consent_nft_account.study_account = study_account.key();
        consent_nft_account.consent_timestamp = clock.unix_timestamp;
        consent_nft_account.eligibility_proof_hash = eligibility_proof.unwrap_or([0u8; 32]);
        consent_nft_account.consent_document_version = study_account.consent_document_hash;
        
        // Enhanced metadata fields for Metaplex compatibility
        consent_nft_account.study_title = study_title.clone();
        consent_nft_account.study_type = study_type;
        consent_nft_account.image_uri = image_uri;
        consent_nft_account.privacy_level = if study_account.requires_zk_proof {
            "Zero-Knowledge".to_string()
        } else {
            "Pseudonymous".to_string()
        };
        
        // Metadata URI will be populated by frontend after uploading JSON to IPFS
        consent_nft_account.metadata_uri = "".to_string();
        
        consent_nft_account.is_revoked = false;
        consent_nft_account.bump = bumps.consent_nft_account;

        // Mint the consent NFT to the participant's wallet
        let study_key = study_account.key();
        let participant_key = self.participant.key();
        let consent_nft_seeds = &[
            CONSENT_SEED_PREFIX.as_bytes(),
            study_key.as_ref(),
            participant_key.as_ref(),
            &[bumps.consent_nft_account],
        ];
        let signer_seeds = &[&consent_nft_seeds[..]];

        // Create Core NFT Asset using Metaplex Core (single account design)
        let nft_name = format!("Consent - {}", &study_title);
        let nft_uri = format!("https://recrusearch.com/metadata/consent/{}", study_id);

        // Use Metaplex Core to create the NFT asset manually
        let create_ix = CreateV2 {
            asset: self.core_asset.key(),
            collection: None,
            authority: None,
            payer: self.participant.key(),
            owner: Some(self.participant.key()),
            update_authority: Some(consent_nft_account.key()),
            system_program: self.system_program.key(),
            log_wrapper: None,
        }.instruction(CreateV2InstructionArgs {
            data_state: DataState::AccountState,
            name: nft_name,
            uri: nft_uri,
            plugins: None,
            external_plugin_adapters: None,
        });

        anchor_lang::solana_program::program::invoke_signed(
            &create_ix,
            &[
                self.core_asset.to_account_info(),
                self.participant.to_account_info(),
                consent_nft_account.to_account_info(),
                self.system_program.to_account_info(),
                self.mpl_core_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        // Update study participant count
        study_account.enrolled_count = study_account.enrolled_count
            .checked_add(1)
            .ok_or(RecruSearchError::MathOverflow)?;

        msg!(
            "Consent NFT minted for participant {} in study '{}' (ID: {})",
            self.participant.key(),
            consent_nft_account.study_title,
            study_id
        );

        Ok(())
    }
} 