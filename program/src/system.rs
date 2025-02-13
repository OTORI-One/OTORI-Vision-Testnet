use crate::mock_sdk::{
    AccountInfo,
    Pubkey,
    ProgramResult,
};
use borsh::BorshSerialize;

pub fn create_program_account(
    program_id: &Pubkey,
    account_info: &AccountInfo,
    _authority_info: &AccountInfo,
    space: u64,
    _system_program: &AccountInfo,
) -> ProgramResult {
    // Mock implementation for testing
    let mut data = account_info.data.borrow_mut();
    data.resize(space as usize, 0);
    *account_info.owner.borrow_mut() = *program_id;
    Ok(())
}

pub fn initialize_account<T: BorshSerialize>(
    _program_id: &Pubkey,
    account_info: &AccountInfo,
    data: &T,
) -> ProgramResult {
    account_info.set_data(data)
} 