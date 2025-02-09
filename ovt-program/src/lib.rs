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
        
        // Verify UTXO ownership and payment
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

        // Update state with new supply
        let mut new_state = state;
        new_state.total_supply = new_state.total_supply
            .checked_sub(ovt_to_burn)
            .ok_or(OVTError::InvalidSupplyChange)?;

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
} 