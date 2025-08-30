use anyhow::Result;
use std::env;

pub struct Config {
    pub redis_url: String,
    pub scylla_nodes: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let scylla_nodes_str =
            env::var("SCYLLA_NODES").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
        let scylla_nodes = scylla_nodes_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            redis_url: env::var("REDIS_URL")?,
            scylla_nodes,
        })
    }
}
