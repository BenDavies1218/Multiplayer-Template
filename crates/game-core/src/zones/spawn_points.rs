use bevy::prelude::*;

/// Resource storing spawn point positions, populated from the triggers GLB.
/// Falls back to a default position if no spawn points are defined.
#[derive(Resource, Debug)]
pub struct SpawnPoints {
    pub points: Vec<Vec3>,
    pub default_position: Vec3,
    next_index: usize,
}

impl Default for SpawnPoints {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            default_position: Vec3::new(0.0, 3.0, 0.0),
            next_index: 0,
        }
    }
}

impl SpawnPoints {
    pub fn new(points: Vec<Vec3>, default_position: Vec3) -> Self {
        Self {
            points,
            default_position,
            next_index: 0,
        }
    }

    /// Get the next spawn position (round-robin).
    /// Returns the default position if no spawn points are defined.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Vec3 {
        if self.points.is_empty() {
            return self.default_position;
        }
        let pos = self.points[self.next_index];
        self.next_index = (self.next_index + 1) % self.points.len();
        pos
    }
}
