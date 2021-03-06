#[cfg(test)]
mod tests {
    use amethyst::{
        ecs::{Builder, Join, ReadStorage, World, WorldExt},
        Error,
    };
    use amethyst_test::AmethystApplication;
    use object_status_model::config::StunPoints;

    use object_status_play::StunPointsReductionSystem;

    #[test]
    fn reduces_stun_points_each_tick() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_system(StunPointsReductionSystem::new(), "", &[])
            .with_effect(|world| create_entity_with_stun_points(world, 3))
            .with_assertion(|world| assert_entity_with_stun_points(world, 2))
            .with_assertion(|world| assert_entity_with_stun_points(world, 1))
            .with_assertion(|world| assert_entity_with_stun_points(world, 0))
            .with_assertion(|world| assert_entity_with_stun_points(world, 0))
            .run()
    }

    fn create_entity_with_stun_points(world: &mut World, points: u32) {
        world.create_entity().with(StunPoints::new(points)).build();
    }

    fn assert_entity_with_stun_points(world: &mut World, points: u32) {
        let stun_points = world
            .system_data::<ReadStorage<'_, StunPoints>>()
            .join()
            .next()
            .cloned()
            .expect("Expected entity with `StunPoints` to exist.");

        assert_eq!(StunPoints::new(points), stun_points);
    }
}
