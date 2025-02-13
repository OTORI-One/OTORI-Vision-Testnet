use bitcoin::{Network, PublicKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunesError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Insufficient signatures")]
    InsufficientSignatures,
    #[error("Invalid admin keys")]
    InvalidAdminKeys,
    #[error("Bitcoin RPC error: {0}")]
    BitcoinRPC(String),
}

#[derive(Debug, Clone)]
pub enum PositionType {
    PreTGE,
    PostTGE,
}

#[derive(Debug, Clone)]
pub enum PositionStatus {
    Active,
    Exited,
    Pending,
}

#[derive(Debug, Clone)]
pub struct PortfolioPosition {
    pub name: String,
    pub amount: u64,
    pub price_per_token: u64,
    pub currency_spent: u64,
    pub transaction_id: Option<String>,
    pub safe_inscription_id: Option<String>,
    pub entry_timestamp: u64,
    pub position_type: PositionType,
    pub status: PositionStatus,
}

#[allow(dead_code)]
pub struct RunesClient {
    network: Network,
    rpc_url: String,
    auth: Option<(String, String)>,
}

impl RunesClient {
    pub fn new(network: Network, rpc_url: String, auth: Option<(String, String)>) -> Self {
        Self {
            network,
            rpc_url,
            auth,
        }
    }

    pub async fn mint_tokens(
        &self,
        _amount: u64,  // Prefixed with _ since it's unused in mock
        signatures: Vec<String>,
        admin_pubkeys: Vec<PublicKey>,
    ) -> Result<String, RunesError> {
        // Mock implementation for testing
        if signatures.len() < 3 {
            return Err(RunesError::InsufficientSignatures);
        }
        if admin_pubkeys.len() != 5 {
            return Err(RunesError::InvalidAdminKeys);
        }
        Ok("mock_txid".to_string())
    }

    pub async fn add_post_tge_position(
        &self,
        _position: PortfolioPosition,
        signatures: &[String],
        admin_pubkeys: &[PublicKey],
    ) -> Result<String, RunesError> {
        // Mock implementation for testing
        if signatures.len() < 3 {
            return Err(RunesError::InsufficientSignatures);
        }
        if admin_pubkeys.len() != 5 {
            return Err(RunesError::InvalidAdminKeys);
        }
        Ok("mock_position_id".to_string())
    }

    pub async fn verify_admin_multisig(
        &self,
        signatures: &[String],
        _message: &[u8],
        admin_pubkeys: &[PublicKey],
    ) -> Result<bool, RunesError> {
        // Mock implementation for testing
        if signatures.len() < 3 {
            return Err(RunesError::InsufficientSignatures);
        }
        if admin_pubkeys.len() != 5 {
            return Err(RunesError::InvalidAdminKeys);
        }
        Ok(true)
    }
} 