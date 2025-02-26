use bitcoin::{
    Transaction, 
    TxOut, 
    ScriptBuf,
    absolute::LockTime,
    transaction::Version,
    Amount,
};
use arch_program::program_error::ProgramError;
use crate::bitcoin::utxo::{UtxoMeta, UtxoStatus};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "client")]
use {
    reqwest::{Client, StatusCode},
    bitcoincore_rpc::RpcApi,
};

#[derive(Error, Debug)]
pub enum BitcoinRpcError {
    #[cfg(feature = "client")]
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    #[error("Bitcoin RPC error: {0}")]
    RpcError(String),
    #[error("Invalid response format")]
    InvalidResponse,
    #[error("UTXO not found")]
    UtxoNotFound,
    #[error("RPC connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Transaction not found: {0}")]
    TxNotFound(String),
    #[error("Insufficient confirmations: required {required}, got {actual}")]
    InsufficientConfirmations { required: u32, actual: u32 },
}

impl From<BitcoinRpcError> for ProgramError {
    fn from(e: BitcoinRpcError) -> Self {
        ProgramError::Custom(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct BitcoinRpcConfig {
    pub bitcoin_endpoint: String,
    pub electrs_endpoint: String,
    pub auth: Option<(String, String)>,
    pub network: String,
    pub min_confirmations: u32,
}

impl Default for BitcoinRpcConfig {
    fn default() -> Self {
        Self {
            bitcoin_endpoint: "http://127.0.0.1:18443".to_string(),
            electrs_endpoint: "http://127.0.0.1:3002".to_string(),
            auth: Some(("bitcoin".to_string(), "bitcoinpass".to_string())),
            network: "regtest".to_string(),
            min_confirmations: 1,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ElectrsTransaction {
    txid: String,
    confirmations: u32,
    vout: Vec<ElectrsOutput>,
}

#[derive(Debug, Deserialize)]
struct ElectrsOutput {
    value: u64,
    scriptpubkey: String,
}

pub struct BitcoinRpcClient {
    config: BitcoinRpcConfig,
    http_client: reqwest::Client,
}

impl BitcoinRpcClient {
    pub fn new(config: BitcoinRpcConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn get_transaction(&self, txid: &str) -> Result<Transaction, BitcoinRpcError> {
        let url = format!("{}/tx/{}", self.config.electrs_endpoint, txid);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BitcoinRpcError::ConnectionFailed(e.to_string()))?;
            
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(BitcoinRpcError::TxNotFound(txid.to_string()));
        }
        
        let tx: ElectrsTransaction = response
            .json()
            .await
            .map_err(|e| BitcoinRpcError::InvalidResponse)?;
            
        // Convert Electrs transaction to arch_bitcoin::Transaction
        Ok(Transaction {
            version: Version(2),
            lock_time: LockTime::ZERO,
            input: vec![], // We don't need inputs for our use case
            output: tx.vout.into_iter().map(|out| TxOut {
                value: Amount::from_sat(out.value),
                script_pubkey: ScriptBuf::from_bytes(&hex::decode(&out.scriptpubkey)
                    .map_err(|_| BitcoinRpcError::InvalidResponse)?),
            }).collect(),
        })
    }

    pub async fn get_utxo_status(&self, utxo: &UtxoMeta) -> Result<UtxoStatus, BitcoinRpcError> {
        let url = format!("{}/tx/{}/outspend/{}", self.config.electrs_endpoint, utxo.txid, utxo.vout);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BitcoinRpcError::ConnectionFailed(e.to_string()))?;
            
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(UtxoStatus::Invalid);
        }
        
        #[derive(Deserialize)]
        struct OutspendStatus {
            spent: bool,
            confirmations: Option<u32>,
        }
        
        let status: OutspendStatus = response
            .json()
            .await
            .map_err(|e| BitcoinRpcError::InvalidResponse)?;
            
        if status.spent {
            Ok(UtxoStatus::Spent)
        } else {
            match status.confirmations {
                Some(conf) if conf >= self.config.min_confirmations => Ok(UtxoStatus::Active),
                Some(_) | None => Ok(UtxoStatus::Pending),
            }
        }
    }

    pub async fn get_confirmations(&self, txid: &str) -> Result<u32, BitcoinRpcError> {
        let url = format!("{}/tx/{}/status", self.config.electrs_endpoint, txid);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BitcoinRpcError::ConnectionFailed(e.to_string()))?;
            
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(0);
        }
        
        #[derive(Deserialize)]
        struct TxStatus {
            confirmed: bool,
            block_height: Option<u32>,
        }
        
        let status: TxStatus = response
            .json()
            .await
            .map_err(|e| BitcoinRpcError::InvalidResponse)?;
            
        if !status.confirmed {
            Ok(0)
        } else {
            // Get current block height from Bitcoin Core
            let url = format!("{}/blocks/tip/height", self.config.electrs_endpoint);
            let current_height: u32 = self.http_client
                .get(&url)
                .send()
                .await
                .map_err(|e| BitcoinRpcError::ConnectionFailed(e.to_string()))?
                .json()
                .await
                .map_err(|e| BitcoinRpcError::InvalidResponse)?;
                
            Ok(current_height - status.block_height.unwrap_or(current_height) + 1)
        }
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
        let tx_hex = hex::encode(tx.serialize().as_ref());
        let url = format!("{}/tx", self.config.electrs_endpoint);
        
        let response = self.http_client
            .post(&url)
            .body(tx_hex)
            .send()
            .await
            .map_err(|e| BitcoinRpcError::ConnectionFailed(e.to_string()))?;
            
        if !response.status().is_success() {
            return Err(BitcoinRpcError::RpcError("Failed to broadcast transaction".to_string()));
        }
        
        let txid: String = response
            .text()
            .await
            .map_err(|_| BitcoinRpcError::RpcError("Invalid response format".to_string()))?;
            
        Ok(txid)
    }
} 