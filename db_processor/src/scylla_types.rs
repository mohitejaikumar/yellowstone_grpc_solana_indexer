use chrono::DateTime;

use scylla::macros::FromRow;
use serde::{Deserialize, Serialize};
use yellowstone_gRPC::types::{SolanaAccount, SolanaTransaction};

/// ScyllaDB-compatible transaction struct that matches the schema exactly
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScyllaTransaction {
    pub signature: String,
    pub slot: i64,
    pub is_vote: bool,
    pub tx_index: i64,
    pub success: bool,
    pub fee: i64,
    pub compute_units_consumed: i64,
    pub instructions_json: String,
    pub account_keys_json: String,
    pub log_messages_json: String,
    pub pre_balances_json: String,
    pub post_balances_json: String,
    pub timestamp_ms: i64,
}

/// ScyllaDB-compatible account struct that matches the schema exactly
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScyllaAccount {
    pub pubkey: String,
    pub lamports: i64,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: i64,
    pub data: String,
    pub write_version: i64,
    pub slot: i64,
    pub txn_signature: String,
    pub timestamp_ms: i64,
}

/// Trait for converting domain structs to ScyllaDB-compatible structs
pub trait ToScyllaDb<T> {
    fn to_scylla(&self) -> Result<T, Box<dyn std::error::Error>>;
}

impl ToScyllaDb<ScyllaTransaction> for SolanaTransaction {
    fn to_scylla(&self) -> Result<ScyllaTransaction, Box<dyn std::error::Error>> {
        let instructions_json = serde_json::to_string(&self.instructions)?;
        let account_keys_json = serde_json::to_string(&self.account_keys)?;
        let log_messages_json = serde_json::to_string(&self.log_messages)?;
        let pre_balances_json = serde_json::to_string(&self.pre_balances)?;
        let post_balances_json = serde_json::to_string(&self.post_balances)?;

        Ok(ScyllaTransaction {
            signature: self.signature.clone(),
            slot: self.slot as i64,
            is_vote: self.is_vote,
            tx_index: self.index as i64,
            success: self.success,
            fee: self.fee.unwrap_or(0) as i64,
            compute_units_consumed: self.compute_units_consumed.unwrap_or(0) as i64,
            instructions_json,
            account_keys_json,
            log_messages_json,
            pre_balances_json,
            post_balances_json,
            timestamp_ms: self.timestamp.timestamp_millis(),
        })
    }
}

impl ToScyllaDb<ScyllaAccount> for SolanaAccount {
    fn to_scylla(&self) -> Result<ScyllaAccount, Box<dyn std::error::Error>> {
        Ok(ScyllaAccount {
            pubkey: self.pubkey.clone(),
            lamports: self.lamports as i64,
            owner: self.owner.clone(),
            executable: self.executable,
            rent_epoch: self.rent_epoch as i64,
            data: self.data.clone(),
            write_version: self.write_version as i64,
            slot: self.slot as i64,
            txn_signature: self.txn_signature.clone().unwrap_or_default(),
            timestamp_ms: self.timestamp.timestamp_millis(),
        })
    }
}

/// Helper trait for converting from ScyllaDB structs back to domain structs
pub trait FromScyllaDb<T> {
    fn from_scylla(scylla_data: T) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

impl FromScyllaDb<ScyllaTransaction> for SolanaTransaction {
    fn from_scylla(scylla_tx: ScyllaTransaction) -> Result<Self, Box<dyn std::error::Error>> {
        let instructions = serde_json::from_str(&scylla_tx.instructions_json)?;
        let account_keys = serde_json::from_str(&scylla_tx.account_keys_json)?;
        let log_messages = serde_json::from_str(&scylla_tx.log_messages_json)?;
        let pre_balances = serde_json::from_str(&scylla_tx.pre_balances_json)?;
        let post_balances = serde_json::from_str(&scylla_tx.post_balances_json)?;

        let timestamp =
            DateTime::from_timestamp_millis(scylla_tx.timestamp_ms).ok_or("Invalid timestamp")?;

        Ok(SolanaTransaction {
            signature: scylla_tx.signature,
            slot: scylla_tx.slot as u64,
            is_vote: scylla_tx.is_vote,
            index: scylla_tx.tx_index as u64,
            success: scylla_tx.success,
            fee: if scylla_tx.fee == 0 {
                None
            } else {
                Some(scylla_tx.fee as u64)
            },
            compute_units_consumed: if scylla_tx.compute_units_consumed == 0 {
                None
            } else {
                Some(scylla_tx.compute_units_consumed as u64)
            },
            instructions,
            account_keys,
            log_messages,
            pre_balances,
            post_balances,
            timestamp,
        })
    }
}

impl FromScyllaDb<ScyllaAccount> for SolanaAccount {
    fn from_scylla(scylla_acc: ScyllaAccount) -> Result<Self, Box<dyn std::error::Error>> {
        let timestamp =
            DateTime::from_timestamp_millis(scylla_acc.timestamp_ms).ok_or("Invalid timestamp")?;

        Ok(SolanaAccount {
            pubkey: scylla_acc.pubkey,
            lamports: scylla_acc.lamports as u64,
            owner: scylla_acc.owner,
            executable: scylla_acc.executable,
            rent_epoch: scylla_acc.rent_epoch as u64,
            data: scylla_acc.data,
            write_version: scylla_acc.write_version as u64,
            slot: scylla_acc.slot as u64,
            txn_signature: if scylla_acc.txn_signature.is_empty() {
                None
            } else {
                Some(scylla_acc.txn_signature)
            },
            timestamp,
        })
    }
}
