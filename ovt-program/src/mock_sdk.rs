use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::io;

// Basic types we need
pub type Pubkey = [u8; 32];
pub type Signature = [u8; 64];

// Program context and result types
pub struct ProgramContext {
    pub accounts: Vec<AccountInfo>,
    pub program_id: Pubkey,
}

#[derive(Debug)]
pub enum ProgramError {
    InvalidArgument,
    InvalidInstructionData,
    InvalidAccountData,
    AccountNotFound,
    InsufficientFunds,
    MissingRequiredSignature,
    Custom(String),
    Arithmetic,
}

impl From<io::Error> for ProgramError {
    fn from(_err: io::Error) -> Self {
        ProgramError::InvalidInstructionData
    }
}

pub type ProgramResult<T = ()> = Result<T, ProgramError>;

// Account types
#[derive(Clone)]
pub struct AccountInfo {
    pub key: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
}

pub struct Mint {
    pub authority: Pubkey,
    pub supply: u64,
    pub decimals: u8,
}

pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
}

// Trait definitions
pub trait Program {
    fn process_instruction(ctx: &ProgramContext, data: &[u8]) -> ProgramResult;
}

// Implementation helpers
impl ProgramContext {
    pub fn get<T>(&self, index: usize) -> Result<&AccountInfo, ProgramError> {
        self.accounts.get(index).ok_or(ProgramError::AccountNotFound)
    }
}

pub mod prelude {
    pub use super::*;
    pub use super::program::*;
    pub use super::state::*;
    pub use super::token::*;
}

pub mod program {
    use super::*;

    pub use super::{Program, ProgramContext, ProgramResult, ProgramError};

    #[derive(Clone, Debug)]
    pub struct Instruction {
        pub program_id: Pubkey,
        pub accounts: Vec<AccountMeta>,
        pub data: Vec<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct AccountMeta {
        pub pubkey: Pubkey,
        pub is_signer: bool,
        pub is_writable: bool,
    }

    impl AccountMeta {
        pub fn new(pubkey: Pubkey, is_writable: bool) -> Self {
            Self {
                pubkey,
                is_signer: false,
                is_writable,
            }
        }

        pub fn new_readonly(pubkey: Pubkey, is_signer: bool) -> Self {
            Self {
                pubkey,
                is_signer,
                is_writable: false,
            }
        }
    }

    impl Instruction {
        pub fn new_with_borsh(
            program_id: Pubkey,
            data: &impl BorshSerialize,
            accounts: Vec<AccountMeta>,
        ) -> Self {
            let data = data.try_to_vec().unwrap();
            Self {
                program_id,
                accounts,
                data,
            }
        }
    }
}

pub mod state {
    use super::*;

    #[derive(Clone)]
    pub struct Account<T> {
        pub data: T,
    }

    impl<T> Account<T> {
        pub fn get_data<U: BorshDeserialize>(&self) -> Result<U, ProgramError> {
            // In a real implementation, this would deserialize from bytes
            Err(ProgramError::Custom("Not implemented".to_string()))
        }

        pub fn set_data<U: BorshSerialize>(&mut self, _data: U) -> Result<(), ProgramError> {
            // In a real implementation, this would serialize to bytes
            Ok(())
        }
    }
}

pub mod token {
    use super::*;

    impl Mint {
        pub fn new(
            _info: &AccountInfo,
            authority: Pubkey,
            _freeze_authority: Option<Pubkey>,
            decimals: u8,
        ) -> Result<Self, ProgramError> {
            Ok(Self {
                authority,
                supply: 0,
                decimals,
            })
        }

        pub fn mint_to(&self, _account: &AccountInfo, _amount: u64) -> Result<(), ProgramError> {
            // In a real implementation, this would update token accounts
            Ok(())
        }

        pub fn burn(&self, _account: &AccountInfo, _amount: u64) -> Result<(), ProgramError> {
            // In a real implementation, this would update token accounts
            Ok(())
        }

        pub fn burn_from(&self, _account: &AccountInfo, _amount: u64) -> Result<(), ProgramError> {
            // In a real implementation, this would update token accounts
            Ok(())
        }
    }

    impl TokenAccount {
        pub fn get_data(&self) -> Result<Self, ProgramError> {
            Ok(self.clone())
        }
    }
}

// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    
    pub struct ProgramTestContext {
        pub accounts: Vec<AccountInfo>,
    }
    
    impl ProgramTestContext {
        pub fn new() -> Self {
            Self {
                accounts: Vec::new(),
            }
        }
        
        pub fn create_mint(&mut self) -> AccountInfo {
            // Create a test mint account
            let key = [1u8; 32];
            AccountInfo {
                key,
                is_signer: false,
                is_writable: true,
                lamports: 1000000,
                data: Vec::new(),
                owner: [0u8; 32],
            }
        }
        
        pub fn create_token_account(&mut self, mint: &Pubkey) -> AccountInfo {
            // Create a test token account
            let key = [2u8; 32];
            AccountInfo {
                key,
                is_signer: false,
                is_writable: true,
                lamports: 1000000,
                data: Vec::new(),
                owner: *mint,
            }
        }
        
        pub fn create_account<T>(&mut self) -> AccountInfo {
            // Create a test account
            let key = [3u8; 32];
            AccountInfo {
                key,
                is_signer: false,
                is_writable: true,
                lamports: 1000000,
                data: Vec::new(),
                owner: [0u8; 32],
            }
        }

        pub fn create_account_with_lamports(&mut self, lamports: u64) -> AccountInfo {
            let mut account = self.create_account::<()>();
            account.lamports = lamports;
            account
        }

        pub fn process_instruction(
            &mut self,
            program: &impl Program,
            accounts: Vec<program::AccountMeta>,
            instruction_data: &[u8],
        ) -> ProgramResult {
            let ctx = ProgramContext {
                accounts: self.accounts.clone(),
                program_id: [0u8; 32],
            };
            Program::process_instruction(program, &ctx, instruction_data)
        }

        pub fn get_account_data<T: BorshDeserialize>(&self, _pubkey: &Pubkey) -> Result<T, ProgramError> {
            // In a real implementation, this would deserialize account data
            Err(ProgramError::Custom("Not implemented".to_string()))
        }

        pub fn get_token_account(&self, _pubkey: &Pubkey) -> Result<TokenAccount, ProgramError> {
            // In a real implementation, this would get token account data
            Ok(TokenAccount {
                mint: [0u8; 32],
                owner: [0u8; 32],
                amount: 0,
            })
        }

        pub fn set_account_data<T: BorshSerialize>(&mut self, _pubkey: &Pubkey, _data: &T) -> Result<(), ProgramError> {
            // In a real implementation, this would serialize and store account data
            Ok(())
        }
    }
}

// Helper trait implementations
impl Clone for TokenAccount {
    fn clone(&self) -> Self {
        Self {
            mint: self.mint,
            owner: self.owner,
            amount: self.amount,
        }
    }
}

// Add msg! macro for logging
#[macro_export]
macro_rules! msg {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
} 