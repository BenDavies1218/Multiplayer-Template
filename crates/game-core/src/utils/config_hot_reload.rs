//! Hot-reload system for JSON config files.
//!
//! Polls config files for modification-time changes and silently updates
//! the corresponding Bevy Resource. On parse error the old config is kept.
//!
//! Compiled out on WASM where filesystem watching is not available.

#[cfg(not(target_family = "wasm"))]
mod inner {
    use std::any::TypeId;
    use std::collections::HashMap;
    use std::marker::PhantomData;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};

    use bevy::prelude::*;
    use serde::de::DeserializeOwned;

    use crate::utils::config_loader::resolve_asset_path;

    // -- Trait-object reload dispatch ------------------------------------------

    trait ConfigReloader: Send + Sync {
        fn reload(&self, world: &mut World, path: &Path);
    }

    struct TypedReloader<T> {
        filename: String,
        _marker: PhantomData<T>,
    }

    impl<T> ConfigReloader for TypedReloader<T>
    where
        T: Resource + DeserializeOwned + Default + Send + Sync + 'static,
    {
        fn reload(&self, world: &mut World, path: &Path) {
            match std::fs::read_to_string(path) {
                Ok(contents) => match serde_json::from_str::<T>(&contents) {
                    Ok(new_config) => {
                        world.insert_resource(new_config);
                        info!("[CONFIG RELOAD] Reloaded {}", self.filename);
                    }
                    Err(e) => {
                        warn!(
                            "[CONFIG RELOAD] Parse error in {}: {}. Keeping old config.",
                            self.filename, e
                        );
                    }
                },
                Err(e) => {
                    warn!(
                        "[CONFIG RELOAD] Could not read {}: {}. Keeping old config.",
                        self.filename, e
                    );
                }
            }
        }
    }

    // -- Watcher resource ------------------------------------------------------

    struct WatchedConfig {
        path: PathBuf,
        last_modified: Option<SystemTime>,
        reloader: Box<dyn ConfigReloader>,
    }

    #[derive(Resource)]
    pub struct ConfigWatcher {
        configs: HashMap<TypeId, WatchedConfig>,
        poll_interval: Duration,
        last_poll: Duration,
    }

    // -- Plugin ----------------------------------------------------------------

    pub struct ConfigHotReloadPlugin {
        pub poll_interval_secs: f32,
    }

    impl Default for ConfigHotReloadPlugin {
        fn default() -> Self {
            Self {
                poll_interval_secs: 1.0,
            }
        }
    }

    impl Plugin for ConfigHotReloadPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(ConfigWatcher {
                configs: HashMap::new(),
                poll_interval: Duration::from_secs_f32(self.poll_interval_secs),
                last_poll: Duration::ZERO,
            });
            app.add_systems(Update, poll_config_changes);
        }
    }

    // -- Extension trait -------------------------------------------------------

    pub trait ConfigWatchExt {
        fn watch_config<T>(&mut self, filename: &str) -> &mut Self
        where
            T: Resource + DeserializeOwned + Default + Send + Sync + 'static;
    }

    impl ConfigWatchExt for App {
        fn watch_config<T>(&mut self, filename: &str) -> &mut Self
        where
            T: Resource + DeserializeOwned + Default + Send + Sync + 'static,
        {
            let asset_path = resolve_asset_path();
            let path = PathBuf::from(format!("{}/config/{}", asset_path, filename));

            let last_modified = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());

            let reloader = Box::new(TypedReloader::<T> {
                filename: filename.to_string(),
                _marker: PhantomData,
            });

            self.world_mut()
                .resource_mut::<ConfigWatcher>()
                .configs
                .insert(
                    TypeId::of::<T>(),
                    WatchedConfig {
                        path,
                        last_modified,
                        reloader,
                    },
                );

            self
        }
    }

    // -- Polling system --------------------------------------------------------

    fn poll_config_changes(world: &mut World) {
        let now = world.resource::<Time<Real>>().elapsed();
        let (poll_interval, last_poll) = {
            let watcher = world.resource::<ConfigWatcher>();
            (watcher.poll_interval, watcher.last_poll)
        };

        if now - last_poll < poll_interval {
            return;
        }

        world.resource_mut::<ConfigWatcher>().last_poll = now;

        // First pass: collect type IDs of configs whose files have changed.
        let changed: Vec<TypeId> = {
            let mut watcher = world.resource_mut::<ConfigWatcher>();
            let mut changed = Vec::new();
            for (&type_id, watched) in watcher.configs.iter_mut() {
                let current_mtime = std::fs::metadata(&watched.path)
                    .ok()
                    .and_then(|m| m.modified().ok());

                if current_mtime != watched.last_modified && current_mtime.is_some() {
                    watched.last_modified = current_mtime;
                    changed.push(type_id);
                }
            }
            changed
        };

        if changed.is_empty() {
            return;
        }

        // Second pass: remove watcher, call reloaders, re-insert.
        let watcher = world.remove_resource::<ConfigWatcher>().unwrap();
        for type_id in &changed {
            if let Some(watched) = watcher.configs.get(type_id) {
                watched.reloader.reload(world, &watched.path.clone());
            }
        }
        world.insert_resource(watcher);
    }
}

// -- Public re-exports (platform-gated) ----------------------------------------

#[cfg(not(target_family = "wasm"))]
pub use inner::{ConfigHotReloadPlugin, ConfigWatchExt};

/// No-op plugin for WASM where filesystem watching is unavailable.
#[cfg(target_family = "wasm")]
pub struct ConfigHotReloadPlugin;

#[cfg(target_family = "wasm")]
impl Default for ConfigHotReloadPlugin {
    fn default() -> Self {
        Self
    }
}

#[cfg(target_family = "wasm")]
impl bevy::prelude::Plugin for ConfigHotReloadPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}
}

#[cfg(target_family = "wasm")]
pub trait ConfigWatchExt {
    fn watch_config<T>(&mut self, _filename: &str) -> &mut Self
    where
        T: bevy::prelude::Resource + serde::de::DeserializeOwned + Default + Send + Sync + 'static;
}

#[cfg(target_family = "wasm")]
impl ConfigWatchExt for bevy::prelude::App {
    fn watch_config<T>(&mut self, _filename: &str) -> &mut Self
    where
        T: bevy::prelude::Resource + serde::de::DeserializeOwned + Default + Send + Sync + 'static,
    {
        self
    }
}
