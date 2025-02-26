use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use bitcoin::{
    Transaction, 
    TxIn, 
    TxOut, 
    Script,
    absolute::LockTime,
    transaction::Version,
    Amount,
};
use crate::bitcoin::utxo::{UtxoMeta, UtxoStatus};

use crate::bitcoin_rpc::{BitcoinRpcClient, BitcoinRpcConfig, BitcoinRpcError};

#[derive(Debug, Clone)]
struct MockTransaction {
    confirmations: u32,
    outputs: Vec<TxOut>,
    is_valid: bool,
}

pub struct MockBitcoinNode {
    transactions: Arc<Mutex<HashMap<String, MockTransaction>>>,
    utxo_set: Arc<Mutex<HashMap<(String, u32), bool>>>, // (txid, vout) -> is_spent
}

impl Default for MockBitcoinNode {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBitcoinNode {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(Mutex::new(HashMap::new())),
            utxo_set: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_transaction(&self, txid: &str, confirmations: u32, outputs: Vec<TxOut>, is_valid: bool) {
        let mut txs = self.transactions.lock().unwrap();
        txs.insert(txid.to_string(), MockTransaction {
            confirmations,
            outputs,
            is_valid,
        });

        // Add UTXOs
        let mut utxos = self.utxo_set.lock().unwrap();
        for vout in 0..outputs.len() {
            utxos.insert((txid.to_string(), vout as u32), false); // not spent initially
        }
    }

    pub fn spend_utxo(&self, txid: &str, vout: u32) {
        let mut utxos = self.utxo_set.lock().unwrap();
        utxos.insert((txid.to_string(), vout), true);
    }

    pub fn is_utxo_spent(&self, txid: &str, vout: u32) -> bool {
        let utxos = self.utxo_set.lock().unwrap();
        *utxos.get(&(txid.to_string(), vout)).unwrap_or(&true)
    }

    pub fn get_transaction(&self, txid: &str) -> Option<MockTransaction> {
        let txs = self.transactions.lock().unwrap();
        txs.get(txid).cloned()
    }
}

pub struct MockBitcoinRpcClient {
    node: Arc<MockBitcoinNode>,
    config: BitcoinRpcConfig,
}

impl MockBitcoinRpcClient {
    pub fn new(config: BitcoinRpcConfig, node: Arc<MockBitcoinNode>) -> Self {
        Self { node, config }
    }

    pub async fn get_transaction(&self, txid: &str) -> Result<Transaction, BitcoinRpcError> {
        match self.node.get_transaction(txid) {
            Some(mock_tx) if mock_tx.is_valid => {
                Ok(Transaction {
                    version: Version(2),
                    lock_time: LockTime::ZERO,
                    input: vec![],
                    output: mock_tx.outputs,
                })
            }
            Some(_) => Err(BitcoinRpcError::RpcError("Invalid transaction format".to_string())),
            None => Err(BitcoinRpcError::TxNotFound(txid.to_string())),
        }
    }

    pub async fn get_utxo_status(&self, utxo: &UtxoMeta) -> Result<UtxoStatus, BitcoinRpcError> {
        // Check if transaction exists
        let tx = match self.node.get_transaction(&utxo.txid) {
            Some(tx) => tx,
            None => return Ok(UtxoStatus::Invalid),
        };

        // Check if UTXO is spent
        if self.node.is_utxo_spent(&utxo.txid, utxo.vout) {
            return Ok(UtxoStatus::Spent);
        }

        // Check confirmations
        if tx.confirmations == 0 {
            Ok(UtxoStatus::Pending)
        } else if tx.confirmations >= self.config.min_confirmations {
            Ok(UtxoStatus::Active)
        } else {
            Ok(UtxoStatus::Pending)
        }
    }

    pub async fn get_confirmations(&self, txid: &str) -> Result<u32, BitcoinRpcError> {
        Ok(self.node.get_transaction(txid)
            .map(|tx| tx.confirmations)
            .unwrap_or(0))
    }

    pub async fn validate_utxo(&self, utxo: &UtxoMeta) -> Result<(), BitcoinRpcError> {
        let status = self.get_utxo_status(utxo).await?;
        
        match status {
            UtxoStatus::Active => Ok(()),
            UtxoStatus::Pending => {
                let confirmations = self.get_confirmations(&utxo.txid).await?;
                Err(BitcoinRpcError::InsufficientConfirmations {
                    required: self.config.min_confirmations,
                    actual: confirmations,
                })
            },
            UtxoStatus::Spent => Err(BitcoinRpcError::RpcError("UTXO is spent".to_string())),
            UtxoStatus::Invalid => Err(BitcoinRpcError::RpcError("Invalid UTXO".to_string())),
        }
    }

    pub async fn broadcast_transaction(&self, tx: &Transaction) -> Result<String, BitcoinRpcError> {
        if tx.input.is_empty() || tx.output.is_empty() {
            return Err(BitcoinRpcError::RpcError("Invalid transaction format".to_string()));
        }
        Ok("mock_txid".to_string())
    }
} 