//! Runtime config, read once at startup from env vars. See `.env.example` for every var.

use std::{net::SocketAddr, time::Duration};

use anyhow::{Context, ensure};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug)]
pub struct Config {
    pub db: DbConfig,
    pub http: HttpConfig,
    pub addr: SocketAddr,
    pub log_format: LogFormat,
}

#[derive(Debug)]
pub struct DbConfig {
    pub url: SecretString,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
}

/// Limits applied by the HTTP middleware stack.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub request_timeout: Duration,
    pub body_limit_bytes: usize,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            body_limit_bytes: 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable, for local dev.
    Pretty,
    /// One JSON object per line, for production log shippers.
    Json,
}

/// Env vars as read by `envy`: one field per var, named in snake_case.
#[derive(Deserialize)]
struct RawConfig {
    #[serde(default = "default_api_addr")]
    api_addr: SocketAddr,
    database_url: SecretString,
    #[serde(default = "default_db_max_connections")]
    db_max_connections: u32,
    #[serde(default = "default_db_acquire_timeout_secs")]
    db_acquire_timeout_secs: u64,
    #[serde(default = "default_log_format")]
    log_format: LogFormat,
    #[serde(default = "default_request_timeout_secs")]
    request_timeout_secs: u64,
    #[serde(default = "default_body_limit_bytes")]
    body_limit_bytes: usize,
}

fn default_api_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 3000))
}
fn default_db_max_connections() -> u32 {
    10
}
fn default_db_acquire_timeout_secs() -> u64 {
    5
}
fn default_log_format() -> LogFormat {
    LogFormat::Pretty
}
fn default_request_timeout_secs() -> u64 {
    HttpConfig::default().request_timeout.as_secs()
}
fn default_body_limit_bytes() -> usize {
    HttpConfig::default().body_limit_bytes
}

impl Config {
    /// Loads `.env` if present, then reads the process env. Real env vars win over `.env`.
    pub fn load() -> anyhow::Result<Self> {
        if let Err(err) = dotenvy::dotenv()
            && !err.not_found()
        {
            return Err(err).context("failed to read .env");
        }
        Self::from_vars(std::env::vars())
    }

    /// Parses and validates config from `(name, value)` pairs. Tests call this directly.
    pub fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> anyhow::Result<Self> {
        let raw: RawConfig = envy::from_iter(vars).context("invalid config env vars")?;
        raw.validate()
    }
}

impl RawConfig {
    fn validate(self) -> anyhow::Result<Config> {
        ensure!(
            self.db_max_connections > 0,
            "DB_MAX_CONNECTIONS must be > 0"
        );
        ensure!(
            self.db_acquire_timeout_secs > 0,
            "DB_ACQUIRE_TIMEOUT_SECS must be > 0"
        );
        ensure!(
            self.request_timeout_secs > 0,
            "REQUEST_TIMEOUT_SECS must be > 0"
        );
        ensure!(self.body_limit_bytes > 0, "BODY_LIMIT_BYTES must be > 0");
        Ok(Config {
            addr: self.api_addr,
            db: DbConfig {
                url: self.database_url,
                max_connections: self.db_max_connections,
                acquire_timeout: Duration::from_secs(self.db_acquire_timeout_secs),
            },
            http: HttpConfig {
                request_timeout: Duration::from_secs(self.request_timeout_secs),
                body_limit_bytes: self.body_limit_bytes,
            },
            log_format: self.log_format,
        })
    }
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn only_database_url_is_required() {
        let config = Config::from_vars(vars(&[("DATABASE_URL", "postgres://x")])).unwrap();
        assert_eq!(config.addr, default_api_addr());
        assert_eq!(config.db.url.expose_secret(), "postgres://x");
        assert_eq!(config.db.max_connections, 10);
        assert_eq!(config.http.request_timeout, Duration::from_secs(30));
        assert_eq!(config.log_format, LogFormat::Pretty);
    }

    #[test]
    fn missing_database_url_fails() {
        assert!(Config::from_vars(vars(&[])).is_err());
    }

    #[test]
    fn reads_overrides() {
        let config = Config::from_vars(vars(&[
            ("DATABASE_URL", "postgres://x"),
            ("API_ADDR", "0.0.0.0:8080"),
            ("DB_MAX_CONNECTIONS", "3"),
            ("LOG_FORMAT", "json"),
            ("BODY_LIMIT_BYTES", "2048"),
        ]))
        .unwrap();
        assert_eq!(config.addr.port(), 8080);
        assert_eq!(config.db.max_connections, 3);
        assert_eq!(config.log_format, LogFormat::Json);
        assert_eq!(config.http.body_limit_bytes, 2048);
    }

    #[test]
    fn rejects_bad_values() {
        for (key, value) in [
            ("API_ADDR", "nope"),
            ("LOG_FORMAT", "xml"),
            ("DB_MAX_CONNECTIONS", "0"),
            ("REQUEST_TIMEOUT_SECS", "-1"),
        ] {
            let result = Config::from_vars(vars(&[("DATABASE_URL", "postgres://x"), (key, value)]));
            assert!(result.is_err(), "{key}={value} should fail");
        }
    }

    #[test]
    fn debug_redacts_database_url() {
        let config =
            Config::from_vars(vars(&[("DATABASE_URL", "postgres://u:hunter2@h/db")])).unwrap();
        assert!(!format!("{config:?}").contains("hunter2"));
    }
}
