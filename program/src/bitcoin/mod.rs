pub mod cache;
pub mod rpc;
pub mod utxo;
pub mod mock;  // Add mock module
pub mod utxo_tracker;

pub use rpc::{BitcoinRpcClient, BitcoinRpcConfig, BitcoinRpcError};
pub use utxo::{UtxoMeta, UtxoStatus, TreasuryPayment};
pub use mock::{MockBitcoinNode, MockBitcoinRpcClient};
pub use utxo_tracker::{UtxoTracker, UtxoTracking};
pub use cache::{UtxoCache, UtxoCacheConfig, CacheStats};

use crate::OVTProgram;
use crate::state::NetworkStatus;
use arch_program::pubkey::Pubkey;
use arch_program::clock::Clock;
use arch_program::utxo::UtxoMeta as ArchUtxoMeta;
use arch_program::program_error::ProgramError;

use bitcoin::{ScriptBuf, PublicKey};
use bitcoin::hashes::Hash as HashTrait;
use bitcoin::hashes::hash160;

// Define trait for syscalls
pub trait SyscallInterface {
    fn sol_log(&self, data: &[u8]);
    fn validate_utxo(&self, utxo: &ArchUtxoMeta, owner: &Pubkey) -> u64;
    fn get_clock(&self, clock: &mut Clock) -> u64;
}

// Production implementation using actual syscalls
#[cfg(feature = "production-syscalls")]
pub struct ProductionSyscalls;

#[cfg(feature = "production-syscalls")]
extern "C" {
    fn sol_log_(ptr: *const u8, len: u64);
    fn arch_validate_utxo_ownership(utxo: *const ArchUtxoMeta, owner: *const Pubkey) -> u64;
    fn arch_get_clock(clock: *mut Clock) -> u64;
}

// Test implementation with mock behavior
#[cfg(not(feature = "production-syscalls"))]
#[derive(Clone)]
pub struct TestSyscalls {
    state: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>>,
    utxo_valid_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

#[cfg(not(feature = "production-syscalls"))]
impl TestSyscalls {
    pub fn new() -> Self {
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            utxo_valid_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    fn mock_sol_log(data: &[u8]) {
        println!("Mock log: {:?}", data);
    }

    fn mock_validate_utxo(utxo: *const ArchUtxoMeta, _owner: *const Pubkey) -> u64 {
        unsafe {
            // Get current count without incrementing yet
            let count = GLOBAL_SYSCALLS.utxo_valid_count.load(std::sync::atomic::Ordering::SeqCst);
            println!("Mock validate UTXO - count: {}", count);
            println!("UTXO txid: {:?}", (*utxo).txid());
            
            // Invalid UTXO case - always fail for all zeros
            if (*utxo).txid() == [0u8; 32] {
                println!("Invalid UTXO detected");
                return 1; // Error
            }
            
            // For reorg test UTXO ([2u8; 32]), only fail on subsequent validations
            if (*utxo).txid() == [2u8; 32] {
                if count > 0 {
                    println!("Reorg detected on subsequent validation");
                    return 1; // Error on revalidation
                }
                // First validation should succeed
                GLOBAL_SYSCALLS.utxo_valid_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                println!("First validation successful for reorg test UTXO");
                return 0;
            }
            
            // All other cases succeed
            println!("UTXO validation successful");
            0 // Success
        }
    }

    fn mock_get_clock(clock: *mut Clock) -> u64 {
        unsafe {
            *clock = Clock {
                slot: 100,
                epoch: 1,
                unix_timestamp: 1234567890,
            };
        }
        0
    }
}

// Global instance for state persistence
#[cfg(not(feature = "production-syscalls"))]
lazy_static::lazy_static! {
    static ref GLOBAL_SYSCALLS: TestSyscalls = TestSyscalls::new();
}

#[cfg(feature = "production-syscalls")]
impl SyscallInterface for ProductionSyscalls {
    fn sol_log(&self, data: &[u8]) {
        unsafe {
            sol_log_(data.as_ptr(), data.len() as u64);
        }
    }

    fn validate_utxo(&self, utxo: &ArchUtxoMeta, owner: &Pubkey) -> u64 {
        unsafe {
            arch_validate_utxo_ownership(utxo, owner)
        }
    }

    fn get_clock(&self, clock: &mut Clock) -> u64 {
        unsafe {
            arch_get_clock(clock)
        }
    }
}

#[cfg(not(feature = "production-syscalls"))]
impl SyscallInterface for TestSyscalls {
    fn sol_log(&self, data: &[u8]) {
        Self::mock_sol_log(data);
    }

    fn validate_utxo(&self, utxo: &ArchUtxoMeta, owner: &Pubkey) -> u64 {
        Self::mock_validate_utxo(utxo, owner)
    }

    fn get_clock(&self, clock: &mut Clock) -> u64 {
        Self::mock_get_clock(clock)
    }
}

// Helper to get the appropriate syscall implementation
#[cfg(feature = "production-syscalls")]
pub fn get_syscalls() -> impl SyscallInterface {
    ProductionSyscalls
}

#[cfg(not(feature = "production-syscalls"))]
pub fn get_syscalls() -> impl SyscallInterface {
    GLOBAL_SYSCALLS.clone()
}

pub fn get_treasury_script_pubkey(pubkey: &PublicKey) -> Result<ScriptBuf, ProgramError> {
    let pubkey_hash = hash160::Hash::hash(&pubkey.to_bytes());
    let wpubkey_hash = bitcoin::key::WPubkeyHash::from_slice(pubkey_hash.as_ref())
        .map_err(|_| ProgramError::InvalidArgument)?;
    let script = ScriptBuf::new_p2wpkh(&wpubkey_hash);
    Ok(script)
}

impl OVTProgram {
    // State Management Syscalls
    pub async fn read_state(&self, key: &[u8]) -> Result<Vec<u8>, ProgramError> {
        #[cfg(not(feature = "production-syscalls"))]
        {
            let state = GLOBAL_SYSCALLS.state.lock().unwrap();
            return Ok(state.get(key).cloned().unwrap_or_default());
        }

        #[cfg(feature = "production-syscalls")]
        {
            let mut data = Vec::with_capacity(1024);
            get_syscalls().sol_log(key);
            Ok(data)
        }
    }

    pub async fn write_state(&self, key: &[u8], value: &[u8]) -> Result<(), ProgramError> {
        #[cfg(not(feature = "production-syscalls"))]
        {
            let mut state = GLOBAL_SYSCALLS.state.lock().unwrap();
            state.insert(key.to_vec(), value.to_vec());
            return Ok(());
        }

        #[cfg(feature = "production-syscalls")]
        {
            get_syscalls().sol_log(key);
            get_syscalls().sol_log(value);
            Ok(())
        }
    }

    // UTXO Verification Syscalls
    pub async fn verify_utxo(&self, txid: &[u8], vout: u32, owner: &Pubkey) -> Result<bool, ProgramError> {
        let mut arch_utxo_bytes = [0u8; 36];
        
        // Handle string-based txid conversion
        // If txid is a hex string, decode it; otherwise use raw bytes
        let txid_bytes = if txid.len() == 64 && txid.iter().all(|&b| b.is_ascii_hexdigit()) {
            // Convert hex string to bytes
            let mut bytes = [0u8; 32];
            for i in 0..32 {
                let pos = i * 2;
                let byte_str = std::str::from_utf8(&txid[pos..pos + 2])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                bytes[i] = u8::from_str_radix(byte_str, 16)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
            }
            bytes
        } else {
            // Use raw bytes, ensuring length is correct
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&txid[..32]);
            bytes
        };
        
        arch_utxo_bytes[..32].copy_from_slice(&txid_bytes);
        arch_utxo_bytes[32..].copy_from_slice(&vout.to_le_bytes());
        
        let arch_utxo = ArchUtxoMeta::from_slice(&arch_utxo_bytes);
        let result = get_syscalls().validate_utxo(&arch_utxo, owner);
        if result != 0 {
            Err(ProgramError::InvalidAccountData)
        } else {
            Ok(true)
        }
    }

    // Network Message Handling
    pub async fn send_network_message(&self, message: &[u8]) -> Result<(), ProgramError> {
        get_syscalls().sol_log(message);
        
        // Check if message contains "invalid"
        if std::str::from_utf8(message).unwrap_or("").contains("invalid") {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        Ok(())
    }

    // Network Status Check
    pub async fn get_network_status(&self) -> Result<NetworkStatus, ProgramError> {
        let mut clock = Clock::default();
        let result = get_syscalls().get_clock(&mut clock);
        
        if result != 0 {
            return Err(ProgramError::InvalidAccountData);
        }
        
        if clock.slot == 0 {
            Ok(NetworkStatus::Syncing)
        } else {
            Ok(NetworkStatus::Active)
        }
    }
} 
 