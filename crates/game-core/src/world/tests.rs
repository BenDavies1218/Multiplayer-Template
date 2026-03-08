#[cfg(test)]
mod tests {
    use super::super::*;
    use super::super::processor::create_compound_collider;

    #[test]
    fn test_world_assets_default() {
        let assets = WorldAssets::default();
        assert!(assets.visual.is_none());
        assert!(assets.collision.is_none());
    }

    #[test]
    fn test_compound_collider_creation() {
        let _collider = create_compound_collider(vec![
            (Vec3::ZERO, Quat::IDENTITY, Collider::cuboid(1.0, 1.0, 1.0)),
        ]);

        // Just verify it compiles
        assert!(true);
    }
}
