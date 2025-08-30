mod config;

use crate::config::Config;
use anyhow::Result;
use futures::SinkExt;
use redis::Client;
use tracing::info;
use yellowstone_gRPC::{client::YellowstoneClient, subscriptions::Subscriptions};

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

fn setup_rustls() {
    // Install default crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging();
    setup_rustls();

    dotenv::dotenv().ok();

    info!("Starting Solana Indexer Pipeline");

    let config = Config::from_env()?;

    let mut yellowstone_client = YellowstoneClient::create_yellowstone_client(
        &config.yellowstone_endpoint,
        config.yellowstone_token,
    )
    .await?;

    let (mut subscriber_tx, subscribe_rx) =
        YellowstoneClient::subscribe(&mut yellowstone_client).await?;
    let defi_subscription_request = Subscriptions::create_defi_subscription();

    subscriber_tx.send(defi_subscription_request).await?;

    info!("Subscribed to defi transactions. Starting stream processing...");
    let redis_client = Client::open(config.redis_url)?;
    let mut redis_connection = redis_client.get_connection()?;
    YellowstoneClient::handle_stream(subscribe_rx, &mut redis_connection, "yellowstone_gRPC_streams").await?;

    Ok(())
}
