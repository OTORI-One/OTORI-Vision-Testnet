use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub enum OVTInstruction {
    /// Initialize OVT state
    Initialize {
        treasury_pubkey_bytes: [u8; 33],
    },
    /// Calculate and update NAV
    UpdateNAV {
        btc_price_sats: u64,
    },
    /// Execute buyback and burn
    BuybackBurn {
        payment_txid: String,
        payment_amount_sats: u64,
    },
} 