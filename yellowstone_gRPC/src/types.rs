use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaTransaction {
    pub signature: String,
    pub slot: u64,
    pub is_vote: bool,
    pub index: u64,
    pub success: bool,
    pub fee: Option<u64>,
    pub compute_units_consumed: Option<u64>,
    pub instructions: Vec<TransactionInstruction>,
    pub account_keys: Vec<String>,
    pub log_messages: Vec<String>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInstruction {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaAccount {
    pub pubkey: String,
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: String, // base64 encoded
    pub write_version: u64,
    pub slot: u64,
    pub txn_signature: Option<String>, // base58 encoded if present
    pub timestamp: DateTime<Utc>,
}
