//! Structured JSON logging.
//!
//! Policy: log events, never request or response bodies, never query strings,
//! never headers. Anything that could identify a person or carry a secret
//! stays out of the logs.

use tracing::Level;
use tracing_subscriber::EnvFilter;

pub fn init(level: Level) {
    // `RUST_LOG` is deliberately ignored: the level comes from validated config.
    let filter = EnvFilter::new(level.as_str());
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(true)
        .with_target(false)
        .init();
}
