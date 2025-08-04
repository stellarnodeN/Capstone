use anchor_lang::prelude::*;
use crate::state::{AdminAccount, RecruSearchError};
use crate::state::constants::{DEFAULT_PROTOCOL_FEE_BPS, MAX_PROTOCOL_FEE_BPS, MIN_STUDY_DURATION, MAX_STUDY_DURATION};

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(
        init,
        payer = protocol_admin,
        space = 8 + AdminAccount::INIT_SPACE,
        seeds = [b"admin"],
        bump
    )]
    pub admin_state: Account<'info, AdminAccount>,

    #[account(mut)]
    pub protocol_admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeProtocol<'info> {
    pub fn initialize_protocol(
        &mut self,
        protocol_fee_basis_points: Option<u16>,
        min_study_duration: Option<u32>,
        max_study_duration: Option<u32>,
        bumps: &InitializeProtocolBumps,
    ) -> Result<()> {
        let validated_config = self.validate_protocol_config(
            protocol_fee_basis_points,
            min_study_duration,
            max_study_duration,
        )?;

        self.initialize_admin_state(validated_config, bumps)?;

        self.log_protocol_initialization()?;

        Ok(())
    }

    fn validate_protocol_config(
        &self,
        protocol_fee_basis_points: Option<u16>,
        min_study_duration: Option<u32>,
        max_study_duration: Option<u32>,
    ) -> Result<ProtocolConfig> {
        let fee_bps = protocol_fee_basis_points.unwrap_or(DEFAULT_PROTOCOL_FEE_BPS);
        require!(
            fee_bps <= MAX_PROTOCOL_FEE_BPS,
            RecruSearchError::ExcessiveProtocolFee
        );

        let min_duration = min_study_duration.unwrap_or(MIN_STUDY_DURATION as u32);
        let max_duration = max_study_duration.unwrap_or(MAX_STUDY_DURATION as u32);
        
        require!(
            max_duration > min_duration,
            RecruSearchError::InvalidDataCollectionPeriod
        );

        Ok(ProtocolConfig {
            protocol_fee_basis_points: fee_bps,
            min_study_duration: min_duration,
            max_study_duration: max_duration,
        })
    }

    fn initialize_admin_state(
        &mut self,
        config: ProtocolConfig,
        bumps: &InitializeProtocolBumps,
    ) -> Result<()> {
        let admin_state = &mut self.admin_state;

        admin_state.protocol_admin = self.protocol_admin.key();
        admin_state.protocol_fee_bps = config.protocol_fee_basis_points;
        admin_state.min_study_duration = config.min_study_duration as u64;
        admin_state.max_study_duration = config.max_study_duration as u64;
        admin_state.total_studies = 0;
        admin_state.total_participants = 0;
        admin_state.total_rewards_distributed = 0;
        admin_state.bump = bumps.admin_state;

        Ok(())
    }

    fn log_protocol_initialization(&self) -> Result<()> {
        let admin_state = &self.admin_state;
        
        msg!("Protocol initialized | Admin: {} | Fee: {}% | Status: Active", 
             admin_state.protocol_admin, 
             admin_state.protocol_fee_bps as f64 / 100.0
        );

        Ok(())
    }
}

#[derive(Debug)]
struct ProtocolConfig {
    pub protocol_fee_basis_points: u16,
    pub min_study_duration: u32,
    pub max_study_duration: u32,
} 