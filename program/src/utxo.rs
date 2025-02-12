use crate::mock_sdk::{
    AccountInfo,
    Pubkey,
    ProgramResult,
};

#[derive(Debug, Clone)]
pub struct UtxoMeta {
    pub txid: String,
    pub vout: u32,
    pub amount: u64,
    pub script_pubkey: String,
    pub confirmations: u32,
}

pub struct BitcoinPayment {
    pub txid: String,
    pub amount_sats: u64,
    pub utxo: UtxoMeta,
}

pub fn verify_bitcoin_payment(
    _utxo_info: &AccountInfo,
    _expected_amount: u64,
    _treasury_pubkey: &[u8; 33],
) -> ProgramResult {
    // Mock implementation for testing
    Ok(())
}

pub fn verify_utxo_ownership(
    _utxo_info: &AccountInfo,
    _program_id: &Pubkey,
) -> ProgramResult {
    // Mock implementation for testing
    Ok(())
}