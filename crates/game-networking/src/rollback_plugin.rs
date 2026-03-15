//! Rollback plugin — initializes rollback threshold configuration.
//!
//! Must be added before any rollback checks run. Reads thresholds from
//! [`RollbackConfig`](game_core::core_config::RollbackConfig) and stores them
//! in a global static for use by the comparison functions in [`crate::rollback`].

use bevy::prelude::*;
use game_core::performance_config::RollbackConfig;

use crate::rollback;

pub struct RollbackPlugin {
    pub config: RollbackConfig,
}

impl Plugin for RollbackPlugin {
    fn build(&self, _app: &mut App) {
        rollback::init_rollback_config(self.config.clone());
    }
}
