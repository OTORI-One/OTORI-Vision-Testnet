use thiserror::Error;
use arch_sdk::program::ProgramError;

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
}

impl From<OVTError> for ProgramError {
    fn from(e: OVTError) -> Self {
        ProgramError::Custom(e.to_string())
    }
} 