//! Character movement systems
//!
//! Separated into client and server modules for clarity:
//! - `client`: Camera-relative movement with direct velocity control
//! - `server`: Server-authoritative movement validation

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;
