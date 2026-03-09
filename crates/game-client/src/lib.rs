//! Game Client - Client-specific game logic and rendering
//!
//! This crate contains all client-side code:
//! - Client app building and transport setup
//! - Client input handling
//! - Rendering and visual effects
//! - Client-side prediction and interpolation

pub mod app;
pub mod client;
pub mod client_config;
pub mod renderer;
pub mod transport;

// Re-export
pub use client::ClientPlugin;
pub use client_config::GameClientConfig;
pub use renderer::FirstPersonPlugin;
pub use transport::{ExampleClient, ClientTransports, connect};
