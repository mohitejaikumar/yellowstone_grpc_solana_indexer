use tracing::{error, info};
use yellowstone_gRPC::types::{IndexEvent, SolanaAccount, SolanaTransaction};

use crate::{redis_client::RedisConsumer, scylla_client::ScyllaWriter};
use anyhow::Result;

pub async fn process(
    messages: Vec<(String, IndexEvent)>,
    scylla_writer: &mut ScyllaWriter,
    redis_client: &RedisConsumer,
) -> Result<()> {
    let mut accounts = Vec::<SolanaAccount>::new();
    let mut transactions = Vec::<SolanaTransaction>::new();
    let mut message_ids = Vec::<String>::new();

    for (message_id, event) in messages {
        info!("Message ID: {}", message_id);
        message_ids.push(message_id);
        match event {
            IndexEvent::Transaction(transaction) => {
                println!("{}", transaction);
                transactions.push(transaction);
            }
            IndexEvent::Account(account) => {
                println!("{}", account);
                accounts.push(account);
            }
            IndexEvent::Slot(slot) => {
                println!("{}", slot);
            }
            IndexEvent::Block(block) => {
                println!("{}", block);
            }
        }
    }

    scylla_writer
        .add_accounts(accounts)
        .await
        .unwrap_or_else(|e| {
            error!("Error adding accounts to ScyllaDB: {}", e);
            std::process::exit(1);
        });

    scylla_writer
        .add_transactions(transactions)
        .await
        .unwrap_or_else(|e| {
            error!("Error adding transactions to ScyllaDB: {}", e);
            std::process::exit(1);
        });

    scylla_writer.flush_all_batches().await.unwrap_or_else(|e| {
        error!("Error flushing all batches: {}", e);
        std::process::exit(1);
    });
    info!("Flushed all batches successfully");
    redis_client
        .acknowledge(message_ids.as_slice())
        .unwrap_or_else(|e| {
            error!("Error acknowledging messages: {}", e);
            std::process::exit(1);
        });
    info!("Acknowledged messages successfully");
    Ok(())
}
