//! Game Server - Server-specific game logic
//!
//! This crate contains all server-side code:
//! - Server game logic
//! - Player management
//! - Authoritative simulation

pub mod server;

// Re-export
pub use server::ServerPlugin;
