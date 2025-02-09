use arch_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};
use crate::error::OVTError;

pub fn create_program_account(
    program_id: &Pubkey,
    new_account: &AccountInfo,
    payer: &AccountInfo,
    space: u64,
    system_program: &AccountInfo,
) -> Result<(), ProgramError> {
    // Create account instruction
    let create_account_ix = system_instruction::create_account(
        &payer.key,
        &new_account.key,
        space,
        program_id,
    );

    // Invoke system program
    arch_program::program::invoke(
        &create_account_ix,
        &[
            payer.clone(),
            new_account.clone(),
            system_program.clone(),
        ],
    )?;

    Ok(())
}

pub fn initialize_account<T: borsh::BorshSerialize>(
    program_id: &Pubkey,
    account: &AccountInfo,
    data: &T,
) -> Result<(), ProgramError> {
    if !account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    if account.owner != program_id {
        return Err(OVTError::InvalidTreasuryKey.into());
    }

    let serialized = borsh::to_vec(data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    let mut account_data = account.data.borrow_mut();
    account_data[..serialized.len()].copy_from_slice(&serialized);

    Ok(())
} 