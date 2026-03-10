//! Generic JSON config loader with error-tolerant fallback to defaults

use serde::de::DeserializeOwned;

/// Base path for config files relative to the assets directory.
const CONFIG_DIR: &str = "config";

/// Default asset path used when no env var is set.
const DEFAULT_ASSET_PATH: &str = "../../assets";

/// Resolve the base asset path.
///
/// 1. `ASSET_PATH` env var (if set)
/// 2. Falls back to `../../assets`
pub fn resolve_asset_path() -> String {
    std::env::var("ASSET_PATH").unwrap_or_else(|_| DEFAULT_ASSET_PATH.to_string())
}

/// Load a JSON config file from `{asset_path}/config/{filename}`.
///
/// The asset path is resolved via [`resolve_asset_path`].
///
/// - If the file is missing, logs a warning and returns `T::default()`.
/// - If the JSON is invalid, logs an error and returns `T::default()`.
/// - Missing fields in the JSON are filled from `T::default()` via `#[serde(default)]`.
pub fn load_config<T: DeserializeOwned + Default>(filename: &str) -> T {
    let asset_path = resolve_asset_path();
    load_config_from(&asset_path, filename)
}

/// Load a JSON config file from a specific asset base path.
///
/// Same behavior as [`load_config`] but with an explicit base path.
pub fn load_config_from<T: DeserializeOwned + Default>(assets_base: &str, filename: &str) -> T {
    let path = format!("{}/{}/{}", assets_base, CONFIG_DIR, filename);
    match std::fs::read_to_string(&path) {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(config) => {
                eprintln!("[CONFIG] Loaded {}", path);
                config
            }
            Err(e) => {
                eprintln!("[CONFIG ERROR] Failed to parse {}: {}. Using defaults.", path, e);
                T::default()
            }
        },
        Err(e) => {
            eprintln!("[CONFIG WARNING] Could not read {}: {}. Using defaults.", path, e);
            T::default()
        }
    }
}
