use anyhow::Result;
use futures::{channel::mpsc, Sink, Stream, StreamExt};
use tonic::{transport::ClientTlsConfig, Status};
use tracing::{error, info};
use yellowstone_grpc_client::{
    GeyserGrpcBuilderError, GeyserGrpcClient, GeyserGrpcClientResult, Interceptor,
};
use yellowstone_grpc_proto::geyser::{
    subscribe_update, SubscribeRequest, SubscribeUpdate, SubscribeUpdateAccount,
    SubscribeUpdateSlot, SubscribeUpdateTransaction,
};

pub struct YellowstoneClient;

impl YellowstoneClient {
    pub async fn create_yellowstone_client(
        endpoint: &str,
        token: Option<String>,
    ) -> Result<GeyserGrpcClient<impl Interceptor>, GeyserGrpcBuilderError> {
        let builder = GeyserGrpcClient::build_from_shared(endpoint.to_string())?
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .x_token(token)?;

        let client = builder.connect().await?;
        return Ok(client);
    }

    pub async fn subscribe(
        client: &mut GeyserGrpcClient<impl Interceptor>,
    ) -> GeyserGrpcClientResult<(
        impl Sink<SubscribeRequest, Error = mpsc::SendError>,
        impl Stream<Item = Result<SubscribeUpdate, Status>>,
    )> {
        client.subscribe().await
    }

    pub async fn handle_stream(
        mut stream: impl Stream<Item = Result<SubscribeUpdate, Status>> + Unpin,
    ) -> Result<()> {
        while let Some(message) = stream.next().await {
            match message {
                Ok(update) => {
                    println!("Processing update: {:?}", update);
                    Self::process_update(update).await?;
                }
                Err(error) => {
                    error!("Stream Error: {}", error);
                }
            }
        }
        Ok(())
    }

    pub async fn process_update(update: SubscribeUpdate) -> Result<()> {
        match update.update_oneof {
            Some(subscribe_update::UpdateOneof::Account(account)) => {
                Self::handle_account_update(account).await?;
            }
            Some(subscribe_update::UpdateOneof::Transaction(transaction)) => {
                Self::handle_transaction_update(transaction).await?;
            }
            Some(subscribe_update::UpdateOneof::Slot(slot)) => {
                Self::handle_slot_update(slot).await?;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn handle_account_update(account_update: SubscribeUpdateAccount) -> Result<()> {
        if let Some(account) = account_update.account {
            // TODO: logic to convert it to what i have to save in database , and pass on to redis
            info!("Account: {:?}", account);
            println!("Account: {:?}", account);
        }

        Ok(())
    }

    pub async fn handle_transaction_update(
        transaction_update: SubscribeUpdateTransaction,
    ) -> Result<()> {
        if let Some(transaction) = transaction_update.transaction {
            // TODO: logic to convert it to what i have to save in database , and pass on to redis
            info!("Transaction: {:?}", transaction);
            println!("Transaction: {:?}", transaction);
        }

        Ok(())
    }

    pub async fn handle_slot_update(slot_update: SubscribeUpdateSlot) -> Result<()> {
        // TODO: logic to convert it to what i have to save in database , and pass on to redis
        info!("Slot: {:?}", slot_update.slot);
        println!("Slot: {:?}", slot_update.slot);

        Ok(())
    }
}
