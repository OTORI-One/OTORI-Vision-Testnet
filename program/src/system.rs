use crate::mock_sdk::{
    AccountInfo,
    Pubkey,
    ProgramResult,
};
use borsh::BorshSerialize;

pub fn create_program_account(
    _program_id: &Pubkey,
    _account_info: &AccountInfo,
    _authority_info: &AccountInfo,
    _space: u64,
    _system_program: &AccountInfo,
) -> ProgramResult {
    // Mock implementation for testing
    Ok(())
}

pub fn initialize_account<T: BorshSerialize>(
    _program_id: &Pubkey,
    account_info: &AccountInfo,
    data: &T,
) -> ProgramResult {
    account_info.set_data(data)
} 