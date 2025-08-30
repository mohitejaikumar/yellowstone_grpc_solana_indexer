use redis::{
    Client, Commands, RedisResult,
    streams::{StreamReadOptions, StreamReadReply},
};
use tracing::info;
use yellowstone_gRPC::types::IndexEvent;

pub struct RedisConsumer {
    client: Client,
    stream_name: String,
    group_name: String,
    consumer_name: String,
}

impl RedisConsumer {
    pub fn new(
        redis_url: &str,
        stream_name: &str,
        group_name: &str,
        consumer_name: &str,
    ) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client: client,
            stream_name: stream_name.to_string(),
            group_name: group_name.to_string(),
            consumer_name: consumer_name.to_string(),
        })
    }

    pub fn create_consumer_group(&self) -> RedisResult<()> {
        let mut connection = self.client.get_connection()?;

        match connection.xgroup_create_mkstream::<&str, &str, &str, ()>(
            &self.stream_name,
            &self.group_name,
            "0",
        ) {
            Ok(_) => println!("Consumer group '{}' created successfully", self.group_name),
            Err(e) => {
                if e.to_string().contains("BUSYGROUP") {
                    println!("Consumer group '{}' already exists", self.group_name);
                } else {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn acknowledge(&self, message_ids: &[String]) -> RedisResult<i64> {
        let mut conn = self.client.get_connection()?;

        let ack_count: i64 = conn.xack(&self.stream_name, &self.group_name, message_ids)?;

        info!("Acknowledged {} messages", ack_count);

        Ok(ack_count)
    }

    pub fn consume_message(
        &self,
        count: usize,
        block_ms: usize,
    ) -> RedisResult<Vec<(String, IndexEvent)>> {
        let mut conn = self.client.get_connection()?;

        let result: StreamReadReply = conn.xread_options(
            &[&self.stream_name],
            &[">"],
            &StreamReadOptions::default()
                .group(&self.group_name, &self.consumer_name)
                .count(count)
                .block(block_ms),
        )?;

        let mut messages = Vec::new();

        for stream in result.keys {
            for message in stream.ids {
                info!("ID: {}", message.id);

                if let Some(value) = message.map.get("payload") {
                    let payload: String = redis::from_redis_value(value)?;
                    match serde_json::from_str::<IndexEvent>(&payload) {
                        Ok(event) => {
                            info!("Got IndexEvent: {:?}", event);
                            messages.push((message.id, event));
                        }
                        Err(e) => {
                            info!("Failed to deserialize IndexEvent: {}", e);
                        }
                    }
                }
            }
        }

        Ok(messages)

        // Extract the messages and message_id from result
    }
}
