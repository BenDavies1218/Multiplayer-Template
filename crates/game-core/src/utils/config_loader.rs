//! Generic JSON config loader with error-tolerant fallback to defaults

use serde::de::DeserializeOwned;

/// Base path for config files relative to the assets directory.
const CONFIG_DIR: &str = "config";

/// Load a JSON config file from `{assets_base}/config/{filename}`.
///
/// - If the file is missing, logs a warning and returns `T::default()`.
/// - If the JSON is invalid, logs an error and returns `T::default()`.
/// - Missing fields in the JSON are filled from `T::default()` via `#[serde(default)]`.
pub fn load_config<T: DeserializeOwned + Default>(assets_base: &str, filename: &str) -> T {
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
