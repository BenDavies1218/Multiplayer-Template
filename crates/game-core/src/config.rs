//! Configuration module for loading environment variables
//!
//! This module handles loading configuration from environment variables with sensible defaults.
//! Configuration is loaded from a .env file if present, falling back to defaults if not.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// Server host address (e.g., "127.0.0.1" or "0.0.0.0")
    pub server_host: String,
    /// Server port number
    pub server_port: u16,
    /// Fixed timestep frequency in Hz
    pub fixed_timestep_hz: f64,
    /// Network send interval frequency in Hz
    pub send_interval_hz: f64,
    /// Client timeout in seconds
    pub client_timeout_secs: i32,
    /// Interpolation buffer in milliseconds
    pub interpolation_buffer_ms: u64,
    /// Rust log level (e.g., "info", "debug", "trace")
    pub rust_log: String,
    /// Path to SSL certificate file
    pub cert_path: String,
    /// Path to SSL private key file
    pub key_path: String,
    /// Path to certificate digest file
    pub digest_path: String,
}

impl Config {
    /// Load configuration from environment variables with defaults
    pub fn load() -> Self {
        Self {
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),

            server_port: std::env::var("SERVER_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5888),

            fixed_timestep_hz: std::env::var("FIXED_TIMESTEP_HZ")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(64.0),

            send_interval_hz: std::env::var("SEND_INTERVAL_HZ")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(64.0),

            client_timeout_secs: std::env::var("CLIENT_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),

            interpolation_buffer_ms: std::env::var("INTERPOLATION_BUFFER_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),

            rust_log: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),

            cert_path: std::env::var("CERT_PATH")
                .unwrap_or_else(|_| "./certificates/cert.pem".to_string()),

            key_path: std::env::var("KEY_PATH")
                .unwrap_or_else(|_| "./certificates/key.pem".to_string()),

            digest_path: std::env::var("DIGEST_PATH")
                .unwrap_or_else(|_| "./certificates/digest.txt".to_string()),
        }
    }

    /// Get server address as SocketAddr
    pub fn server_addr(&self) -> SocketAddr {
        let ip: IpAddr = self.server_host
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
        SocketAddr::new(ip, self.server_port)
    }

    /// Get send interval as Duration
    pub fn send_interval(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.send_interval_hz)
    }

    /// Get interpolation buffer as Duration
    pub fn interpolation_buffer(&self) -> Duration {
        Duration::from_millis(self.interpolation_buffer_ms)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::load()
    }
}

/// Initialize environment variables from .env file
///
/// This should be called early in main() before any configuration is loaded.
/// It's safe to call this function multiple times or if the .env file doesn't exist.
pub fn init() {
    dotenvy::dotenv().ok(); // Ignore error if .env doesn't exist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::load();
        assert_eq!(config.fixed_timestep_hz, 64.0);
        assert!(config.server_port > 0);
    }

    #[test]
    fn test_server_addr() {
        let config = Config {
            server_host: "127.0.0.1".to_string(),
            server_port: 5000,
            ..Config::default()
        };
        let addr = config.server_addr();
        assert_eq!(addr.port(), 5000);
    }
}
