//! Typed configuration read from environment variables.
//!
//! The process refuses to start on any missing or invalid required value
//! (fail closed). Error messages name the variable but never echo its value,
//! because `DATABASE_URL` carries credentials.

use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use sqlx::postgres::PgConnectOptions;
use tracing::Level;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_LOG_LEVEL: &str = "info";

/// Upper bound for a whole request. Slow clients must not hold resources.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// No endpoint in this phase accepts a large payload; 64 KiB is generous for JSON.
pub const MAX_BODY_BYTES: usize = 64 * 1024;
/// `/ready` must answer quickly even if the database hangs, so orchestrators
/// are not left waiting.
pub const READY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("required environment variable {0} is missing or empty")]
    Missing(&'static str),
    #[error("environment variable {name} is invalid: {reason}")]
    Invalid {
        name: &'static str,
        reason: &'static str,
    },
}

/// A string that must never appear in logs or debug output.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: Secret,
    pub bind_addr: SocketAddr,
    pub log_level: Level,
}

impl Config {
    /// Reads the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| match std::env::var(name) {
            Ok(v) => Some(v),
            // A non-UTF-8 value is passed through as an unusable string so
            // that validation reports it as invalid instead of "missing".
            Err(std::env::VarError::NotUnicode(_)) => Some(String::from("\u{FFFD}")),
            Err(std::env::VarError::NotPresent) => None,
        })
    }

    /// Same as [`Config::from_env`] with an injectable source, so tests never
    /// mutate the real process environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let get = |name: &'static str| lookup(name).filter(|v| !v.trim().is_empty());

        let database_url = get("DATABASE_URL").ok_or(ConfigError::Missing("DATABASE_URL"))?;
        // Parsing here means a typo fails at startup, not on the first request.
        // The parser error is dropped on purpose: it may quote the URL.
        let invalid_url = || ConfigError::Invalid {
            name: "DATABASE_URL",
            reason: "not a valid PostgreSQL connection URL",
        };
        // The parser alone accepts other schemes, so check the scheme too.
        if !(database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")) {
            return Err(invalid_url());
        }
        PgConnectOptions::from_str(&database_url).map_err(|_| invalid_url())?;

        let bind_addr = get("BIND_ADDR").unwrap_or_else(|| DEFAULT_BIND_ADDR.to_owned());
        let bind_addr =
            bind_addr
                .trim()
                .parse::<SocketAddr>()
                .map_err(|_| ConfigError::Invalid {
                    name: "BIND_ADDR",
                    reason: "expected host:port such as 127.0.0.1:8080",
                })?;

        let log_level = get("LOG_LEVEL").unwrap_or_else(|| DEFAULT_LOG_LEVEL.to_owned());
        let log_level = log_level
            .trim()
            .parse::<Level>()
            .map_err(|_| ConfigError::Invalid {
                name: "LOG_LEVEL",
                reason: "expected one of error, warn, info, debug, trace",
            })?;

        Ok(Self {
            database_url: Secret(database_url),
            bind_addr,
            log_level,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const URL: &str = "postgres://user:pass@localhost:5432/db";

    fn load(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        Config::from_lookup(|k| map.get(k).cloned())
    }

    #[test]
    fn defaults_apply_when_only_database_url_is_set() {
        let c = load(&[("DATABASE_URL", URL)]).unwrap();
        assert_eq!(c.bind_addr, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(c.log_level, Level::INFO);
    }

    #[test]
    fn explicit_values_are_used() {
        let c = load(&[
            ("DATABASE_URL", URL),
            ("BIND_ADDR", "0.0.0.0:9000"),
            ("LOG_LEVEL", "debug"),
        ])
        .unwrap();
        assert_eq!(c.bind_addr.port(), 9000);
        assert_eq!(c.log_level, Level::DEBUG);
    }

    #[test]
    fn missing_database_url_fails() {
        assert_eq!(load(&[]).unwrap_err(), ConfigError::Missing("DATABASE_URL"));
    }

    #[test]
    fn blank_database_url_fails() {
        assert_eq!(
            load(&[("DATABASE_URL", "   ")]).unwrap_err(),
            ConfigError::Missing("DATABASE_URL")
        );
    }

    #[test]
    fn non_postgres_database_url_fails() {
        let err = load(&[("DATABASE_URL", "mysql://u:p@h/db")]).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                name: "DATABASE_URL",
                ..
            }
        ));
    }

    #[test]
    fn invalid_bind_addr_fails() {
        for bad in [
            "localhost",
            "8080",
            "127.0.0.1",
            "127.0.0.1:99999",
            "nope:1",
        ] {
            let err = load(&[("DATABASE_URL", URL), ("BIND_ADDR", bad)]).unwrap_err();
            assert!(
                matches!(
                    err,
                    ConfigError::Invalid {
                        name: "BIND_ADDR",
                        ..
                    }
                ),
                "{bad}"
            );
        }
    }

    #[test]
    fn invalid_log_level_fails() {
        let err = load(&[("DATABASE_URL", URL), ("LOG_LEVEL", "loud")]).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                name: "LOG_LEVEL",
                ..
            }
        ));
    }

    #[test]
    fn errors_and_debug_output_never_contain_the_password() {
        let err =
            load(&[("DATABASE_URL", "postgres://user:hunter2@host:notaport/db")]).unwrap_err();
        assert!(!err.to_string().contains("hunter2"));
        assert!(!format!("{err:?}").contains("hunter2"));

        let c = load(&[("DATABASE_URL", "postgres://user:hunter2@localhost/db")]).unwrap();
        assert!(!format!("{c:?}").contains("hunter2"));
    }
}
