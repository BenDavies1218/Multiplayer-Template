//! Game Core - Shared game logic and protocol
//!
//! This crate contains all the shared code between client and server:
//! - Network protocol definitions
//! - Shared game logic
//! - Common utilities and settings

pub mod protocol;
pub mod shared;
pub mod common;
pub mod config;
pub mod world;
pub mod movement;

// Re-export commonly used items
pub use protocol::*;
pub use shared::*;
pub use config::Config;
