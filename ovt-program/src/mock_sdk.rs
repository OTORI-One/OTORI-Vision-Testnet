use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;
use std::convert::From;
use std::io;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use crate::OVTState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct Pubkey(pub [u8; 32]);

impl Pubkey {
    pub fn new() -> Self {
        Self([0; 32])
    }

    pub fn new_unique() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(bytes)
    }
}

// Program context and result types
pub struct ProgramContext {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountInfo>,
}

#[derive(Debug)]
pub enum ProgramError {
    InvalidArgument,
    InvalidInstructionData,
    InvalidAccountData,
    AccountNotFound,
    InsufficientFunds,
    MissingRequiredSignature,
    Arithmetic,
    Overflow,
    Custom(String),
}

impl std::error::Error for ProgramError {}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgramError::InvalidArgument => write!(f, "Invalid argument"),
            ProgramError::InvalidInstructionData => write!(f, "Invalid instruction data"),
            ProgramError::InvalidAccountData => write!(f, "Invalid account data"),
            ProgramError::AccountNotFound => write!(f, "Account not found"),
            ProgramError::InsufficientFunds => write!(f, "Insufficient funds"),
            ProgramError::MissingRequiredSignature => write!(f, "Missing required signature"),
            ProgramError::Arithmetic => write!(f, "Arithmetic error"),
            ProgramError::Overflow => write!(f, "Overflow"),
            ProgramError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<io::Error> for ProgramError {
    fn from(_err: io::Error) -> Self {
        ProgramError::InvalidInstructionData
    }
}

pub type ProgramResult = Result<(), ProgramError>;

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub key: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
    pub lamports: RefCell<u64>,
    pub data: RefCell<Vec<u8>>,
    pub owner: Pubkey,
}

impl AccountInfo {
    pub fn new(key: Pubkey, is_signer: bool, is_writable: bool) -> Self {
        Self {
            key,
            is_signer,
            is_writable,
            lamports: RefCell::new(0),
            data: RefCell::new(Vec::new()),
            owner: Pubkey::new(),
        }
    }

    pub fn get_data<T: BorshDeserialize>(&self) -> Result<T, ProgramError> {
        let data = self.data.borrow();
        println!("get_data: key={:?}, data_len={}, is_empty={}", self.key, data.len(), data.is_empty());
        
        if data.is_empty() {
            return Err(ProgramError::InvalidAccountData);
        }
        
        // Try to deserialize regardless of zero bytes
        borsh::BorshDeserialize::try_from_slice(&data)
            .map_err(|e| {
                println!("get_data deserialization error: {:?}", e);
                ProgramError::InvalidAccountData
            })
    }

    pub fn set_data<T: BorshSerialize>(&self, data: &T) -> Result<(), ProgramError> {
        println!("set_data: key={:?}, is_writable={}", self.key, self.is_writable);
        if !self.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }
        let mut serialized = Vec::new();
        data.serialize(&mut serialized)
            .map_err(|e| {
                println!("set_data serialization error: {:?}", e);
                ProgramError::InvalidAccountData
            })?;
        
        let mut account_data = self.data.borrow_mut();
        account_data.resize(serialized.len(), 0);
        account_data.copy_from_slice(&serialized);
        println!("set_data success: data_len={}", account_data.len());
        Ok(())
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Mint {
    pub authority: Pubkey,
    pub supply: u64,
    pub decimals: u8,
}

impl Mint {
    pub fn new(authority: Pubkey, decimals: u8) -> Self {
        Self {
            authority,
            supply: 0,
            decimals,
        }
    }

    pub fn mint_to(&mut self, amount: u64) -> ProgramResult {
        self.supply = self.supply.checked_add(amount).ok_or(ProgramError::Overflow)?;
        Ok(())
    }

    pub fn burn(&mut self, amount: u64) -> ProgramResult {
        self.supply = self.supply.checked_sub(amount).ok_or(ProgramError::Overflow)?;
        Ok(())
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub delegate: Option<Pubkey>,
    pub state: u8,
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority: Option<Pubkey>,
}

// Trait definitions
pub trait Program {
    fn process_instruction(ctx: &ProgramContext, data: &[u8]) -> ProgramResult;
}

// Implementation helpers
impl ProgramContext {
    pub fn new(program_id: Pubkey, accounts: Vec<AccountInfo>) -> Self {
        Self {
            program_id,
            accounts,
        }
    }

    pub fn get(&self, index: usize) -> Result<&AccountInfo, ProgramError> {
        self.accounts.get(index).ok_or(ProgramError::AccountNotFound)
    }
}

pub mod prelude {
    pub use super::*;
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

    #[derive(Clone, BorshSerialize, BorshDeserialize)]
    pub struct Account<T: Clone> {
        pub data: T,
    }
}

// Test utilities
pub mod test_utils {
    use super::*;
    use super::program::{AccountMeta, Instruction};
    
    #[derive(Debug, Clone)]
    pub struct AccountHandle {
        pub key: Pubkey,
        pub is_signer: bool,
        pub is_writable: bool,
    }
    
    pub struct TestClient {
        pub accounts: HashMap<Pubkey, AccountInfo>,
        next_pubkey: u64,
    }
    
    impl TestClient {
        pub fn new() -> Self {
            Self {
                accounts: HashMap::new(),
                next_pubkey: 0,
            }
        }
        
        pub fn create_mint(
            &mut self,
            program_id: Pubkey,
            decimals: u8,
            authority: Pubkey,
        ) -> Result<AccountHandle, ProgramError> {
            let key = Pubkey::new_unique();
            let mut account = AccountInfo::new(key, false, true);
            account.owner = program_id;
            let mint = Mint::new(authority, decimals);
            account.set_data(&mint)?;
            self.accounts.insert(key, account);
            Ok(AccountHandle {
                key,
                is_signer: false,
                is_writable: true,
            })
        }
        
        pub fn create_token_account(
            &mut self,
            program_id: Pubkey,
            mint: Pubkey,
            owner: Pubkey,
        ) -> Result<AccountHandle, ProgramError> {
            let key = Pubkey::new_unique();
            let mut account = AccountInfo::new(key, false, true);
            account.owner = program_id;
            let token_account = TokenAccount {
                mint,
                owner,
                amount: 0,
                delegate: None,
                state: 1,
                is_native: None,
                delegated_amount: 0,
                close_authority: None,
            };
            account.set_data(&token_account)?;
            self.accounts.insert(key, account);
            Ok(AccountHandle {
                key,
                is_signer: false,
                is_writable: true,
            })
        }
        
        pub fn create_account(&mut self, program_id: Pubkey) -> Result<AccountHandle, ProgramError> {
            let key = Pubkey::new_unique();
            let mut account = AccountInfo::new(key, false, true);
            account.owner = program_id;
            
            // Initialize with empty OVTState
            let empty_state = OVTState {
                nav_sats: 0,
                treasury_pubkey_bytes: [0u8; 33],
                total_supply: 0,
                last_nav_update: 0,
            };
            
            // Serialize the empty state
            let mut serialized = Vec::new();
            empty_state.serialize(&mut serialized)
                .map_err(|_| ProgramError::InvalidAccountData)?;
            
            // Ensure the account data is properly initialized
            account.data = RefCell::new(serialized.clone());
            
            // Verify the data can be deserialized
            let _: OVTState = borsh::BorshDeserialize::try_from_slice(&serialized)
                .map_err(|_| ProgramError::InvalidAccountData)?;
            
            self.accounts.insert(key, account);
            Ok(AccountHandle {
                key,
                is_signer: false,
                is_writable: true,
            })
        }

        pub fn create_account_with_lamports(&mut self, lamports: u64) -> Result<AccountHandle, ProgramError> {
            let key = Pubkey::new_unique();
            let mut account = AccountInfo::new(key, false, true);
            *account.lamports.borrow_mut() = lamports;
            self.accounts.insert(key, account);
            Ok(AccountHandle {
                key,
                is_signer: false,
                is_writable: true,
            })
        }

        pub fn process_transaction(
            &mut self,
            program_id: Pubkey,
            accounts: Vec<AccountMeta>,
            instruction_data: Vec<u8>,
        ) -> Result<(), ProgramError> {
            let program_context = ProgramContext {
                program_id,
                accounts: accounts.iter().map(|meta| {
                    match self.accounts.get(&meta.pubkey) {
                        Some(account) => AccountInfo {
                            key: account.key,
                            is_signer: meta.is_signer,
                            is_writable: meta.is_writable,
                            lamports: account.lamports.clone(),
                            data: account.data.clone(),
                            owner: account.owner,
                        },
                        None => {
                            println!("Account not found: {:?}", meta.pubkey);
                            println!("Available accounts: {:?}", self.accounts.keys().collect::<Vec<_>>());
                            panic!("Account not found");
                        }
                    }
                }).collect(),
            };

            crate::OVTProgram::process_instruction(&program_context, &instruction_data)
        }

        pub fn get_account_data<T: BorshDeserialize>(&self, pubkey: &Pubkey) -> Result<T, ProgramError> {
            self.accounts.get(pubkey)
                .ok_or(ProgramError::AccountNotFound)?
                .get_data()
        }

        pub fn get_token_account(&self, pubkey: &Pubkey) -> Result<TokenAccount, ProgramError> {
            self.get_account_data(pubkey)
        }

        pub fn set_account_data<T: BorshSerialize>(&mut self, pubkey: &Pubkey, data: &T) -> Result<(), ProgramError> {
            let account = self.accounts.get_mut(pubkey)
                .ok_or(ProgramError::AccountNotFound)?;
            account.set_data(data)?;
            Ok(())
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