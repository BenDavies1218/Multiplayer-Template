//! Generic JSON config loader with error-tolerant fallback to defaults

use serde::de::DeserializeOwned;

/// Base path for config files relative to the assets directory.
const CONFIG_DIR: &str = "config";

/// Default asset path used when no env var is set.
const DEFAULT_ASSET_PATH: &str = "assets";

/// Default Bevy `AssetPlugin` file_path for local development.
///
/// Resolved relative to `CARGO_MANIFEST_DIR` (i.e. the app's package dir).
const DEFAULT_BEVY_ASSET_PATH: &str = "../../assets";

/// Resolve the base asset path for config file loading (relative to CWD).
///
/// 1. `ASSET_PATH` env var (if set)
/// 2. Falls back to `assets`
pub fn resolve_asset_path() -> String {
    std::env::var("ASSET_PATH").unwrap_or_else(|_| DEFAULT_ASSET_PATH.to_string())
}

/// Resolve the asset path for Bevy's `AssetPlugin::file_path`.
///
/// When `ASSET_PATH` is set (e.g. Docker), returns that absolute path
/// (absolute paths in `PathBuf::join` replace the root, so Bevy uses it directly).
/// For WASM, returns `"assets"` (Trunk copies assets into dist/).
/// Otherwise falls back to `../../assets` which is correct relative to
/// each app's `CARGO_MANIFEST_DIR` during local development.
pub fn resolve_asset_path_for_bevy() -> String {
    if let Ok(path) = std::env::var("ASSET_PATH") {
        return path;
    }
    #[cfg(target_family = "wasm")]
    {
        "assets".to_string()
    }
    #[cfg(not(target_family = "wasm"))]
    {
        DEFAULT_BEVY_ASSET_PATH.to_string()
    }
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
                eprintln!(
                    "[CONFIG ERROR] Failed to parse {}: {}. Using defaults.",
                    path, e
                );
                T::default()
            }
        },
        Err(e) => {
            eprintln!(
                "[CONFIG WARNING] Could not read {}: {}. Using defaults.",
                path, e
            );
            T::default()
        }
    }
}
