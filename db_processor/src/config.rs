use anyhow::Result;
use std::env;

pub struct Config {
    pub redis_url: String
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            redis_url: env::var("REDIS_URL")?,
        })
    }
}
