use crate::{config::Config, redis_client::RedisConsumer};
use redis::RedisResult;
use tracing::{error, info};

mod config;
mod logger;
mod redis_client;

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

#[tokio::main]
async fn main() -> RedisResult<()> {
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

    loop {
        match redis_client.consume_message(5, 0) {
            Ok(messages) => {
                info!("Consumed {} messages", messages.len());
                for (message_id, event) in messages {
                    info!("Message ID: {}", message_id);
                    logger::logging(event);
                }
            }
            Err(e) => {
                error!("Error consuming message: {}", e);
            }
        }
    }
}
