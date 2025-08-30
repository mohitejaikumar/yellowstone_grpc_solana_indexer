use crate::{config::Config, redis_client::RedisConsumer, scylla_client::ScyllaWriter};
use anyhow::Result;
use tracing::{error, info};

mod config;
mod processor;
mod redis_client;
mod scylla_client;
mod scylla_types;

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging();
    dotenv::dotenv().ok();

    let config = Config::from_env().unwrap();

    let redis_client = RedisConsumer::new(
        &config.redis_url,
        "yellowstone_gRPC_streams",
        "db_processor",
        "db_processor_consumer_1",
    )?;

    redis_client.create_consumer_group()?;

    info!("Consumer group created successfully");

    // Initialize ScyllaDB client
    let scylla_nodes: Vec<&str> = config.scylla_nodes.iter().map(|s| s.as_str()).collect();
    let mut writer = ScyllaWriter::new(
        scylla_nodes,
        "solana_indexer",
        "accounts",
        "transactions",
        1000,
    )
    .await
    .unwrap_or_else(|e| {
        error!("Error creating ScyllaDB writer: {}", e);
        std::process::exit(1);
    });

    // Keyspace and table creation
    writer.create_keyspace().await.unwrap_or_else(|e| {
        error!("Error creating keyspace: {}", e);
        std::process::exit(1);
    });

    writer.create_accounts_table().await.unwrap_or_else(|e| {
        error!("Error creating accounts table: {}", e);
        std::process::exit(1);
    });
    writer
        .create_transactions_table()
        .await
        .unwrap_or_else(|e| {
            error!("Error creating transactions table: {}", e);
            std::process::exit(1);
        });

    loop {
        match redis_client.consume_message(5, 0) {
            Ok(messages) => {
                info!("Consumed {} messages", messages.len());
                processor::process(messages, &mut writer, &redis_client).await?;
            }
            Err(e) => {
                error!("Error consuming message: {}", e);
            }
        }
    }
}
