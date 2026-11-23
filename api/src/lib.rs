//! Trevio API server library.
//!
//! The binary in `main.rs` is a thin wrapper so that everything else can be
//! exercised by integration tests without opening a real socket.

pub mod app;
pub mod config;
pub mod error;
pub mod telemetry;
