//! Game Server - Server-specific game logic
//!
//! This crate contains all server-side code:
//! - Server app building and transport setup
//! - Server game logic
//! - Player management
//! - Authoritative simulation

pub mod app;
pub mod transport;
pub mod server;

// Re-export
pub use server::ServerPlugin;
pub use transport::{ExampleServer, ServerTransports, WebTransportCertificateSettings, start};
