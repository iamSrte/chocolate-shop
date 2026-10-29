use std::net::SocketAddr;

use anyhow::Context;

pub struct Config {
    pub addr: SocketAddr,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let addr = std::env::var("API_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".into())
            .parse()
            .context("API_ADDR must be host:port")?;
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        Ok(Self { addr, database_url })
    }
}
