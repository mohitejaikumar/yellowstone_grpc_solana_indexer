use anyhow::Result;
use std::env;

pub struct Config {
    pub yellowstone_endpoint: String,
    pub yellowstone_token: Option<String>,
    pub redis_url: String
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            yellowstone_endpoint: env::var("YELLOWSTONE_ENDPOINT")?,
            yellowstone_token: env::var("YELLOWSTONE_TOKEN").ok(),
            redis_url: env::var("REDIS_URL")?,
        })
    }
}
