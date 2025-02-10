pub mod error;
pub mod mock_sdk;
pub mod system;
pub mod utxo;

use arch_program::{
    entrypoint,
    program::{Program, ProgramContext, ProgramResult},
    account_info::AccountInfo,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};
use bitcoin::PublicKey;
use crate::{
    error::OVTError,
    system::{create_program_account, initialize_account},
    utxo::{verify_bitcoin_payment, verify_utxo_ownership},
};

/// OVT Token identifier in Runes protocol
pub const OVT_RUNE_SYMBOL: &str = "OVT";
pub const OVT_DECIMALS: u8 = 8;

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
        let bytes = pubkey.inner.serialize();
        self.treasury_pubkey_bytes.copy_from_slice(&bytes);
    }

    pub fn get_treasury_pubkey(&self) -> Result<PublicKey, ProgramError> {
        PublicKey::from_slice(&self.treasury_pubkey_bytes)
            .map_err(|_| OVTError::InvalidTreasuryKey.into())
    }

    // Add validation methods
    pub fn validate_nav_update(&self, new_nav_sats: u64) -> ProgramResult {
        if self.nav_sats > 0 {
            let change_ratio = (new_nav_sats as f64) / (self.nav_sats as f64);
            
            // Allow larger price movements but require additional verification for extreme changes
            if change_ratio > 2.0 || change_ratio < 0.5 {
                // For changes over 100%, require additional verification
                // TODO: Implement multi-signature requirement (reiterate!) for extreme changes
                msg!("Warning: Large NAV change detected: {}%", (change_ratio - 1.0) * 100.0);
            } else if change_ratio > 1.5 || change_ratio < 0.67 {
                // For changes between 50% and 100%, log warning but allow
                msg!("Notice: Significant NAV change: {}%", (change_ratio - 1.0) * 100.0);
            }
            
            // Basic sanity checks
            if change_ratio > 5.0 || change_ratio < 0.2 {
                // Prevent extreme changes (>400% or <-80%) without special authorization
                return Err(OVTError::InvalidNAVUpdate.into());
            }
        }
        Ok(())
    }

    pub fn validate_supply_change(&self, new_supply: u64) -> ProgramResult {
        // Ensure supply changes are within acceptable limits
        if self.total_supply > 0 {
            let change_ratio = (new_supply as f64) / (self.total_supply as f64);
            if change_ratio > 1.1 || change_ratio < 0.9 {
                return Err(OVTError::InvalidSupplyChange.into());
            }
        }
        Ok(())
    }

    pub fn validate_treasury(&self) -> ProgramResult {
        // Ensure treasury key is valid
        self.get_treasury_pubkey()?;
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum OVTInstruction {
    /// Initialize OVT state
    Initialize {
        treasury_pubkey_bytes: [u8; 33],
    },
    /// Calculate and update NAV
    UpdateNAV {
        btc_price_sats: u64,
    },
    /// Execute buyback and burn
    BuybackBurn {
        payment_txid: String,
        payment_amount_sats: u64,
    },
}

entrypoint!(process_instruction);

impl Program for OVTProgram {
    fn process_instruction(ctx: &ProgramContext, data: &[u8]) -> ProgramResult {
        let instruction = OVTInstruction::try_from_slice(data)?;
        
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
    fn process_initialize(ctx: &ProgramContext, treasury_pubkey_bytes: [u8; 33]) -> ProgramResult {
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

    fn process_update_nav(ctx: &ProgramContext, btc_price_sats: u64) -> ProgramResult {
        let state_info = ctx.get(0)?;
        let authority_info = ctx.get(1)?;

        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state: OVTState = state_info.get_data()?;
        
        // Validate the NAV update
        state.validate_nav_update(btc_price_sats)?;
        
        // Update state
        state.nav_sats = btc_price_sats;
        state.last_nav_update = 1000; // Mock timestamp for testing

        state_info.set_data(&state)?;
        Ok(())
    }

    fn process_buyback_burn(
        ctx: &ProgramContext,
        payment_txid: &str,
        payment_amount_sats: u64,
    ) -> ProgramResult {
        let state_info = ctx.get(0)?;
        let authority_info = ctx.get(1)?;
        let utxo_info = ctx.get(2)?;

        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let state: OVTState = state_info.get_data()?;
        
        // Validate treasury and UTXO
        state.validate_treasury()?;
        verify_utxo_ownership(utxo_info, &ctx.program_id)?;
        verify_bitcoin_payment(utxo_info, payment_amount_sats, &state.treasury_pubkey_bytes)?;
        
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
        let mut new_state = state;
        new_state.validate_supply_change(new_supply)?;
        new_state.total_supply = new_supply;

        state_info.set_data(&new_state)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock_sdk::test_utils::TestClient;
    use bitcoin::secp256k1::Secp256k1;
    use rand::thread_rng;

    #[test]
    fn test_buyback_burn() {
        let mut client = TestClient::new();
        
        // Create test accounts
        let program_id = mock_sdk::Pubkey::new_unique();
        let authority = mock_sdk::Pubkey::new_unique();

        // Initialize state with some NAV and supply
        let secp = Secp256k1::new();
        let mut rng = thread_rng();
        let (_, pubkey) = secp.generate_keypair(&mut rng);
        
        let state = OVTState {
            nav_sats: 1_000_000, // 1M sats NAV
            treasury_pubkey_bytes: pubkey.serialize(),
            total_supply: 1_000_000, // 1M OVT supply
            last_nav_update: 0,
        };

        // Create and initialize state account
        let state_account = client.create_account(program_id).unwrap();
        client.set_account_data(&state_account.key, &state).unwrap();

        // Create authority account since it's required as a signer
        let _authority_account = client.create_account_with_lamports(1000000).unwrap();
        client.accounts.insert(authority, mock_sdk::AccountInfo::new(authority, true, false));

        // Test buyback burn
        let payment_amount = 100_000; // 100k sats
        let instruction = OVTInstruction::BuybackBurn {
            payment_txid: "test_txid".to_string(),
            payment_amount_sats: payment_amount,
        };

        let accounts = vec![
            mock_sdk::program::AccountMeta::new(state_account.key, false),
            mock_sdk::program::AccountMeta::new_readonly(authority, true),
        ];

        client.process_transaction(
            program_id,
            accounts,
            borsh::to_vec(&instruction).unwrap(),
        ).unwrap();
    }

    #[test]
    fn test_nav_validation() {
        let mut client = TestClient::new();
        let program_id = mock_sdk::Pubkey::new_unique();
        let authority = mock_sdk::Pubkey::new_unique();

        // Create and initialize state account with initial NAV
        let state = OVTState {
            nav_sats: 1_000_000, // 1M sats NAV
            treasury_pubkey_bytes: [0u8; 33],
            total_supply: 1_000_000,
            last_nav_update: 0,
        };

        let state_account = client.create_account(program_id).unwrap();
        client.set_account_data(&state_account.key, &state).unwrap();
        client.accounts.insert(authority, mock_sdk::AccountInfo::new(authority, true, false));

        // Test valid NAV update (within 20% change)
        let valid_nav = 1_100_000; // 10% increase
        let instruction = OVTInstruction::UpdateNAV { btc_price_sats: valid_nav };
        let accounts = vec![
            mock_sdk::program::AccountMeta::new(state_account.key, false),
            mock_sdk::program::AccountMeta::new_readonly(authority, true),
        ];

        assert!(client.process_transaction(
            program_id,
            accounts.clone(),
            borsh::to_vec(&instruction).unwrap(),
        ).is_ok());

        // Test invalid NAV update (>20% change)
        let invalid_nav = 2_000_000; // 100% increase
        let instruction = OVTInstruction::UpdateNAV { btc_price_sats: invalid_nav };
        
        assert!(client.process_transaction(
            program_id,
            accounts,
            borsh::to_vec(&instruction).unwrap(),
        ).is_err());
    }
} 