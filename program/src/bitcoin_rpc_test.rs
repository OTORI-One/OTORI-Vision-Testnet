use bitcoin::{
    Transaction, 
    TxIn, 
    TxOut, 
    Script,
    absolute::LockTime,
    transaction::Version,
    Amount,
    Sequence,
};
use crate::bitcoin::utxo::{UtxoMeta, UtxoStatus};
use tokio::test;

use crate::bitcoin_rpc::{BitcoinRpcClient, BitcoinRpcConfig, BitcoinRpcError};

// Test configuration using regtest environment
fn get_test_config() -> BitcoinRpcConfig {
    BitcoinRpcConfig {
        bitcoin_endpoint: "http://127.0.0.1:18443".to_string(),
        electrs_endpoint: "http://127.0.0.1:3002".to_string(),
        auth: Some(("bitcoin".to_string(), "bitcoinpass".to_string())),
        network: "regtest".to_string(),
        min_confirmations: 1,
    }
}

async fn setup_test_client() -> BitcoinRpcClient {
    BitcoinRpcClient::new(get_test_config())
}

async fn generate_blocks(count: u32) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let auth = base64::encode("bitcoin:bitcoinpass".as_bytes());
    
    let response = client
        .post("http://127.0.0.1:18443")
        .header("Authorization", format!("Basic {}", auth))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": "test",
            "method": "generatetoaddress",
            "params": [
                count,
                "bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080" // Standard regtest address
            ]
        }))
        .send()
        .await?;
        
    if !response.status().is_success() {
        return Err("Failed to generate blocks".into());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_transaction() {
    let client = setup_test_client().await;
    
    // Create a test transaction
    let tx = Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: Default::default(),
            script_sig: Script::new(),
            sequence: Sequence::MAX,
            witness: vec![],
        }],
        output: vec![TxOut {
            value: Amount::from_sat(100000),
            script_pubkey: Script::new(),
        }],
    };
    
    // Broadcast transaction
    let txid = client.broadcast_transaction(&tx).await.unwrap();
    
    // Test fetching the transaction
    let result = client.get_transaction(&txid).await;
    assert!(result.is_ok());
    
    // Test non-existent transaction
    let result = client.get_transaction("nonexistent_tx").await;
    assert!(matches!(result, Err(BitcoinRpcError::TxNotFound(_))));
}

#[tokio::test]
async fn test_utxo_validation() {
    let client = setup_test_client().await;
    
    // Create and broadcast a test transaction
    let tx = Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: Default::default(),
            script_sig: Script::new(),
            sequence: Sequence::MAX,
            witness: vec![],
        }],
        output: vec![TxOut {
            value: Amount::from_sat(100000),
            script_pubkey: Script::new(),
        }],
    };
    
    let txid = client.broadcast_transaction(&tx).await.unwrap();
    
    let test_utxo = UtxoMeta {
        txid: txid.clone(),
        vout: 0,
        amount: 100000,
        script_pubkey: "test_script".to_string(),
        confirmations: 0,
    };
    
    // Test unconfirmed UTXO
    let status = client.get_utxo_status(&test_utxo).await.unwrap();
    assert_eq!(status, UtxoStatus::Pending);
    
    // Generate a block to confirm the transaction
    generate_blocks(1).await.unwrap();
    
    // Test confirmed UTXO
    let status = client.get_utxo_status(&test_utxo).await.unwrap();
    assert_eq!(status, UtxoStatus::Active);
}

#[tokio::test]
async fn test_confirmations() {
    let client = setup_test_client().await;
    
    // Create and broadcast a test transaction
    let tx = Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: Default::default(),
            script_sig: Script::new(),
            sequence: Sequence::MAX,
            witness: vec![],
        }],
        output: vec![TxOut {
            value: Amount::from_sat(100000),
            script_pubkey: Script::new(),
        }],
    };
    
    let txid = client.broadcast_transaction(&tx).await.unwrap();
    
    // Test unconfirmed transaction
    let confirmations = client.get_confirmations(&txid).await.unwrap();
    assert_eq!(confirmations, 0);
    
    // Generate blocks
    generate_blocks(3).await.unwrap();
    
    // Test confirmed transaction
    let confirmations = client.get_confirmations(&txid).await.unwrap();
    assert!(confirmations >= 3);
}

#[tokio::test]
async fn test_error_handling() {
    let client = setup_test_client().await;
    
    // Test UTXO validation with insufficient confirmations
    let test_utxo = UtxoMeta {
        txid: "unconfirmed_tx".to_string(),
        vout: 0,
        amount: 100000,
        script_pubkey: "test_script".to_string(),
        confirmations: 0,
    };
    
    let result = client.validate_utxo(&test_utxo).await;
    assert!(matches!(result, Err(BitcoinRpcError::InsufficientConfirmations { .. })));
} 