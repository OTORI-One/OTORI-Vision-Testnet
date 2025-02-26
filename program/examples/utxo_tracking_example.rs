use program::bitcoin::{
    BitcoinRpcClient, BitcoinRpcConfig, 
    UtxoMeta, UtxoStatus,
    UtxoTracker, UtxoTracking
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Bitcoin RPC client
    let config = BitcoinRpcConfig {
        rpc_endpoint: "http://localhost:8332".to_string(),
        rpc_auth: Some("user:password".to_string()),
        electrs_endpoint: "http://localhost:3000".to_string(),
        network: bitcoin::Network::Regtest,
        min_confirmations: 6,
    };
    
    let rpc_client = Arc::new(BitcoinRpcClient::new(config));
    
    // Create UTXO tracker
    let mut tracker = UtxoTracker::new(rpc_client.clone(), 6);
    
    // Example: Add a UTXO to track
    let example_utxo = UtxoMeta {
        txid: "abc123abc123abc123abc123abc123abc123abc123abc123abc123abc123abc1".to_string(),
        vout: 0,
        amount_sats: 100000, // 0.001 BTC
        script_pubkey: "0014d85c2e4ef14448a3f3c7a90063f73d9440f8b56d".to_string(),
        confirmations: 0,
    };
    
    // Add UTXO to tracker with initial Pending status
    tracker.add_utxo(example_utxo, UtxoStatus::Pending).await;
    
    // Update confirmations for all tracked UTXOs
    tracker.update_confirmations().await;
    
    // Check for chain reorganizations
    tracker.handle_chain_reorg().await;
    
    // Get all active UTXOs
    let active_utxos = tracker.get_utxos_by_status(UtxoStatus::Active).await;
    println!("Active UTXOs: {}", active_utxos.len());
    
    // Get total value of active UTXOs
    let total_value = tracker.get_total_value_by_status(UtxoStatus::Active).await;
    println!("Total value of active UTXOs: {} sats", total_value);
    
    // Mark a UTXO as spent
    tracker.mark_utxo_spent("abc123abc123abc123abc123abc123abc123abc123abc123abc123abc123abc1").await;
    
    Ok(())
} 