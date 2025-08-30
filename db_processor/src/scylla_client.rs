use std::error::Error;
use std::sync::Arc;

use crate::scylla_types::{FromScyllaDb, ScyllaAccount, ScyllaTransaction, ToScyllaDb};
use scylla::{Session, SessionBuilder};
use yellowstone_gRPC::types::{SolanaAccount, SolanaTransaction};

pub struct ScyllaWriter {
    session: Arc<Session>,
    keyspace: String,
    accounts_table: String,
    transactions_table: String,
    batch_size: usize,
    account_batch: Vec<ScyllaAccount>,
    transaction_batch: Vec<ScyllaTransaction>,
}

impl ScyllaWriter {
    pub async fn new(
        nodes: Vec<&str>,
        keyspace: &str,
        accounts_table: &str,
        transactions_table: &str,
        batch_size: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let session: Session = SessionBuilder::new().known_nodes(nodes).build().await?;

        let session = Arc::new(session);

        let writer = Self {
            session,
            keyspace: keyspace.to_string(),
            accounts_table: accounts_table.to_string(),
            transactions_table: transactions_table.to_string(),
            batch_size,
            account_batch: Vec::with_capacity(batch_size),
            transaction_batch: Vec::with_capacity(batch_size),
        };

        Ok(writer)
    }

    pub async fn create_keyspace(&self) -> Result<(), Box<dyn Error>> {
        let create_keyspace_query = format!(
            r#"
            CREATE KEYSPACE IF NOT EXISTS {}
            WITH REPLICATION = {{
                'class': 'SimpleStrategy',
                'replication_factor': 1
            }}
            "#,
            self.keyspace
        );

        self.session
            .query_unpaged(create_keyspace_query, &[])
            .await?;

        // Use the keyspace
        let use_keyspace_query = format!("USE {}", self.keyspace);
        self.session.query_unpaged(use_keyspace_query, &[]).await?;

        Ok(())
    }

    pub async fn create_accounts_table(&self) -> Result<(), Box<dyn Error>> {
        let create_table_query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}.{} (
                pubkey text PRIMARY KEY,
                lamports bigint,
                owner text,
                executable boolean,
                rent_epoch bigint,
                data text,
                write_version bigint,
                slot bigint,
                txn_signature text,
                timestamp_ms bigint
            );
            "#,
            self.keyspace, self.accounts_table
        );

        self.session.query_unpaged(create_table_query, &[]).await?;

        Ok(())
    }

    pub async fn create_transactions_table(&self) -> Result<(), Box<dyn Error>> {
        let create_table_query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}.{} (
                signature text PRIMARY KEY,
                slot bigint,
                is_vote boolean,
                tx_index bigint,
                success boolean,
                fee bigint,
                compute_units_consumed bigint,
                instructions_json text,
                account_keys_json text,
                log_messages_json text,
                pre_balances_json text,
                post_balances_json text,
                timestamp_ms bigint
            );
            "#,
            self.keyspace, self.transactions_table
        );

        self.session.query_unpaged(create_table_query, &[]).await?;

        // Create index on slot for efficient slot-based queries
        let create_slot_index = format!(
            "CREATE INDEX IF NOT EXISTS ON {}.{} (slot);",
            self.keyspace, self.transactions_table
        );
        self.session.query_unpaged(create_slot_index, &[]).await?;

        Ok(())
    }

    pub async fn flush_account_batch(&mut self) -> Result<(), Box<dyn Error>> {
        if self.account_batch.is_empty() {
            return Ok(());
        }

        let start_time = std::time::Instant::now();

        let insert_query = format!(
            "INSERT INTO {}.{} (pubkey, lamports, owner, executable, rent_epoch, data, write_version, slot, txn_signature, timestamp_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            self.keyspace, self.accounts_table
        );

        for account in &self.account_batch {
            self.session
                .query_unpaged(
                    insert_query.as_str(),
                    (
                        &account.pubkey,
                        account.lamports,
                        &account.owner,
                        account.executable,
                        account.rent_epoch,
                        &account.data,
                        account.write_version,
                        account.slot,
                        &account.txn_signature,
                        account.timestamp_ms,
                    ),
                )
                .await?;
        }

        let duration = start_time.elapsed();
        println!(
            "Inserted {} accounts in {:?} (avg: {:.2}ms per account)",
            self.account_batch.len(),
            duration,
            duration.as_millis() as f64 / self.account_batch.len() as f64
        );

        self.account_batch.clear();
        Ok(())
    }

    pub async fn flush_transaction_batch(&mut self) -> Result<(), Box<dyn Error>> {
        if self.transaction_batch.is_empty() {
            return Ok(());
        }

        let start_time = std::time::Instant::now();

        let insert_query = format!(
            "INSERT INTO {}.{} (signature, slot, is_vote, tx_index, success, fee, compute_units_consumed, instructions_json, account_keys_json, log_messages_json, pre_balances_json, post_balances_json, timestamp_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            self.keyspace, self.transactions_table
        );

        for transaction in &self.transaction_batch {
            self.session
                .query_unpaged(
                    insert_query.as_str(),
                    (
                        &transaction.signature,
                        transaction.slot,
                        transaction.is_vote,
                        transaction.tx_index,
                        transaction.success,
                        transaction.fee,
                        transaction.compute_units_consumed,
                        &transaction.instructions_json,
                        &transaction.account_keys_json,
                        &transaction.log_messages_json,
                        &transaction.pre_balances_json,
                        &transaction.post_balances_json,
                        transaction.timestamp_ms,
                    ),
                )
                .await?;
        }

        let duration = start_time.elapsed();
        println!(
            "Inserted {} transactions in {:?} (avg: {:.2}ms per transaction)",
            self.transaction_batch.len(),
            duration,
            duration.as_millis() as f64 / self.transaction_batch.len() as f64
        );

        self.transaction_batch.clear();
        Ok(())
    }

    pub async fn add_account(&mut self, account: SolanaAccount) -> Result<(), Box<dyn Error>> {
        let mut scylla_account = account.to_scylla()?;
        scylla_account.data = "".to_string(); // Clear data for storage efficiency
        self.account_batch.push(scylla_account);
        if self.account_batch.len() >= self.batch_size {
            self.flush_account_batch().await?;
        }
        Ok(())
    }

    pub async fn add_transaction(
        &mut self,
        transaction: SolanaTransaction,
    ) -> Result<(), Box<dyn Error>> {
        let scylla_transaction = transaction.to_scylla()?;
        self.transaction_batch.push(scylla_transaction);
        if self.transaction_batch.len() >= self.batch_size {
            self.flush_transaction_batch().await?;
        }
        Ok(())
    }

    pub async fn add_accounts(
        &mut self,
        accounts: Vec<SolanaAccount>,
    ) -> Result<(), Box<dyn Error>> {
        for account in accounts {
            self.add_account(account).await?;
        }
        Ok(())
    }

    pub async fn add_transactions(
        &mut self,
        transactions: Vec<SolanaTransaction>,
    ) -> Result<(), Box<dyn Error>> {
        for transaction in transactions {
            self.add_transaction(transaction).await?;
        }
        Ok(())
    }

    pub async fn flush_all_batches(&mut self) -> Result<(), Box<dyn Error>> {
        self.flush_account_batch().await?;
        self.flush_transaction_batch().await?;
        Ok(())
    }

    // ---------------------------
    // Queries - Accounts
    // ---------------------------

    pub async fn query_accounts_by_slot(
        &self,
        slot: u64,
    ) -> Result<Vec<SolanaAccount>, Box<dyn Error>> {
        let query = format!(
            "SELECT * FROM {}.{} WHERE slot = ? ALLOW FILTERING",
            self.keyspace, self.accounts_table
        );

        let rows = self
            .session
            .query_unpaged(query.as_str(), (slot as i64,))
            .await?;
        let scylla_accounts = rows
            .rows_typed::<ScyllaAccount>()?
            .collect::<Result<Vec<_>, _>>()?;

        let accounts = scylla_accounts
            .into_iter()
            .map(|acc| SolanaAccount::from_scylla(acc))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accounts)
    }

    pub async fn query_account_history(
        &self,
        pubkey: &str,
        limit: u32,
    ) -> Result<Vec<SolanaAccount>, Box<dyn Error>> {
        let query = format!(
            "SELECT * FROM {}.{} WHERE pubkey = ? LIMIT ?",
            self.keyspace, self.accounts_table
        );

        let rows = self
            .session
            .query_unpaged(query.as_str(), (pubkey, limit as i32))
            .await?;
        let scylla_accounts = rows
            .rows_typed::<ScyllaAccount>()?
            .collect::<Result<Vec<_>, _>>()?;

        let accounts = scylla_accounts
            .into_iter()
            .map(|acc| SolanaAccount::from_scylla(acc))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accounts)
    }

    // ---------------------------
    // Queries - Transactions
    // ---------------------------

    pub async fn query_transactions_by_slot(
        &self,
        slot: u64,
    ) -> Result<Vec<SolanaTransaction>, Box<dyn Error>> {
        let query = format!(
            "SELECT * FROM {}.{} WHERE slot = ? ALLOW FILTERING",
            self.keyspace, self.transactions_table
        );

        let rows = self
            .session
            .query_unpaged(query.as_str(), (slot as i64,))
            .await?;
        let scylla_transactions = rows
            .rows_typed::<ScyllaTransaction>()?
            .collect::<Result<Vec<_>, _>>()?;

        let transactions = scylla_transactions
            .into_iter()
            .map(|tx| SolanaTransaction::from_scylla(tx))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    pub async fn query_transaction_by_signature(
        &self,
        signature: &str,
    ) -> Result<Option<SolanaTransaction>, Box<dyn Error>> {
        let query = format!(
            "SELECT * FROM {}.{} WHERE signature = ?",
            self.keyspace, self.transactions_table
        );

        let rows = self
            .session
            .query_unpaged(query.as_str(), (signature,))
            .await?;
        let mut scylla_transactions = rows
            .rows_typed::<ScyllaTransaction>()?
            .collect::<Result<Vec<_>, _>>()?;

        match scylla_transactions.pop() {
            Some(scylla_tx) => Ok(Some(SolanaTransaction::from_scylla(scylla_tx)?)),
            None => Ok(None),
        }
    }

    pub async fn query_transactions_by_account(
        &self,
        account: &str,
        limit: u32,
    ) -> Result<Vec<SolanaTransaction>, Box<dyn Error>> {
        // Since we're storing account_keys as JSON, we need to search within the JSON text
        let query = format!(
            "SELECT * FROM {}.{} LIMIT ? ALLOW FILTERING",
            self.keyspace, self.transactions_table
        );

        let rows = self
            .session
            .query_unpaged(query.as_str(), (limit as i32,))
            .await?;
        let all_transactions = rows
            .rows_typed::<ScyllaTransaction>()?
            .collect::<Result<Vec<_>, _>>()?;

        // Convert and filter in memory for account presence in JSON
        let converted_transactions: Result<Vec<SolanaTransaction>, _> = all_transactions
            .into_iter()
            .filter(|tx| tx.account_keys_json.contains(&format!("\"{}\"", account)))
            .take(limit as usize)
            .map(|tx| SolanaTransaction::from_scylla(tx))
            .collect();

        let filtered_transactions = converted_transactions?;

        Ok(filtered_transactions)
    }

    pub async fn get_transactions_with_log_pattern(
        &self,
        pattern: &str,
        limit: u32,
    ) -> Result<Vec<SolanaTransaction>, Box<dyn Error>> {
        // Note: ScyllaDB doesn't have direct pattern matching like ClickHouse
        // This is a simplified implementation - you might want to use a different approach
        // such as maintaining a separate index or using full-text search
        let query = format!(
            "SELECT * FROM {}.{} LIMIT ? ALLOW FILTERING",
            self.keyspace, self.transactions_table
        );

        let rows = self
            .session
            .query_unpaged(query.as_str(), (limit as i32,))
            .await?;
        let all_transactions = rows
            .rows_typed::<ScyllaTransaction>()?
            .collect::<Result<Vec<_>, _>>()?;

        // Filter and convert in memory (not ideal for large datasets)
        let converted_transactions: Result<Vec<SolanaTransaction>, _> = all_transactions
            .into_iter()
            .filter(|tx| tx.log_messages_json.contains(pattern))
            .take(limit as usize)
            .map(|tx| SolanaTransaction::from_scylla(tx))
            .collect();

        let filtered_transactions = converted_transactions?;

        Ok(filtered_transactions)
    }

    pub async fn get_failed_transactions_by_slot(
        &self,
        slot: u64,
    ) -> Result<Vec<SolanaTransaction>, Box<dyn Error>> {
        let query = format!(
            "SELECT * FROM {}.{} WHERE slot = ? AND success = false ALLOW FILTERING",
            self.keyspace, self.transactions_table
        );

        let rows = self
            .session
            .query_unpaged(query.as_str(), (slot as i64,))
            .await?;
        let scylla_transactions = rows
            .rows_typed::<ScyllaTransaction>()?
            .collect::<Result<Vec<_>, _>>()?;

        let transactions = scylla_transactions
            .into_iter()
            .map(|tx| SolanaTransaction::from_scylla(tx))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }
}
