use crate::error::OVTError;
use arch_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    utxo::UtxoMeta,
};

pub struct BitcoinPayment {
    pub txid: String,
    pub amount_sats: u64,
    pub utxo: UtxoMeta,
}

pub fn verify_bitcoin_payment(
    utxo_info: &AccountInfo,
    expected_amount: u64,
    treasury_pubkey: &[u8; 33],
) -> Result<BitcoinPayment, ProgramError> {
    let utxo = utxo_info.utxo;
    
    // Verify the UTXO belongs to treasury
    if utxo.script_pubkey != treasury_pubkey {
        return Err(OVTError::InvalidBitcoinPayment.into());
    }

    // Verify the amount
    if utxo.value < expected_amount {
        return Err(OVTError::InsufficientFunds.into());
    }

    Ok(BitcoinPayment {
        txid: utxo.outpoint.txid.to_string(),
        amount_sats: utxo.value,
        utxo: utxo.clone(),
    })
}

pub fn verify_utxo_ownership(
    utxo_info: &AccountInfo,
    owner: &Pubkey,
) -> Result<(), ProgramError> {
    if utxo_info.owner != owner {
        return Err(OVTError::UTXOVerificationFailed.into());
    }
    Ok(())
}