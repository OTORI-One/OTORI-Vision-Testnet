use thiserror::Error;
use crate::mock_sdk::ProgramError;

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
}

impl From<OVTError> for ProgramError {
    fn from(e: OVTError) -> Self {
        ProgramError::Custom(format!("{:?}", e))
    }
} 