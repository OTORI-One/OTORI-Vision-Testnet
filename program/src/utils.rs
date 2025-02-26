use arch_program::{
    program_error::ProgramError,
    account::AccountInfo,
    pubkey::Pubkey,
};
use borsh::BorshSerialize;
use crate::AccountInfoExt;

pub fn create_program_account(
    _program_id: &Pubkey,
    _account: &AccountInfo,
    _payer: &AccountInfo,
    _size: u64,
    _system_program: &AccountInfo,
) -> Result<(), ProgramError> {
    // In a real implementation, this would create an account
    // For testing, we'll assume the account already exists
    Ok(())
}

pub fn initialize_account<T: BorshSerialize>(
    _program_id: &Pubkey,
    account: &AccountInfo,
    data: &T,
) -> Result<(), ProgramError> {
    // Initialize the account with the provided data
    account.set_data(data)
} 