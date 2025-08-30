mod config;

use anyhow::Result;
use futures::SinkExt;
use tracing::info;
use yellowstone_gRPC::{client::YellowstoneClient, subscriptions::Subscriptions};

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Install default crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    info!("Starting Solana Indexer Pipeline");
    println!("Starting Solana Indexer Pipeline");

    let config = Config::from_env()?;

    let mut yellowstone_client = YellowstoneClient::create_yellowstone_client(
        &config.yellowstone_endpoint,
        config.yellowstone_token,
    )
    .await?;

    println!("Created yellowstone client");

    let (mut subscriber_tx, subscribe_rx) =
        YellowstoneClient::subscribe(&mut yellowstone_client).await?;
    let defi_subscription_request = Subscriptions::create_defi_subscription();

    subscriber_tx.send(defi_subscription_request).await?;

    info!("Subscribed to defi transactions. Starting stream processing...");
    println!("Subscribed to defi transactions. Starting stream processing...");
    YellowstoneClient::handle_stream(subscribe_rx).await?;

    Ok(())
}
