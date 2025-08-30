use crate::types::{IndexEvent, SolanaAccount, SolanaTransaction};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use bs58;
use chrono::Utc;
use futures::{channel::mpsc, Sink, Stream, StreamExt};
use redis::{Commands, Connection};
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
        redis_client_connection: &mut Connection,
        stream_name: &str,
    ) -> Result<()> {
        while let Some(message) = stream.next().await {
            match message {
                Ok(update) => {
                    Self::process_update(update, redis_client_connection, stream_name).await?;
                }
                Err(error) => {
                    error!("Stream Error: {}", error);
                }
            }
        }
        Ok(())
    }

    pub async fn process_update(
        update: SubscribeUpdate,
        redis_client_connection: &mut Connection,
        stream_name: &str,
    ) -> Result<()> {
        match update.update_oneof {
            Some(subscribe_update::UpdateOneof::Account(account)) => {
                Self::handle_account_update(account, redis_client_connection, stream_name).await?;
            }
            Some(subscribe_update::UpdateOneof::Transaction(transaction)) => {
                Self::handle_transaction_update(transaction, redis_client_connection, stream_name)
                    .await?;
            }
            Some(subscribe_update::UpdateOneof::Slot(slot)) => {
                Self::handle_slot_update(slot, redis_client_connection).await?;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn handle_account_update(
        account_update: SubscribeUpdateAccount,
        redis_client_connection: &mut Connection,
        stream_name: &str,
    ) -> Result<()> {
        if let Some(solana_account) = Self::to_solana_account(account_update) {
            info!(
                "Account: pubkey={}, lamports={}, owner={}, executable={}",
                solana_account.pubkey,
                solana_account.lamports,
                solana_account.owner,
                solana_account.executable
            );

            let account_payload = serde_json::to_string(&IndexEvent::Account(solana_account))?;
            let payload = [("payload", account_payload)];
            redis_client_connection.xadd(stream_name, "*", &payload)?;
        }

        Ok(())
    }

    pub async fn handle_transaction_update(
        transaction_update: SubscribeUpdateTransaction,
        redis_client_connection: &mut Connection,
        stream_name: &str,
    ) -> Result<()> {
        if let Some(solana_transaction) = Self::to_solana_transaction(transaction_update) {
            info!(
                "Transaction: signature={}, slot={}, success={}",
                solana_transaction.signature, solana_transaction.slot, solana_transaction.success
            );

            let transaction_payload =
                serde_json::to_string(&IndexEvent::Transaction(solana_transaction))?;
            let payload = [("payload", transaction_payload)];
            redis_client_connection.xadd(stream_name, "*", &payload)?;
        }

        Ok(())
    }

    pub async fn handle_slot_update(
        slot_update: SubscribeUpdateSlot,
        redis_client_connection: &mut Connection,
    ) -> Result<()> {
        info!("Slot: {:?}", slot_update.slot);
        redis_client_connection.set("current_slot", slot_update.slot)?;

        Ok(())
    }

    fn to_solana_transaction(
        transaction_update: SubscribeUpdateTransaction,
    ) -> Option<SolanaTransaction> {
        if let Some(transaction_info) = transaction_update.transaction {
            let signature = bs58::encode(&transaction_info.signature).into_string();

            let (
                success,
                fee,
                compute_units_consumed,
                instructions,
                account_keys,
                log_messages,
                pre_balances,
                post_balances,
            ) = if let (Some(transaction), Some(meta)) = (
                transaction_info.transaction.as_ref(),
                transaction_info.meta.as_ref(),
            ) {
                let success = meta.err.is_none();
                let fee = Some(meta.fee);
                let compute_units_consumed = meta.compute_units_consumed;

                let mut instructions = Vec::new();
                if let Some(message) = transaction.message.as_ref() {
                    for instruction in &message.instructions {
                        let program_id_index = instruction.program_id_index as usize;
                        let program_id = if program_id_index < message.account_keys.len() {
                            bs58::encode(&message.account_keys[program_id_index]).into_string()
                        } else {
                            String::new()
                        };

                        let accounts: Vec<String> = instruction
                            .accounts
                            .iter()
                            .filter_map(|&idx| {
                                message
                                    .account_keys
                                    .get(idx as usize)
                                    .map(|key| bs58::encode(key).into_string())
                            })
                            .collect();

                        instructions.push(crate::types::TransactionInstruction {
                            program_id,
                            accounts,
                            data: general_purpose::STANDARD.encode(&instruction.data),
                        });
                    }
                }

                let account_keys: Vec<String> = if let Some(message) = transaction.message.as_ref()
                {
                    message
                        .account_keys
                        .iter()
                        .map(|key| bs58::encode(key).into_string())
                        .collect()
                } else {
                    Vec::new()
                };

                let log_messages: Vec<String> = meta.log_messages.clone();

                let pre_balances = meta.pre_balances.clone();
                let post_balances = meta.post_balances.clone();

                (
                    success,
                    fee,
                    compute_units_consumed,
                    instructions,
                    account_keys,
                    log_messages,
                    pre_balances,
                    post_balances,
                )
            } else {
                (
                    false,
                    None,
                    None,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            };

            Some(SolanaTransaction {
                signature,
                slot: transaction_update.slot,
                is_vote: transaction_info.is_vote,
                index: transaction_info.index,
                success,
                fee,
                compute_units_consumed,
                instructions,
                account_keys,
                log_messages,
                pre_balances,
                post_balances,
                timestamp: Utc::now(),
            })
        } else {
            None
        }
    }

    fn to_solana_account(account_update: SubscribeUpdateAccount) -> Option<SolanaAccount> {
        if let Some(account_info) = account_update.account {
            let pubkey = bs58::encode(&account_info.pubkey).into_string();
            let owner = bs58::encode(&account_info.owner).into_string();
            let data = general_purpose::STANDARD.encode(&account_info.data);

            let txn_signature = account_info
                .txn_signature
                .map(|sig| bs58::encode(&sig).into_string());

            Some(SolanaAccount {
                pubkey,
                lamports: account_info.lamports,
                owner,
                executable: account_info.executable,
                rent_epoch: account_info.rent_epoch,
                data,
                write_version: account_info.write_version,
                slot: account_update.slot,
                txn_signature,
                timestamp: Utc::now(),
            })
        } else {
            None
        }
    }
}
