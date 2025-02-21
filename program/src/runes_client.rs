use bitcoin::{Network, PublicKey};
use thiserror::Error;
use std::time::Duration;

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

#[derive(Debug, Clone)]
pub struct RunesConfig {
    pub network: Network,
    pub rpc_url: String,
    pub auth: Option<(String, String)>,
    pub retry_config: RetryConfig,
    pub circuit_breaker_config: CircuitBreakerConfig,
    pub mock_mode: bool,
}

#[allow(dead_code)]
pub struct RunesClient {
    network: Network,
    rpc_url: String,
    auth: Option<(String, String)>,
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    mock_mode: bool,
}

struct RetryConfig {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout: Duration,
    half_open_timeout: Duration,
}

impl RunesClient {
    pub fn new(network: Network, rpc_url: String, auth: Option<(String, String)>) -> Self {
        Self {
            network,
            rpc_url,
            auth,
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_millis(5000),
            },
            circuit_breaker: CircuitBreaker {
                failure_threshold: 3,
                reset_timeout: Duration::from_secs(30),
                half_open_timeout: Duration::from_secs(10),
            },
            mock_mode: false,
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

    pub async fn send_transaction(&self, tx: Transaction) -> Result<String, RunesError> {
        if self.mock_mode {
            return self.mock_send_transaction(tx);
        }
        
        self.circuit_breaker.check()?;
        
        let result = self.with_retry(|| async {
            // Actual network call
            self.rpc_client.send_raw_transaction(&tx.serialize())
        }).await;

        match result {
            Ok(txid) => {
                self.circuit_breaker.record_success();
                Ok(txid)
            }
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e.into())
            }
        }
    }
} 