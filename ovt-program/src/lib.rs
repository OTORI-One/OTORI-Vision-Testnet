mod mock_sdk;
use mock_sdk::{
    prelude::*,
    program::{Program, ProgramContext, ProgramResult, ProgramError},
    state::{Account, AccountInfo},
    token::{Mint, TokenAccount},
};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct OVTProgram;

/// Token metadata stored on-chain
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct TokenMetadata {
    /// Total supply of OVT tokens
    pub total_supply: u64,
    /// Current NAV in satoshis
    pub nav_sats: u64,
    /// Treasury account that holds the fund's assets
    pub treasury: Pubkey,
}

/// SAFE investment data
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Serialize, Deserialize)]
pub struct SAFEData {
    /// Company name
    pub company_name: String,
    /// Investment amount in satoshis
    pub investment_amount_sats: u64,
    /// Valuation cap in satoshis
    pub valuation_cap_sats: u64,
    /// Discount rate as basis points (e.g., 2000 = 20%)
    pub discount_rate_bps: u16,
    /// Unlock timestamp
    pub unlock_time: i64,
    /// Whether the SAFE has been converted to equity
    pub converted: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum OVTInstruction {
    /// Initialize the OVT token
    Initialize {
        initial_supply: u64,
    },
    /// Mint new OVT tokens
    Mint {
        amount: u64,
    },
    /// Burn OVT tokens
    Burn {
        amount: u64,
    },
    /// Add a new SAFE investment
    AddSAFE {
        safe_data: SAFEData,
    },
    /// Update SAFE data (e.g., when converting to equity)
    UpdateSAFE {
        safe_id: u64,
        new_data: SAFEData,
    },
    /// Calculate and update NAV
    UpdateNAV,
    /// Buy OVT tokens with Bitcoin
    BuyOVT {
        /// Amount of OVT to buy in base units
        ovt_amount: u64,
        /// Bitcoin payment proof (for testnet, this will be simulated)
        payment_proof: Vec<u8>,
    },
    /// Sell OVT tokens for Bitcoin
    SellOVT {
        /// Amount of OVT to sell in base units
        ovt_amount: u64,
        /// Bitcoin address to receive payment
        btc_address: String,
    },
}

impl Program for OVTProgram {
    fn process_instruction(ctx: &ProgramContext, data: &[u8]) -> ProgramResult {
        let instruction = OVTInstruction::try_from_slice(data)?;
        
        match instruction {
            OVTInstruction::Initialize { initial_supply } => {
                Self::process_initialize(ctx, initial_supply)
            }
            OVTInstruction::Mint { amount } => {
                Self::process_mint(ctx, amount)
            }
            OVTInstruction::Burn { amount } => {
                Self::process_burn(ctx, amount)
            }
            OVTInstruction::AddSAFE { safe_data } => {
                Self::process_add_safe(ctx, safe_data)
            }
            OVTInstruction::UpdateSAFE { safe_id, new_data } => {
                Self::process_update_safe(ctx, safe_id, new_data)
            }
            OVTInstruction::UpdateNAV => {
                Self::process_update_nav(ctx)
            }
            OVTInstruction::BuyOVT { ovt_amount, payment_proof } => {
                Self::process_buy_ovt(ctx, ovt_amount, payment_proof)
            }
            OVTInstruction::SellOVT { ovt_amount, btc_address } => {
                Self::process_sell_ovt(ctx, ovt_amount, btc_address)
            }
        }
    }
}

impl OVTProgram {
    fn process_initialize(ctx: &ProgramContext, initial_supply: u64) -> ProgramResult {
        let mint_info = ctx.get(0)?;
        let metadata_info = ctx.get(1)?;
        let treasury_info = ctx.get(2)?;
        let authority_info = ctx.get(3)?;

        // Verify authority signature
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Initialize mint account
        let mint = Mint::new(
            mint_info,
            authority_info.key,
            None, // No freeze authority
            6,    // 6 decimals like Bitcoin
        )?;

        // Initialize metadata account
        let metadata = TokenMetadata {
            total_supply: initial_supply,
            nav_sats: 0,
            treasury: treasury_info.key,
        };
        let mut metadata_account = Account { data: metadata };
        metadata_account.set_data(metadata)?;

        // Mint initial supply to treasury
        mint.mint_to(treasury_info, initial_supply)?;

        Ok(())
    }

    fn process_mint(ctx: &ProgramContext, amount: u64) -> ProgramResult {
        let mint_info = ctx.get(0)?;
        let metadata_info = ctx.get(1)?;
        let treasury_info = ctx.get(2)?;
        let authority_info = ctx.get(3)?;

        // Verify authority
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Update total supply
        let mut metadata_account = Account::<TokenMetadata>::get_data(metadata_info)?;
        metadata_account.total_supply = metadata_account.total_supply.checked_add(amount)
            .ok_or(ProgramError::Arithmetic)?;
        metadata_info.set_data(metadata_account)?;

        // Mint tokens
        let mint = Mint::new(
            mint_info,
            authority_info.key,
            None,
            6,
        )?;
        mint.mint_to(treasury_info, amount)?;

        Ok(())
    }

    fn process_burn(ctx: &ProgramContext, amount: u64) -> ProgramResult {
        let mint_info = ctx.accounts.get::<Mint>(0)?;
        let metadata_info = ctx.accounts.get::<Account<TokenMetadata>>(1)?;
        let treasury_info = ctx.accounts.get::<TokenAccount>(2)?;
        let authority_info = ctx.accounts.get::<AccountInfo>(3)?;

        // Verify authority
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Update total supply
        let mut metadata = metadata_info.get_data::<TokenMetadata>()?;
        metadata.total_supply = metadata.total_supply.checked_sub(amount)
            .ok_or(ProgramError::Arithmetic)?;
        metadata_info.set_data(metadata)?;

        // Burn tokens
        let mint = Mint::new(
            mint_info,
            authority_info.key,
            None,
            6,
        )?;
        mint.burn(treasury_info, amount)?;

        Ok(())
    }

    fn process_add_safe(ctx: &ProgramContext, safe_data: SAFEData) -> ProgramResult {
        let safes_info = ctx.accounts.get::<Account<Vec<SAFEData>>>(0)?;
        let authority_info = ctx.accounts.get::<AccountInfo>(1)?;

        // Verify authority
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Add SAFE to the list
        let mut safes = safes_info.get_data::<Vec<SAFEData>>()?;
        safes.push(safe_data);
        safes_info.set_data(safes)?;

        Ok(())
    }

    fn process_update_safe(ctx: &ProgramContext, safe_id: u64, new_data: SAFEData) -> ProgramResult {
        let safes_info = ctx.accounts.get::<Account<Vec<SAFEData>>>(0)?;
        let authority_info = ctx.accounts.get::<AccountInfo>(1)?;

        // Verify authority
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Update SAFE data
        let mut safes = safes_info.get_data::<Vec<SAFEData>>()?;
        if safe_id as usize >= safes.len() {
            return Err(ProgramError::InvalidArgument);
        }
        safes[safe_id as usize] = new_data;
        safes_info.set_data(safes)?;

        Ok(())
    }

    fn process_update_nav(ctx: &ProgramContext) -> ProgramResult {
        let metadata_info = ctx.accounts.get::<Account<TokenMetadata>>(0)?;
        let treasury_info = ctx.accounts.get::<TokenAccount>(1)?;
        let safes_info = ctx.accounts.get::<Account<Vec<SAFEData>>>(2)?;
        let oracle_info = ctx.accounts.get::<AccountInfo>(3)?;

        // Get current liquid assets value from treasury
        let treasury = treasury_info.get_data()?;
        let liquid_value = treasury.amount;

        // Calculate illiquid assets value from SAFEs
        let safes = safes_info.get_data::<Vec<SAFEData>>()?;
        let illiquid_value: u64 = safes.iter()
            .filter(|safe| !safe.converted)
            .map(|safe| safe.investment_amount_sats)
            .sum();

        // Update NAV in metadata
        let mut metadata = metadata_info.get_data::<TokenMetadata>()?;
        metadata.nav_sats = liquid_value.checked_add(illiquid_value)
            .ok_or(ProgramError::Arithmetic)?;
        metadata_info.set_data(metadata)?;

        Ok(())
    }

    fn process_buy_ovt(
        ctx: &ProgramContext,
        ovt_amount: u64,
        payment_proof: Vec<u8>,
    ) -> ProgramResult {
        let mint_info = ctx.accounts.get::<Mint>(0)?;
        let metadata_info = ctx.accounts.get::<Account<TokenMetadata>>(1)?;
        let treasury_info = ctx.accounts.get::<TokenAccount>(2)?;
        let buyer_token_account = ctx.accounts.get::<TokenAccount>(3)?;
        let buyer_info = ctx.accounts.get::<AccountInfo>(4)?;
        let oracle_info = ctx.accounts.get::<AccountInfo>(5)?;

        // Verify buyer signature
        if !buyer_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Get current NAV and calculate price
        let metadata = metadata_info.get_data::<TokenMetadata>()?;
        let price_per_token = metadata.nav_sats.checked_div(metadata.total_supply)
            .ok_or(ProgramError::Arithmetic)?;
        let total_price = price_per_token.checked_mul(ovt_amount)
            .ok_or(ProgramError::Arithmetic)?;

        // For testnet: Verify simulated payment (in production this would verify Bitcoin transaction)
        #[cfg(not(test))]
        {
            // TODO: Implement actual Bitcoin payment verification
            // For now, we'll just check if the payment_proof is not empty
            if payment_proof.is_empty() {
                return Err(ProgramError::InvalidArgument);
            }
        }

        // Mint tokens to buyer
        let mint = Mint::new(
            mint_info,
            ctx.program_id,
            None,
            6,
        )?;
        mint.mint_to(buyer_token_account, ovt_amount)?;

        // Update metadata
        let mut metadata = metadata_info.get_data::<TokenMetadata>()?;
        metadata.total_supply = metadata.total_supply.checked_add(ovt_amount)
            .ok_or(ProgramError::Arithmetic)?;
        metadata_info.set_data(metadata)?;

        Ok(())
    }

    fn process_sell_ovt(
        ctx: &ProgramContext,
        ovt_amount: u64,
        btc_address: String,
    ) -> ProgramResult {
        let mint_info = ctx.accounts.get::<Mint>(0)?;
        let metadata_info = ctx.accounts.get::<Account<TokenMetadata>>(1)?;
        let treasury_info = ctx.accounts.get::<TokenAccount>(2)?;
        let seller_token_account = ctx.accounts.get::<TokenAccount>(3)?;
        let seller_info = ctx.accounts.get::<AccountInfo>(4)?;
        let oracle_info = ctx.accounts.get::<AccountInfo>(5)?;

        // Verify seller signature
        if !seller_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Verify seller has enough tokens
        let seller_token_data = seller_token_account.get_data()?;
        if seller_token_data.amount < ovt_amount {
            return Err(ProgramError::InsufficientFunds);
        }

        // Get current NAV and calculate price
        let metadata = metadata_info.get_data::<TokenMetadata>()?;
        let price_per_token = metadata.nav_sats.checked_div(metadata.total_supply)
            .ok_or(ProgramError::Arithmetic)?;
        let total_price = price_per_token.checked_mul(ovt_amount)
            .ok_or(ProgramError::Arithmetic)?;

        // For testnet: Simulate Bitcoin payment (in production this would initiate a Bitcoin transaction)
        #[cfg(not(test))]
        {
            // TODO: Implement actual Bitcoin payment
            // For now, we'll just log the payment details
            msg!("Simulated BTC payment of {} sats to {}", total_price, btc_address);
        }

        // Burn tokens from seller
        let mint = Mint::new(
            mint_info,
            ctx.program_id,
            None,
            6,
        )?;
        mint.burn_from(seller_token_account, ovt_amount)?;

        // Update metadata
        let mut metadata = metadata_info.get_data::<TokenMetadata>()?;
        metadata.total_supply = metadata.total_supply.checked_sub(ovt_amount)
            .ok_or(ProgramError::Arithmetic)?;
        metadata_info.set_data(metadata)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arch_sdk::test_utils::*;

    #[test]
    fn test_initialize() {
        let mut test_ctx = ProgramTestContext::new();
        
        // Create test accounts
        let mint = test_ctx.create_mint();
        let metadata = test_ctx.create_account::<TokenMetadata>();
        let treasury = test_ctx.create_token_account(&mint.pubkey());
        let authority = test_ctx.create_account_with_lamports(1000000);

        // Initialize OVT token
        let initial_supply = 1_000_000;
        let accounts = vec![
            AccountMeta::new(mint.pubkey(), false),
            AccountMeta::new(metadata.pubkey(), false),
            AccountMeta::new(treasury.pubkey(), false),
            AccountMeta::new_readonly(authority.pubkey(), true),
        ];

        let instruction = OVTInstruction::Initialize { initial_supply };
        test_ctx.process_instruction(
            &OVTProgram,
            accounts,
            &instruction.try_to_vec().unwrap(),
        ).unwrap();

        // Verify initialization
        let metadata_data = test_ctx.get_account_data::<TokenMetadata>(&metadata.pubkey()).unwrap();
        assert_eq!(metadata_data.total_supply, initial_supply);
        
        let treasury_data = test_ctx.get_token_account(&treasury.pubkey()).unwrap();
        assert_eq!(treasury_data.amount, initial_supply);
    }

    #[test]
    fn test_buy_ovt() {
        let mut test_ctx = ProgramTestContext::new();
        
        // Create test accounts
        let mint = test_ctx.create_mint();
        let metadata = test_ctx.create_account::<TokenMetadata>();
        let treasury = test_ctx.create_token_account(&mint.pubkey());
        let buyer = test_ctx.create_account_with_lamports(1000000);
        let buyer_token_account = test_ctx.create_token_account(&mint.pubkey());
        let oracle = test_ctx.create_account::<()>();

        // Initialize with some supply and NAV
        let initial_supply = 1_000_000;
        let initial_nav = 10_000_000; // 10M sats
        let metadata_data = TokenMetadata {
            total_supply: initial_supply,
            nav_sats: initial_nav,
            treasury: treasury.pubkey(),
        };
        test_ctx.set_account_data(&metadata.pubkey(), &metadata_data).unwrap();

        // Buy OVT
        let buy_amount = 1000;
        let payment_proof = vec![1, 2, 3, 4]; // Simulated payment proof
        let accounts = vec![
            AccountMeta::new(mint.pubkey(), false),
            AccountMeta::new(metadata.pubkey(), false),
            AccountMeta::new(treasury.pubkey(), false),
            AccountMeta::new(buyer_token_account.pubkey(), false),
            AccountMeta::new_readonly(buyer.pubkey(), true),
            AccountMeta::new_readonly(oracle.pubkey(), false),
        ];

        let instruction = OVTInstruction::BuyOVT {
            ovt_amount: buy_amount,
            payment_proof,
        };
        test_ctx.process_instruction(
            &OVTProgram,
            accounts,
            &instruction.try_to_vec().unwrap(),
        ).unwrap();

        // Verify purchase
        let buyer_balance = test_ctx.get_token_account(&buyer_token_account.pubkey()).unwrap();
        assert_eq!(buyer_balance.amount, buy_amount);
        
        let metadata_after = test_ctx.get_account_data::<TokenMetadata>(&metadata.pubkey()).unwrap();
        assert_eq!(metadata_after.total_supply, initial_supply + buy_amount);
    }
} 