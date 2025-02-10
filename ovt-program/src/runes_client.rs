use bitcoin::{PublicKey, Transaction, TxOut, Script};
use arch_sdk::transaction::Transaction as ArchTransaction;
use serde::{Deserialize, Serialize};
use bitcoin::consensus::encode;
use bitcoin::hashes::Hash;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::sleep;
use governor::{Quota, RateLimiter, DefaultKeyedRateLimiter};
use nonzero_ext::nonzero;

const OTORI_VISION_TOKEN: &str = "OTORI VISION TOKEN";

#[derive(Debug, Serialize, Deserialize)]
pub struct RuneInfo {
    pub symbol: String,
    pub decimals: u8,
    pub supply: u64,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub txid: String,
    pub timestamp: u64,
    pub amount: u64,
    pub confirmations: u32,
    pub fee: u64,
    pub status: TransactionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UTXO {
    pub txid: String,
    pub vout: u32,
    pub amount: u64,
    pub script_pubkey: String,
    pub confirmations: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub fast: u64,    // < 10 minutes
    pub medium: u64,  // < 30 minutes
    pub slow: u64,    // < 1 hour
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub connected_peers: u32,
    pub latest_block: u32,
    pub mempool_size: u32,
    pub network_hashrate: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeHealth {
    pub uptime: u64,
    pub sync_status: f64,
    pub memory_usage: u64,
    pub disk_usage: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    timestamp: Instant,
}

impl<T> CacheEntry<T> {
    fn is_valid(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() < ttl
    }
}

#[derive(Debug)]
pub struct RunesClient {
    network: bitcoin::Network,
    rpc_url: String,
    rpc_auth: Option<(String, String)>,
    cache: Arc<Mutex<HashMap<String, CacheEntry<Vec<u8>>>>>,
    rate_limiter: Arc<DefaultKeyedRateLimiter<String>>,
}

impl RunesClient {
    pub fn new(network: bitcoin::Network, rpc_url: String, rpc_auth: Option<(String, String)>) -> Self {
        Self { 
            network,
            rpc_url,
            rpc_auth,
            cache: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter: Arc::new(RateLimiter::keyed(Quota::per_second(nonzero!(10u32)))),
        }
    }

    async fn make_request<T: serde::de::DeserializeOwned>(
        &self,
        url: String,
        cache_key: Option<String>,
        cache_ttl: Option<Duration>,
    ) -> Result<T, Box<dyn std::error::Error>> {
        // Rate limiting
        if let Err(wait_time) = self.rate_limiter.check_key(&url) {
            sleep(wait_time.wait_time()).await;
        }

        // Check cache if enabled
        if let Some(key) = &cache_key {
            if let Some(ttl) = cache_ttl {
                if let Some(entry) = self.cache.lock().unwrap().get(key) {
                    if entry.is_valid(ttl) {
                        return serde_json::from_slice(&entry.data)
                            .map_err(|e| e.into());
                    }
                }
            }
        }

        // Make request with retry logic
        let mut retries = 3;
        let mut last_error = None;

        while retries > 0 {
            match self.execute_request(&url).await {
                Ok(response) => {
                    // Cache the response if caching is enabled
                    if let Some(key) = cache_key {
                        if cache_ttl.is_some() {
                            self.cache.lock().unwrap().insert(
                                key,
                                CacheEntry {
                                    data: response.clone(),
                                    timestamp: Instant::now(),
                                }
                            );
                        }
                    }
                    return serde_json::from_slice(&response)
                        .map_err(|e| e.into());
                }
                Err(e) => {
                    last_error = Some(e);
                    retries -= 1;
                    if retries > 0 {
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Request failed".into()))
    }

    async fn execute_request(&self, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let mut request = client.get(url);
        
        if let Some((username, password)) = &self.rpc_auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;
        Ok(response.bytes().await?.to_vec())
    }

    pub fn create_rune(
        &self,
        symbol: &str,
        decimals: u8,
        initial_supply: u64,
        supply_limit: Option<u64>,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        // Create Runes protocol envelope
        let rune_data = format!(
            "r1:{}:{}:{}:{}",
            symbol,
            decimals,
            initial_supply,
            supply_limit.unwrap_or(0)
        );

        // Create OP_RETURN output with Runes data
        let script = Script::new_op_return(&rune_data.as_bytes());
        let output = TxOut {
            value: 0,
            script_pubkey: script,
        };

        // Create Bitcoin transaction
        let mut tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![], // To be filled by wallet
            output: vec![output],
        };

        Ok(tx)
    }

    pub fn transfer_rune(
        &self,
        symbol: &str,
        amount: u64,
        to: &PublicKey,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        // Create Runes transfer envelope
        let transfer_data = format!("r1:{}:{}", symbol, amount);

        // Create outputs
        let rune_script = Script::new_op_return(&transfer_data.as_bytes());
        let recipient_script = Script::new_p2pkh(&to.pubkey_hash());

        let outputs = vec![
            TxOut {
                value: 0,
                script_pubkey: rune_script,
            },
            TxOut {
                value: 546, // Minimum dust amount
                script_pubkey: recipient_script,
            },
        ];

        // Create Bitcoin transaction
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![], // To be filled by wallet
            output: outputs,
        };

        Ok(tx)
    }

    pub async fn get_rune_info(&self, symbol: &str) -> Result<RuneInfo, Box<dyn std::error::Error>> {
        let url = format!("{}/rune/{}", self.rpc_url, symbol);
        
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        
        if let Some((username, password)) = &self.rpc_auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;
        let rune_info: RuneInfo = response.json().await?;
        
        Ok(rune_info)
    }

    pub async fn get_balance(&self, symbol: &str, pubkey: &PublicKey) -> Result<u64, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/rune/balance/{}/{}",
            self.rpc_url,
            symbol,
            pubkey.to_string()
        );
        
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        
        if let Some((username, password)) = &self.rpc_auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;
        let balance: u64 = response.json().await?;
        
        Ok(balance)
    }

    // Helper function to verify Runes transaction
    pub fn verify_rune_transfer(&self, tx: &Transaction, symbol: &str) -> Result<bool, Box<dyn std::error::Error>> {
        for output in &tx.output {
            if output.script_pubkey.is_op_return() {
                let data = output.script_pubkey.as_bytes();
                if let Some(rune_data) = String::from_utf8_lossy(data).strip_prefix("r1:") {
                    let parts: Vec<&str> = rune_data.split(':').collect();
                    if parts.get(0) == Some(&symbol) {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub async fn mint_tokens(
        &self,
        amount: u64,
        admin_signatures: Vec<String>,
        admin_pubkeys: Vec<PublicKey>,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        // Verify 3-of-5 multisig requirement
        if admin_signatures.len() < 3 || admin_pubkeys.len() != 5 {
            return Err("Invalid multisig configuration - requires 3 of 5 signatures".into());
        }

        // Create Runes mint envelope with fixed symbol
        let mint_data = format!("r1:{}:mint:{}", OTORI_VISION_TOKEN, amount);
        
        // Create outputs for multisig verification
        let mut outputs = Vec::new();
        
        // Add OP_RETURN output with mint data
        let script = Script::new_op_return(&mint_data.as_bytes());
        outputs.push(TxOut {
            value: 0,
            script_pubkey: script,
        });

        // Add multisig verification output
        let multisig_script = Script::new_multisig(3, &admin_pubkeys);
        outputs.push(TxOut {
            value: 546, // Minimum dust amount
            script_pubkey: multisig_script,
        });

        // Create Bitcoin transaction
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![], // To be filled by wallet
            output: outputs,
        };

        Ok(tx)
    }

    pub fn verify_mint_authorization(
        &self,
        signatures: &[String],
        admin_pubkeys: &[PublicKey],
        mint_amount: u64,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Verify we have exactly 5 admin pubkeys
        if admin_pubkeys.len() != 5 {
            return Err("Invalid number of admin pubkeys".into());
        }

        // Verify we have at least 3 signatures
        if signatures.len() < 3 {
            return Err("Insufficient signatures".into());
        }

        // TODO: Implement actual signature verification
        // This would verify each signature against the mint_amount
        // and ensure they're from valid admin pubkeys

        Ok(true)
    }

    pub async fn get_transaction_history(&self, address: &str) -> Result<Vec<TransactionInfo>, Box<dyn std::error::Error>> {
        let url = format!("{}/transactions/{}", self.rpc_url, address);
        self.make_request(
            url,
            Some(format!("tx_history_{}", address)),
            Some(Duration::from_secs(60)),
        ).await
    }

    pub async fn get_pending_transactions(&self) -> Result<Vec<TransactionInfo>, Box<dyn std::error::Error>> {
        let url = format!("{}/mempool/transactions", self.rpc_url);
        self.make_request(
            url,
            Some("pending_tx".to_string()),
            Some(Duration::from_secs(10)),
        ).await
    }

    pub async fn estimate_fees(&self) -> Result<FeeEstimate, Box<dyn std::error::Error>> {
        let url = format!("{}/fees/estimate", self.rpc_url);
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        
        if let Some((username, password)) = &self.rpc_auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;
        let fees: FeeEstimate = response.json().await?;
        
        Ok(fees)
    }

    pub async fn get_utxo_set(&self, address: &str) -> Result<Vec<UTXO>, Box<dyn std::error::Error>> {
        let url = format!("{}/utxos/{}", self.rpc_url, address);
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        
        if let Some((username, password)) = &self.rpc_auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;
        let utxos: Vec<UTXO> = response.json().await?;
        
        Ok(utxos)
    }

    pub async fn get_network_status(&self) -> Result<NetworkStatus, Box<dyn std::error::Error>> {
        let url = format!("{}/network/status", self.rpc_url);
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        
        if let Some((username, password)) = &self.rpc_auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;
        let status: NetworkStatus = response.json().await?;
        
        Ok(status)
    }

    pub async fn get_node_health(&self) -> Result<NodeHealth, Box<dyn std::error::Error>> {
        let url = format!("{}/node/health", self.rpc_url);
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        
        if let Some((username, password)) = &self.rpc_auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;
        let health: NodeHealth = response.json().await?;
        
        Ok(health)
    }

    pub async fn subscribe_to_events(&self) -> Result<impl Stream<Item = NetworkEvent>, Box<dyn std::error::Error>> {
        let url = format!("{}/events/subscribe", self.rpc_url);
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        
        if let Some((username, password)) = &self.rpc_auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;
        let stream = response
            .bytes_stream()
            .map(|chunk| {
                chunk
                    .map_err(|e| e.into())
                    .and_then(|bytes| serde_json::from_slice::<NetworkEvent>(&bytes).map_err(|e| e.into()))
            })
            .filter_map(|result| async move { result.ok() });

        Ok(stream)
    }

    pub async fn verify_admin_multisig(
        &self,
        signatures: &[String],
        message: &[u8],
        admin_pubkeys: &[PublicKey],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if signatures.len() < 3 || admin_pubkeys.len() != 5 {
            return Err("Invalid multisig configuration - requires 3 of 5 signatures".into());
        }

        // TODO: Implement actual signature verification
        // This would verify each signature against the message
        // and ensure they're from valid admin pubkeys

        Ok(true)
    }

    pub async fn add_post_tge_position(
        &self,
        position: PortfolioPosition,
        admin_signatures: &[String],
        admin_pubkeys: &[PublicKey],
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Verify admin authorization
        self.verify_admin_multisig(admin_signatures, &serde_json::to_vec(&position)?, admin_pubkeys).await?;

        // Verify transaction ID exists and is valid
        if position.transaction_id.is_none() {
            return Err("Transaction ID is required for post-TGE positions".into());
        }

        let url = format!("{}/admin/positions/post-tge", self.rpc_url);
        self.make_request(
            url,
            None,
            None,
        ).await
    }

    pub async fn add_pre_tge_position(
        &self,
        position: PortfolioPosition,
        admin_signatures: &[String],
        admin_pubkeys: &[PublicKey],
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Verify admin authorization
        self.verify_admin_multisig(admin_signatures, &serde_json::to_vec(&position)?, admin_pubkeys).await?;

        // Verify SAFE inscription ID exists
        if position.safe_inscription_id.is_none() {
            return Err("SAFE inscription ID is required for pre-TGE positions".into());
        }

        let url = format!("{}/admin/positions/pre-tge", self.rpc_url);
        self.make_request(
            url,
            None,
            None,
        ).await
    }

    pub async fn exit_position(
        &self,
        position_id: &str,
        exit_data: PortfolioPosition,
        admin_signatures: &[String],
        admin_pubkeys: &[PublicKey],
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Verify admin authorization
        self.verify_admin_multisig(admin_signatures, &serde_json::to_vec(&exit_data)?, admin_pubkeys).await?;

        // Verify transaction ID exists for exit
        if exit_data.transaction_id.is_none() {
            return Err("Transaction ID is required for position exit".into());
        }

        let url = format!("{}/admin/positions/{}/exit", self.rpc_url, position_id);
        self.make_request(
            url,
            None,
            None,
        ).await
    }

    pub async fn get_position_history(
        &self,
        position_id: &str,
    ) -> Result<Vec<PortfolioPosition>, Box<dyn std::error::Error>> {
        let url = format!("{}/admin/positions/{}/history", self.rpc_url, position_id);
        self.make_request(
            url,
            Some(format!("position_history_{}", position_id)),
            Some(Duration::from_secs(300)),
        ).await
    }

    pub async fn get_portfolio_metrics(
        &self,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let url = format!("{}/admin/portfolio/metrics", self.rpc_url);
        self.make_request(
            url,
            Some("portfolio_metrics".to_string()),
            Some(Duration::from_secs(300)),
        ).await
    }

    pub async fn export_portfolio_data(
        &self,
        start_date: u64,
        end_date: u64,
        format: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/admin/portfolio/export?start={}&end={}&format={}",
            self.rpc_url, start_date, end_date, format
        );
        self.make_request(
            url,
            None,
            None,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::Secp256k1;

    #[tokio::test]
    async fn test_runes_flow() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );
        
        let secp = Secp256k1::new();
        let (_, pubkey) = secp.generate_keypair(&mut rand::thread_rng());

        // Create OVT Rune
        let tx = client.create_rune(
            "OVT",
            8,
            1_000_000,
            Some(21_000_000),
        ).unwrap();
        
        // Verify Rune creation transaction
        assert!(client.verify_rune_transfer(&tx, "OVT").unwrap());

        // Create transfer transaction
        let tx = client.transfer_rune(
            "OVT",
            1000,
            &pubkey,
        ).unwrap();
        
        // Verify transfer transaction
        assert!(client.verify_rune_transfer(&tx, "OVT").unwrap());
    }

    #[tokio::test]
    async fn test_edge_cases() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );
        
        let secp = Secp256k1::new();
        let (_, pubkey) = secp.generate_keypair(&mut rand::thread_rng());

        // Test case 1: Zero amount transfer
        let result = client.transfer_rune("OVT", 0, &pubkey);
        assert!(result.is_err(), "Should fail with zero amount");

        // Test case 2: Invalid symbol
        let result = client.transfer_rune("INVALID", 1000, &pubkey);
        assert!(result.is_ok()); // Should create tx but fail later in verification

        // Test case 3: Oversized transfer
        let result = client.transfer_rune("OVT", u64::MAX, &pubkey);
        assert!(result.is_ok()); // Should create tx but fail later in verification
    }

    #[tokio::test]
    async fn test_mint_authorization() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );

        let secp = Secp256k1::new();
        
        // Generate 5 admin keypairs
        let mut admin_pubkeys = Vec::new();
        let mut admin_privkeys = Vec::new();
        for _ in 0..5 {
            let (privkey, pubkey) = secp.generate_keypair(&mut rand::thread_rng());
            admin_pubkeys.push(pubkey);
            admin_privkeys.push(privkey);
        }

        // Test case 1: Valid 3-of-5 multisig
        let signatures = vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()];
        let result = client.verify_mint_authorization(&signatures, &admin_pubkeys, 1000);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test case 2: Insufficient signatures
        let signatures = vec!["sig1".to_string(), "sig2".to_string()];
        let result = client.verify_mint_authorization(&signatures, &admin_pubkeys, 1000);
        assert!(result.is_err());

        // Test case 3: Too many admin pubkeys
        let mut extra_pubkeys = admin_pubkeys.clone();
        extra_pubkeys.push(secp.generate_keypair(&mut rand::thread_rng()).1);
        let result = client.verify_mint_authorization(&signatures, &extra_pubkeys, 1000);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transaction_validation() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );

        let secp = Secp256k1::new();
        let (_, pubkey) = secp.generate_keypair(&mut rand::thread_rng());

        // Create a valid transaction
        let tx = client.create_rune(
            "OVT",
            8,
            1_000_000,
            Some(21_000_000),
        ).unwrap();

        // Test case 1: Valid transaction verification
        assert!(client.verify_rune_transfer(&tx, "OVT").unwrap());

        // Test case 2: Wrong symbol verification
        assert!(!client.verify_rune_transfer(&tx, "WRONG").unwrap());

        // Test case 3: Invalid OP_RETURN data
        let mut invalid_tx = tx.clone();
        invalid_tx.output[0].script_pubkey = Script::new_op_return(b"invalid_data");
        assert!(!client.verify_rune_transfer(&invalid_tx, "OVT").unwrap());

        // Test case 4: Missing OP_RETURN output
        let mut no_opreturn_tx = tx.clone();
        no_opreturn_tx.output.clear();
        assert!(!client.verify_rune_transfer(&no_opreturn_tx, "OVT").unwrap());
    }

    #[tokio::test]
    async fn test_transaction_history() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );

        let history = client.get_transaction_history("test_address").await;
        assert!(history.is_ok());
    }

    #[tokio::test]
    async fn test_fee_estimation() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );

        let fees = client.estimate_fees().await;
        assert!(fees.is_ok());
        let fee_estimate = fees.unwrap();
        assert!(fee_estimate.fast >= fee_estimate.medium);
        assert!(fee_estimate.medium >= fee_estimate.slow);
    }

    #[tokio::test]
    async fn test_network_status() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );

        let status = client.get_network_status().await;
        assert!(status.is_ok());
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );

        let events = client.subscribe_to_events().await;
        assert!(events.is_ok());
    }

    #[tokio::test]
    async fn test_admin_position_management() {
        let client = RunesClient::new(
            bitcoin::Network::Testnet,
            "http://localhost:8332".to_string(),
            None,
        );

        let secp = Secp256k1::new();
        
        // Generate admin keys
        let mut admin_pubkeys = Vec::new();
        for _ in 0..5 {
            let (_, pubkey) = secp.generate_keypair(&mut rand::thread_rng());
            admin_pubkeys.push(pubkey);
        }

        // Test post-TGE position
        let post_tge = PortfolioPosition {
            name: "Test Project".to_string(),
            amount: 1000000,
            price_per_token: 100,
            currency_spent: 100000000,
            transaction_id: Some("txid123".to_string()),
            safe_inscription_id: None,
            entry_timestamp: 1677649200,
            position_type: PositionType::PostTGE,
            status: PositionStatus::Active,
        };

        let signatures = vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()];
        let result = client.add_post_tge_position(post_tge, &signatures, &admin_pubkeys).await;
        assert!(result.is_ok());

        // Test pre-TGE position
        let pre_tge = PortfolioPosition {
            name: "Early Project".to_string(),
            amount: 500000,
            price_per_token: 50,
            currency_spent: 25000000,
            transaction_id: None,
            safe_inscription_id: Some("safe123".to_string()),
            entry_timestamp: 1677649200,
            position_type: PositionType::PreTGE,
            status: PositionStatus::Active,
        };

        let result = client.add_pre_tge_position(pre_tge, &signatures, &admin_pubkeys).await;
        assert!(result.is_ok());

        // Test position exit
        let exit_data = PortfolioPosition {
            name: "Test Project".to_string(),
            amount: 1000000,
            price_per_token: 150,
            currency_spent: 150000000,
            transaction_id: Some("txid456".to_string()),
            safe_inscription_id: None,
            entry_timestamp: 1677735600,
            position_type: PositionType::PostTGE,
            status: PositionStatus::Exited,
        };

        let result = client.exit_position("position123", exit_data, &signatures, &admin_pubkeys).await;
        assert!(result.is_ok());

        // Test position history
        let history = client.get_position_history("position123").await;
        assert!(history.is_ok());
        assert!(!history.unwrap().is_empty());

        // Test portfolio metrics
        let metrics = client.get_portfolio_metrics().await;
        assert!(metrics.is_ok());

        // Test data export
        let export = client.export_portfolio_data(1677649200, 1677735600, "csv").await;
        assert!(export.is_ok());
    }
} 