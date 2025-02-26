use arch_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OVTError {
    #[error("Invalid authority")]
    InvalidAuthority,

    #[error("Invalid SAFE ID")]
    InvalidSAFEId,

    #[error("Arithmetic overflow")]
    ArithmeticOverflow,

    #[error("Invalid token account")]
    InvalidTokenAccount,

    #[error("Invalid metadata account")]
    InvalidMetadataAccount,

    #[error("Invalid mint account")]
    InvalidMintAccount,

    #[error("Invalid treasury account")]
    InvalidTreasuryAccount,

    #[error("Invalid oracle account")]
    InvalidOracleAccount,

    #[error("SAFE not found")]
    SAFENotFound,

    #[error("SAFE already converted")]
    SAFEAlreadyConverted,

    #[error("SAFE not yet unlocked")]
    SAFENotUnlocked,

    #[error("Program error: {0}")]
    ProgramError(#[from] ProgramError),

    #[error("Invalid Bitcoin payment: payment verification failed")]
    InvalidBitcoinPayment,

    #[error("Invalid NAV update")]
    InvalidNAVUpdate,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Invalid treasury key")]
    InvalidTreasuryKey,

    #[error("Invalid supply change")]
    InvalidSupplyChange,

    #[error("Invalid timestamp")]
    InvalidTimestamp,

    #[error("UTXO verification failed")]
    UTXOVerificationFailed,

    #[error("Invalid UTXO")]
    InvalidUTXO,

    #[error("Insufficient confirmations")]
    InsufficientConfirmations,

    #[error("UTXO already spent")]
    UTXOSpent,

    #[error("Invalid transaction")]
    InvalidTransaction,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid admin action")]
    InvalidAdminAction,
}

impl From<OVTError> for ProgramError {
    fn from(e: OVTError) -> Self {
        // Convert the error to a u32 code
        let code = match e {
            OVTError::InvalidAuthority => 1000,
            OVTError::InvalidSAFEId => 1001,
            OVTError::ArithmeticOverflow => 1002,
            OVTError::InvalidTokenAccount => 1003,
            OVTError::InvalidMetadataAccount => 1004,
            OVTError::InvalidMintAccount => 1005,
            OVTError::InvalidTreasuryAccount => 1006,
            OVTError::InvalidOracleAccount => 1007,
            OVTError::SAFENotFound => 1008,
            OVTError::SAFEAlreadyConverted => 1009,
            OVTError::SAFENotUnlocked => 1010,
            OVTError::ProgramError(_) => 1011,
            OVTError::InvalidBitcoinPayment => 1012,
            OVTError::InvalidNAVUpdate => 1013,
            OVTError::InsufficientFunds => 1014,
            OVTError::InvalidTreasuryKey => 1015,
            OVTError::InvalidSupplyChange => 1016,
            OVTError::InvalidTimestamp => 1017,
            OVTError::UTXOVerificationFailed => 1018,
            OVTError::InvalidUTXO => 1019,
            OVTError::InsufficientConfirmations => 1020,
            OVTError::UTXOSpent => 1021,
            OVTError::InvalidTransaction => 1022,
            OVTError::InvalidSignature => 1023,
            OVTError::InvalidAdminAction => 1024,
        };
        ProgramError::Custom(code)
    }
} 