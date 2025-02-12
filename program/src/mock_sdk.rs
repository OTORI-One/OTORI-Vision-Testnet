use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::From;
use std::io;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use crate::OVTState;

// Re-export common types at the root level
pub use account_info::AccountInfo;
pub use pubkey::Pubkey;
pub use program::{Program, ProgramContext, AccountMeta, Instruction};

// Define ProgramResult at the root level
pub type ProgramResult = Result<(), ProgramError>;

pub mod pubkey {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    impl BorshSerialize for Pubkey {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            writer.write_all(&self.0)
        }
    }

    impl BorshDeserialize for Pubkey {
        fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
            if buf.len() < 32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "insufficient bytes",
                ));
            }
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&buf[..32]);
            *buf = &buf[32..];
            Ok(Self(bytes))
        }

        fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            let mut bytes = [0u8; 32];
            reader.read_exact(&mut bytes)?;
            Ok(Self(bytes))
        }
    }
}

pub mod account_info {
    use super::*;
    use super::pubkey::Pubkey;

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
            if data.is_empty() {
                return Err(ProgramError::InvalidAccountData);
            }
            
            borsh::BorshDeserialize::try_from_slice(&data)
                .map_err(|_| ProgramError::InvalidAccountData)
        }

        pub fn set_data<T: BorshSerialize>(&self, data: &T) -> Result<(), ProgramError> {
            if !self.is_writable {
                return Err(ProgramError::InvalidAccountData);
            }
            let serialized = borsh::to_vec(data)
                .map_err(|_| ProgramError::InvalidAccountData)?;
            
            let mut account_data = self.data.borrow_mut();
            account_data.resize(serialized.len(), 0);
            account_data.copy_from_slice(&serialized);
            Ok(())
        }
    }
}

pub mod program {
    use super::*;
    use super::account_info::AccountInfo;
    use super::pubkey::Pubkey;

    pub trait Program {
        fn process_instruction(ctx: &ProgramContext, data: &[u8]) -> ProgramResult;
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

    #[derive(Clone, Debug)]
    pub struct Instruction {
        pub program_id: Pubkey,
        pub accounts: Vec<AccountMeta>,
        pub data: Vec<u8>,
    }

    impl Instruction {
        pub fn new_with_borsh(
            program_id: Pubkey,
            data: &impl BorshSerialize,
            accounts: Vec<AccountMeta>,
        ) -> Self {
            let data = borsh::to_vec(data).unwrap();
            Self {
                program_id,
                accounts,
                data,
            }
        }
    }

    pub struct ProgramContext {
        pub program_id: Pubkey,
        pub accounts: Vec<AccountInfo>,
    }

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

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct TokenAccount {
    pub mint: pubkey::Pubkey,
    pub owner: pubkey::Pubkey,
    pub amount: u64,
    pub delegate: Option<pubkey::Pubkey>,
    pub state: u8,
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority: Option<pubkey::Pubkey>,
}

pub mod test_utils {
    use super::*;
    use super::account_info::AccountInfo;
    use super::pubkey::Pubkey;
    use super::program::AccountMeta;
    
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
        
        pub fn create_token_account(
            &mut self,
            program_id: Pubkey,
            mint: Pubkey,
            owner: Pubkey,
        ) -> Result<AccountHandle, ProgramError> {
            let key = Pubkey::new_unique();
            let account = AccountInfo::new(key, false, true);
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
            
            account.set_data(&empty_state)?;
            self.accounts.insert(key, account);
            Ok(AccountHandle {
                key,
                is_signer: false,
                is_writable: true,
            })
        }

        pub fn create_account_with_lamports(&mut self, lamports: u64) -> Result<AccountHandle, ProgramError> {
            let key = Pubkey::new_unique();
            let account = AccountInfo::new(key, false, true);
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
        ) -> ProgramResult {
            let account_infos: Vec<AccountInfo> = accounts
                .iter()
                .map(|meta| {
                    match self.accounts.get(&meta.pubkey) {
                        Some(account) => {
                            let mut account = account.clone();
                            account.is_signer = meta.is_signer;
                            account.is_writable = meta.is_writable;
                            account
                        }
                        None => panic!("Account not found: {:?}", meta.pubkey),
                    }
                })
                .collect();

            let program_context = program::ProgramContext {
                program_id,
                accounts: account_infos.clone(),
            };

            let result = crate::OVTProgram::process_instruction(&program_context, &instruction_data);

            for account in account_infos.iter() {
                if account.is_writable {
                    self.accounts.insert(account.key, account.clone());
                }
            }

            result
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

// Add entrypoint! macro
#[macro_export]
macro_rules! entrypoint {
    ($process_instruction:ident) => {
        pub fn process_instruction(
            program_id: &Pubkey,
            accounts: &[AccountInfo],
            instruction_data: &[u8],
        ) -> ProgramResult {
            let context = ProgramContext::new(
                *program_id,
                accounts.to_vec(),
            );
            $process_instruction(&context, instruction_data)
        }
    };
} 