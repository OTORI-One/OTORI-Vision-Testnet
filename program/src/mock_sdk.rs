use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::From;
use std::io;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use bitcoin::hashes::{sha256, Hash};
use std::sync::atomic::{AtomicU64, Ordering};
use borsh::io::{Error as BorshError, Write as BorshWrite, Read as BorshRead, ErrorKind};
use bitcoin::{Transaction, Script, ScriptBuf};

// Re-export common types at the root level
pub use account_info::AccountInfo;
pub use pubkey::Pubkey;
pub use program::{Program, ProgramContext, AccountMeta};

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
            let input = format!("test_key_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
            let hash = sha256::Hash::hash(input.as_bytes());
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&hash[..]);
            Self(bytes)
        }
    }

    impl BorshSerialize for Pubkey {
        fn serialize<W: BorshWrite>(&self, writer: &mut W) -> Result<(), BorshError> {
            writer.write_all(&self.0).map_err(|_| BorshError::new(ErrorKind::InvalidData, "Failed to write pubkey"))
        }
    }

    impl BorshDeserialize for Pubkey {
        fn deserialize(buf: &mut &[u8]) -> Result<Self, BorshError> {
            if buf.len() < 32 {
                return Err(BorshError::new(ErrorKind::InvalidData, "Insufficient bytes for pubkey"));
            }
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&buf[..32]);
            *buf = &buf[32..];
            Ok(Self(bytes))
        }

        fn deserialize_reader<R: BorshRead>(reader: &mut R) -> Result<Self, BorshError> {
            let mut bytes = [0u8; 32];
            reader.read_exact(&mut bytes).map_err(|_| BorshError::new(ErrorKind::InvalidData, "Failed to read pubkey"))?;
            Ok(Self(bytes))
        }
    }
}

pub mod account_info {
    use super::*;
    use super::pubkey::Pubkey;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UtxoMeta {
        txid: [u8; 32],
        vout: u32,
    }

    impl UtxoMeta {
        pub fn new(txid: [u8; 32], vout: u32) -> Self {
            Self { txid, vout }
        }

        pub fn from_slice(data: &[u8]) -> Self {
            let mut txid = [0u8; 32];
            txid.copy_from_slice(&data[0..32]);
            let vout = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
            Self { txid, vout }
        }

        pub fn txid(&self) -> &[u8; 32] {
            &self.txid
        }

        pub fn vout(&self) -> u32 {
            self.vout
        }

        pub fn validate(&self, min_confirmations: u32) -> Result<(), ProgramError> {
            // Mock validation that would check Bitcoin network in production
            if self.txid == [0; 32] {
                return Err(ProgramError::Custom("Invalid UTXO: zero txid".to_string()));
            }
            
            // In production this would verify against Bitcoin node
            let mock_confirmations = 6;
            if mock_confirmations < min_confirmations {
                return Err(ProgramError::Custom("Insufficient confirmations".to_string()));
            }

            Ok(())
        }

        pub fn get_status(&self) -> UtxoStatus {
            // Mock implementation - in production would check Bitcoin network
            if self.txid == [0; 32] {
                UtxoStatus::Invalid
            } else {
                UtxoStatus::Active
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum UtxoStatus {
        Pending,    // Waiting for confirmations
        Active,     // Confirmed and spendable  
        Spent,      // UTXO has been consumed
        Invalid,    // UTXO was invalidated (e.g., by reorg)
    }

    impl BorshSerialize for UtxoMeta {
        fn serialize<W: BorshWrite>(&self, writer: &mut W) -> Result<(), BorshError> {
            writer.write_all(&self.txid).map_err(|_| BorshError::new(ErrorKind::InvalidData, "Failed to write txid"))?;
            writer.write_all(&self.vout.to_le_bytes()).map_err(|_| BorshError::new(ErrorKind::InvalidData, "Failed to write vout"))?;
            Ok(())
        }
    }

    impl BorshDeserialize for UtxoMeta {
        fn deserialize_reader<R: BorshRead>(reader: &mut R) -> Result<Self, BorshError> {
            let mut txid = [0u8; 32];
            reader.read_exact(&mut txid).map_err(|_| BorshError::new(ErrorKind::InvalidData, "Failed to read txid"))?;
            
            let mut vout_bytes = [0u8; 4];
            reader.read_exact(&mut vout_bytes).map_err(|_| BorshError::new(ErrorKind::InvalidData, "Failed to read vout"))?;
            let vout = u32::from_le_bytes(vout_bytes);
            
            Ok(Self { txid, vout })
        }
    }

    #[derive(Debug)]
    pub struct AccountInfo {
        pub key: Pubkey,
        pub is_signer: bool,
        pub is_writable: bool,
        pub lamports: RefCell<u64>,
        pub data: RefCell<Vec<u8>>,
        pub owner: RefCell<Pubkey>,
        pub utxo: UtxoMeta,  // Add UTXO field
    }

    impl Clone for AccountInfo {
        fn clone(&self) -> Self {
            Self {
                key: self.key,
                is_signer: self.is_signer,
                is_writable: self.is_writable,
                lamports: RefCell::clone(&self.lamports),
                data: RefCell::clone(&self.data),
                owner: RefCell::clone(&self.owner),
                utxo: self.utxo,
            }
        }
    }

    impl AccountInfo {
        pub fn new(key: Pubkey, is_signer: bool, is_writable: bool) -> Self {
            Self {
                key,
                is_signer,
                is_writable,
                lamports: RefCell::new(0),
                data: RefCell::new(Vec::new()),
                owner: RefCell::new(Pubkey::new()),
                utxo: UtxoMeta::from_slice(&[0; 36]),  // Initialize with empty UTXO
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
            *account_data = serialized;  // Replace entire Vec instead of using copy_from_slice
            Ok(())
        }

        pub fn validate_utxo(&self, min_confirmations: u32) -> Result<(), ProgramError> {
            self.utxo.validate(min_confirmations)
        }

        pub fn get_utxo_status(&self) -> UtxoStatus {
            self.utxo.get_status()
        }
    }
}

pub mod program {
    use super::*;
    use super::account_info::AccountInfo;
    use super::pubkey::Pubkey;
    use bitcoin::{Script, ScriptBuf};

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

    #[derive(Clone)]
    pub struct ProgramContext {
        pub program_id: Pubkey,
        pub accounts: Vec<AccountInfo>,
        pub test_client: Option<super::test_utils::TestClient>,
    }

    impl ProgramContext {
        pub fn new(program_id: Pubkey, accounts: Vec<AccountInfo>) -> Self {
            Self {
                program_id,
                accounts,
                test_client: None,
            }
        }

        pub fn with_test_client(
            program_id: Pubkey,
            accounts: Vec<AccountInfo>,
            test_client: super::test_utils::TestClient,
        ) -> Self {
            Self {
                program_id,
                accounts,
                test_client: Some(test_client),
            }
        }

        pub fn get(&self, index: usize) -> Result<&AccountInfo, ProgramError> {
            self.accounts.get(index).ok_or(ProgramError::AccountNotFound)
        }

        pub fn is_admin(&self, pubkey: &Pubkey) -> bool {
            self.test_client.as_ref()
                .map(|client| client.is_admin(pubkey))
                .unwrap_or(false)
        }
    }

    pub fn get_account_script_pubkey(pubkey: &Pubkey) -> ScriptBuf {
        // Create a P2PKH script using the pubkey
        let mut script_data = Vec::with_capacity(25);
        script_data.extend_from_slice(&[0x76, 0xa9, 0x14]); // OP_DUP OP_HASH160 PUSH20
        script_data.extend_from_slice(&pubkey.0[..20]); // Use first 20 bytes of pubkey as hash
        script_data.extend_from_slice(&[0x88, 0xac]); // OP_EQUALVERIFY OP_CHECKSIG
        ScriptBuf::from_bytes(script_data)
    }

    pub fn get_bitcoin_block_height() -> u32 {
        // Mock implementation - in real environment this would query the Bitcoin network
        123456
    }

    pub fn set_transaction_to_sign(accounts: &[AccountInfo], tx_to_sign: TransactionToSign) -> ProgramResult {
        // Mock implementation of transaction signing
        // In real environment, this would prepare the transaction for signing
        msg!("Setting transaction to sign: {:?}", tx_to_sign);
        Ok(())
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

impl From<borsh::io::Error> for ProgramError {
    fn from(_err: borsh::io::Error) -> Self {
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
    use super::program::{Program, ProgramContext};
    use bitcoin::Transaction;

    #[derive(Debug, Clone)]
    pub struct AccountHandle {
        pub key: Pubkey,
        pub is_signer: bool,
        pub is_writable: bool,
    }

    #[derive(Debug, Clone)]
    pub struct AdminAction {
        pub action_type: String,
        pub description: String,
        pub signatures: Vec<String>,
        pub signed_by: Vec<Pubkey>,
    }

    #[derive(Clone)]
    pub struct TestClient {
        pub accounts: Arc<Mutex<HashMap<Pubkey, AccountInfo>>>,
        pub admin_accounts: Arc<Mutex<HashMap<Pubkey, bool>>>,
        pub pending_actions: Arc<Mutex<Vec<AdminAction>>>,
        pub required_signatures: usize,
        pub total_admins: usize,
    }

    impl TestClient {
        pub fn new() -> Self {
            Self {
                accounts: Arc::new(Mutex::new(HashMap::new())),
                admin_accounts: Arc::new(Mutex::new(HashMap::new())),
                pending_actions: Arc::new(Mutex::new(Vec::new())),
                required_signatures: 3, // 3 out of 5 required
                total_admins: 5,
            }
        }

        pub fn create_account(&mut self, program_id: Pubkey) -> Result<AccountHandle, ProgramError> {
            let key = Pubkey::new_unique();
            let account = AccountInfo {
                key,
                is_signer: false,
                is_writable: true,
                lamports: RefCell::new(0),
                data: RefCell::new(vec![0; 1024]), // Allocate 1KB of space by default
                owner: RefCell::new(program_id),
                utxo: UtxoMeta::from_slice(&[0; 36]),  // Initialize with empty UTXO
            };
            self.accounts.lock().unwrap().insert(key, account);
            Ok(AccountHandle {
                key,
                is_signer: false,
                is_writable: true,
            })
        }

        pub fn create_admin_account(&mut self, _program_id: Pubkey) -> Result<AccountHandle, ProgramError> {
            if self.admin_accounts.lock().unwrap().len() >= self.total_admins {
                return Err(ProgramError::Custom("Maximum number of admins reached".to_string()));
            }

            let key = Pubkey::new_unique();
            let account = AccountInfo {
                key,
                is_signer: true,
                is_writable: false,
                lamports: RefCell::new(1000000),
                data: RefCell::new(Vec::new()),
                owner: RefCell::new(Pubkey::new()),
                utxo: UtxoMeta::from_slice(&[0; 36]),  // Initialize with empty UTXO
            };
            self.accounts.lock().unwrap().insert(key, account);
            self.admin_accounts.lock().unwrap().insert(key, true);
            Ok(AccountHandle {
                key,
                is_signer: true,
                is_writable: false,
            })
        }

        pub fn is_admin(&self, pubkey: &Pubkey) -> bool {
            self.admin_accounts.lock().unwrap().get(pubkey).copied().unwrap_or(false)
        }

        pub fn sign_action(
            &mut self,
            admin_key: &Pubkey,
            action_type: String,
            description: String,
            signature: String,
        ) -> Result<bool, ProgramError> {
            if !self.is_admin(admin_key) {
                return Err(ProgramError::Custom("Not an admin".to_string()));
            }

            let mut pending_actions = self.pending_actions.lock().unwrap();
            let action = pending_actions
                .iter_mut()
                .find(|a| a.action_type == action_type);

            match action {
                Some(action) => {
                    if action.signed_by.contains(admin_key) {
                        return Err(ProgramError::Custom("Admin already signed".to_string()));
                    }
                    action.signatures.push(signature);
                    action.signed_by.push(*admin_key);
                }
                None => {
                    pending_actions.push(AdminAction {
                        action_type,
                        description,
                        signatures: vec![signature],
                        signed_by: vec![*admin_key],
                    });
                }
            }

            // Check if we have enough signatures
            if let Some(action) = pending_actions.last() {
                Ok(action.signatures.len() >= self.required_signatures)
            } else {
                Ok(false)
            }
        }

        pub fn verify_action(
            &self,
            action_type: &str,
            signatures: &[String],
        ) -> Result<bool, ProgramError> {
            if signatures.len() < self.required_signatures {
                return Ok(false);
            }

            let pending_actions = self.pending_actions.lock().unwrap();
            if let Some(action) = pending_actions.iter().find(|a| a.action_type == action_type) {
                // Verify all provided signatures are valid
                for sig in signatures {
                    if !action.signatures.contains(sig) {
                        return Ok(false);
                    }
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }

        pub fn process_transaction(
            &mut self,
            program_id: Pubkey,
            accounts: Vec<AccountMeta>,
            instruction_data: Vec<u8>,
        ) -> ProgramResult {
            let mut ctx_accounts = Vec::new();
            let mut original_keys = Vec::new();
            
            // First, collect the accounts and their metadata
            let account_map = self.accounts.lock().unwrap();
            for meta in accounts {
                let account = account_map.get(&meta.pubkey)
                    .ok_or(ProgramError::AccountNotFound)?;
                
                // Store original key for writable accounts
                if meta.is_writable {
                    original_keys.push(meta.pubkey);
                }
                
                // Create new AccountInfo with same data
                let account_info = AccountInfo {
                    key: account.key,
                    is_signer: meta.is_signer,
                    is_writable: meta.is_writable,
                    lamports: account.lamports.clone(),  // Share the same RefCell
                    data: account.data.clone(),          // Share the same RefCell
                    owner: account.owner.clone(),        // Share the same RefCell
                    utxo: account.utxo,
                };
                
                ctx_accounts.push(account_info);
            }

            let ctx = ProgramContext::with_test_client(program_id, ctx_accounts, self.clone());
            let result = crate::OVTProgram::process_instruction(&ctx, &instruction_data);

            // If instruction succeeded, update writable accounts in the map
            if result.is_ok() {
                let mut account_map = self.accounts.lock().unwrap();
                for (account_info, original_key) in ctx.accounts.into_iter()
                    .zip(original_keys.iter())
                    .filter(|(acc, _)| acc.is_writable)
                {
                    if let Some(account) = account_map.get_mut(original_key) {
                        // Update the contents of the RefCells instead of replacing them
                        *account.lamports.borrow_mut() = *account_info.lamports.borrow();
                        account.data.borrow_mut().clone_from(&account_info.data.borrow());
                        *account.owner.borrow_mut() = *account_info.owner.borrow();
                        *account.utxo = account_info.utxo;
                    }
                }
            }

            result
        }

        pub fn get_account_data<T: BorshDeserialize>(&self, pubkey: &Pubkey) -> Result<T, ProgramError> {
            self.accounts.lock().unwrap().get(pubkey)
                .ok_or(ProgramError::AccountNotFound)?
                .get_data()
        }

        pub fn create_bitcoin_transaction(&self, inputs: Vec<bitcoin::TxIn>, outputs: Vec<bitcoin::TxOut>) -> Transaction {
            Transaction {
                version: bitcoin::transaction::Version::TWO,
                lock_time: bitcoin::absolute::LockTime::ZERO,
                input: inputs,
                output: outputs,
            }
        }

        pub fn create_utxo(&self, txid: [u8; 32], vout: u32) -> UtxoMeta {
            UtxoMeta::new(txid, vout)
        }

        pub fn set_account_utxo(&mut self, pubkey: &Pubkey, utxo: UtxoMeta) -> Result<(), ProgramError> {
            if let Some(account) = self.accounts.lock().unwrap().get_mut(pubkey) {
                account.utxo = utxo;
                Ok(())
            } else {
                Err(ProgramError::AccountNotFound)
            }
        }

        pub fn get_account_utxo(&self, pubkey: &Pubkey) -> Result<UtxoMeta, ProgramError> {
            if let Some(account) = self.accounts.lock().unwrap().get(pubkey) {
                Ok(account.utxo)
            } else {
                Err(ProgramError::AccountNotFound)
            }
        }

        pub fn simulate_bitcoin_block_height(&self) -> u32 {
            // Mock implementation that could be customized for testing
            123456
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

pub mod helper {
    use super::*;
    use bitcoin::Transaction;
    use crate::account_info::AccountInfo;

    pub fn add_state_transition(tx: &mut Transaction, account: &AccountInfo) -> Result<(), ProgramError> {
        // In mock environment, we'll simulate state transition by creating a dummy output
        let script_pubkey = get_account_script_pubkey(&account.key);
        
        // Create a new output with the account's script_pubkey
        let output = bitcoin::TxOut {
            value: account.lamports.borrow().saturating_add(1), // Simulate value transfer
            script_pubkey: script_pubkey,
        };
        
        tx.output.push(output);
        Ok(())
    }
}

#[derive(Debug)]
pub struct TransactionToSign<'a> {
    pub tx_bytes: &'a [u8],
    pub inputs_to_sign: &'a [InputToSign],
}

#[derive(Debug)]
pub struct InputToSign {
    pub index: usize,
    pub signer: Pubkey,
} 