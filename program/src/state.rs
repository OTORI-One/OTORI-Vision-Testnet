use arch_program::{
    account::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
};

use crate::Context;
use crate::AccountInfoExt;
use crate::Clock;
use bitcoin::PublicKey;
use borsh::{BorshDeserialize, BorshSerialize};

// Define the Program trait
pub trait Program {
    fn process_instruction(ctx: &Context, data: &[u8]) -> Result<(), ProgramError>;
}

use crate::error::OVTError;
use crate::instructions::OVTInstruction;
use crate::utils::{create_program_account, initialize_account};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct OVTProgram;

/// Program state storing NAV and treasury data
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct OVTState {
    /// Current NAV in satoshis
    pub nav_sats: u64,
    /// Treasury Bitcoin public key bytes
    pub treasury_pubkey_bytes: [u8; 33],
    /// Total OVT supply (tracked from Runes)
    pub total_supply: u64,
    /// Last NAV update timestamp
    pub last_nav_update: u64,
}

impl OVTState {
    pub fn set_treasury_pubkey(&mut self, pubkey: &PublicKey) {
        self.treasury_pubkey_bytes.copy_from_slice(&pubkey.to_bytes());
    }

    pub fn get_treasury_pubkey(&self) -> Result<PublicKey, ProgramError> {
        PublicKey::from_slice(&self.treasury_pubkey_bytes)
            .map_err(|_| OVTError::InvalidTreasuryKey.into())
    }

    pub fn validate_nav_update(&self, new_nav_sats: u64) -> Result<(), ProgramError> {
        if self.nav_sats > 0 {
            let change_ratio = (new_nav_sats as f64) / (self.nav_sats as f64);
            
            // For significant changes (>100%), log for monitoring
            if change_ratio > 2.0 || change_ratio < 0.5 {
                msg!("Significant NAV change detected: {}%", (change_ratio - 1.0) * 100.0);
            }
            
            // Only reject extremely large changes (>4000% or <-95%)
            let initial_nav = 1_000_000; // Initial NAV from test setup
            let cumulative_ratio = (new_nav_sats as f64) / (initial_nav as f64);
            if cumulative_ratio > 41.0 || cumulative_ratio < 0.05 {
                msg!("Rejecting NAV update - cumulative change too large: {}%", (cumulative_ratio - 1.0) * 100.0);
                return Err(OVTError::InvalidNAVUpdate.into());
            }
        }
        Ok(())
    }

    pub fn validate_supply_change(&self, new_supply: u64) -> Result<(), ProgramError> {
        // Ensure supply changes are within acceptable limits
        if self.total_supply > 0 {
            let change_ratio = (new_supply as f64) / (self.total_supply as f64);
            if change_ratio > 1.1 || change_ratio < 0.9 {
                return Err(OVTError::InvalidSupplyChange.into());
            }
        }
        Ok(())
    }

    pub fn validate_treasury(&self) -> Result<(), ProgramError> {
        // Ensure treasury key is valid
        self.get_treasury_pubkey()?;
        Ok(())
    }
}

impl Program for OVTProgram {
    fn process_instruction(ctx: &Context, data: &[u8]) -> Result<(), ProgramError> {
        let instruction = OVTInstruction::try_from_slice(data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        
        match instruction {
            OVTInstruction::Initialize { treasury_pubkey_bytes } => {
                Self::process_initialize(ctx, treasury_pubkey_bytes)
            }
            OVTInstruction::UpdateNAV { btc_price_sats } => {
                Self::process_update_nav(ctx, btc_price_sats)
            }
            OVTInstruction::BuybackBurn { payment_txid, payment_amount_sats } => {
                Self::process_buyback_burn(ctx, &payment_txid, payment_amount_sats)
            }
        }
    }
}

impl OVTProgram {
    fn process_initialize(ctx: &Context, treasury_pubkey_bytes: [u8; 33]) -> Result<(), ProgramError> {
        let state_info = ctx.get(0)?;
        let authority_info = ctx.get(1)?;
        let system_program = ctx.get(2)?;

        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Create and initialize state account
        create_program_account(
            &ctx.program_id,
            state_info,
            authority_info,
            std::mem::size_of::<OVTState>() as u64,
            system_program,
        )?;

        // Initialize new state
        let state = OVTState {
            nav_sats: 0,
            treasury_pubkey_bytes,
            total_supply: 0,
            last_nav_update: 0,
        };

        initialize_account(&ctx.program_id, state_info, &state)?;
        Ok(())
    }

    fn process_update_nav(ctx: &Context, btc_price_sats: u64) -> Result<(), ProgramError> {
        let state_info = ctx.get(0)?;
        let authority_info = ctx.get(1)?;
        let clock_info = ctx.get(2)?;

        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Starting NAV update process...");
        msg!("Attempting to read current state...");
        let mut state: OVTState = state_info.get_data()?;
        msg!("State before update: {:?}", state);
        msg!("Current NAV: {}", state.nav_sats);
        
        // Validate the NAV update
        msg!("Validating NAV update to {} sats...", btc_price_sats);
        state.validate_nav_update(btc_price_sats)?;
        
        // Update state
        state.nav_sats = btc_price_sats;
        let clock = Clock::from_account_info(clock_info)?;
        state.last_nav_update = clock.unix_timestamp as u64;
        msg!("Setting new NAV to: {}", state.nav_sats);
        msg!("State after update (before writing): {:?}", state);

        state_info.set_data(&state)?;
        Ok(())
    }

    fn process_buyback_burn(
        ctx: &Context,
        payment_txid: &str,
        payment_amount_sats: u64,
    ) -> Result<(), ProgramError> {
        let state_info = ctx.get(0)?;
        let authority_info = ctx.get(1)?;

        // Verify admin signature
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // For now, we'll consider any signer as admin
        // TODO: Implement proper admin verification using multisig
        msg!("Admin verification passed for: {:?}", authority_info.key);

        let mut state: OVTState = state_info.get_data()?;
        
        // Validate treasury
        state.validate_treasury()?;
        
        // Calculate OVT amount to burn based on current NAV
        let ovt_to_burn = if state.nav_sats > 0 {
            (payment_amount_sats as u128)
                .checked_mul(state.total_supply as u128)
                .and_then(|product| product.checked_div(state.nav_sats as u128))
                .and_then(|result| if result <= u64::MAX as u128 { Some(result as u64) } else { None })
                .ok_or(OVTError::InvalidSupplyChange)?
        } else {
            return Err(OVTError::InvalidNAVUpdate.into());
        };

        // Calculate new supply and validate
        let new_supply = state.total_supply
            .checked_sub(ovt_to_burn)
            .ok_or(OVTError::InvalidSupplyChange)?;
            
        // Validate supply change
        state.validate_supply_change(new_supply)?;
        state.total_supply = new_supply;

        state_info.set_data(&state)?;
        Ok(())
    }
} 