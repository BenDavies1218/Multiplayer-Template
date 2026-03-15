//! Game Server - Server-specific game logic
//!
//! This crate contains all server-side code:
//! - Server app building and transport setup
//! - Server game logic
//! - Player management
//! - Authoritative simulation

pub mod app;
pub mod diagnostics;
pub mod movement;
pub mod server;
pub mod server_config;
pub mod spawning;
pub mod transport;

// Re-export
pub use server::ServerPlugin;
pub use server_config::{GameServerConfig, ServerConnectionConfig};
pub use transport::{ExampleServer, ServerTransports, WebTransportCertificateSettings, start};
