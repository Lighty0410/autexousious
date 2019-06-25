use amethyst::{
    ecs::{Entity, System, SystemData, Write, WriteStorage},
    shred::Resources,
    shrev::{EventChannel, ReaderId},
};
use derivative::Derivative;
use derive_new::new;
use kinematic_model::config::{Position, Velocity};
use object_model::play::{Mirrored, ParentObject};
use shred_derive::SystemData;
use spawn_model::{config::Spawn, play::SpawnEvent};
use typename_derive::TypeName;

/// Spawns `GameObject`s.
#[derive(Debug, Default, TypeName, new)]
pub struct SpawnGameObjectRectifySystem {
    /// Reader ID for the `SpawnEvent` channel.
    #[new(default)]
    spawn_event_rid: Option<ReaderId<SpawnEvent>>,
}

#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct SpawnGameObjectRectifySystemData<'s> {
    /// `SpawnEvent` channel.
    #[derivative(Debug = "ignore")]
    pub spawn_ec: Write<'s, EventChannel<SpawnEvent>>,
    /// `ParentObject` components.
    #[derivative(Debug = "ignore")]
    pub parent_objects: WriteStorage<'s, ParentObject>,
    /// `Position<f32>` components.
    #[derivative(Debug = "ignore")]
    pub positions: WriteStorage<'s, Position<f32>>,
    /// `Velocity<f32>` components.
    #[derivative(Debug = "ignore")]
    pub velocities: WriteStorage<'s, Velocity<f32>>,
    /// `Mirrored` components.
    #[derivative(Debug = "ignore")]
    pub mirroreds: WriteStorage<'s, Mirrored>,
}

impl SpawnGameObjectRectifySystem {
    /// Returns the rectified `Position<f32>` for the spawned entity.
    fn position_rectify(
        positions: &WriteStorage<'_, Position<f32>>,
        spawn: &Spawn,
        entity_parent: Entity,
        mirrored_parent: Option<Mirrored>,
    ) -> Position<f32> {
        let spawn_position = spawn.position;
        let spawn_position_x = if let Some(Mirrored(true)) = mirrored_parent {
            -spawn_position.x
        } else {
            spawn_position.x
        };
        let mut position = Position::new(
            spawn_position_x as f32,
            spawn_position.y as f32,
            spawn_position.z as f32,
        );
        if let Some(position_parent) = positions.get(entity_parent) {
            *position = *position + **position_parent;
        }
        position
    }

    /// Returns the rectified `Velocity<f32>` for the spawned entity.
    fn velocity_rectify(
        velocities: &WriteStorage<'_, Velocity<f32>>,
        spawn: &Spawn,
        entity_parent: Entity,
        mirrored_parent: Option<Mirrored>,
    ) -> Velocity<f32> {
        let spawn_velocity = spawn.velocity;
        let spawn_velocity_x = if let Some(Mirrored(true)) = mirrored_parent {
            -spawn_velocity.x
        } else {
            spawn_velocity.x
        };
        let mut velocity = Velocity::new(
            spawn_velocity_x as f32,
            spawn_velocity.y as f32,
            spawn_velocity.z as f32,
        );
        if let Some(velocity_parent) = velocities.get(entity_parent) {
            *velocity = *velocity + **velocity_parent;
        }
        velocity
    }

    /// Returns the rectified `Mirrored` for the spawned entity.
    fn mirrored_rectify(mirrored_parent: Option<Mirrored>) -> Mirrored {
        mirrored_parent.unwrap_or(Mirrored(false))
    }
}

impl<'s> System<'s> for SpawnGameObjectRectifySystem {
    type SystemData = SpawnGameObjectRectifySystemData<'s>;

    fn run(
        &mut self,
        SpawnGameObjectRectifySystemData {
            spawn_ec,
            mut parent_objects,
            mut positions,
            mut velocities,
            mut mirroreds,
        }: Self::SystemData,
    ) {
        let spawn_event_rid = self
            .spawn_event_rid
            .as_mut()
            .expect("Expected `spawn_event_rid` field to be set.");

        spawn_ec.read(spawn_event_rid).for_each(|ev| {
            let spawn = &ev.spawn;
            let entity_parent = ev.entity_parent;
            let entity_spawned = ev.entity_spawned;
            let mirrored_parent = mirroreds.get(entity_parent).copied();

            let position =
                Self::position_rectify(&positions, spawn, entity_parent, mirrored_parent);
            let velocity =
                Self::velocity_rectify(&velocities, spawn, entity_parent, mirrored_parent);
            let mirrored = Self::mirrored_rectify(mirrored_parent);

            parent_objects
                .insert(entity_spawned, ParentObject::new(ev.entity_parent))
                .expect("Failed to insert `ParentObject` component.");
            positions
                .insert(entity_spawned, position)
                .expect("Failed to insert `Position` component.");
            velocities
                .insert(entity_spawned, velocity)
                .expect("Failed to insert `Velocity` component.");
            mirroreds
                .insert(entity_spawned, mirrored)
                .expect("Failed to insert `Mirrored` component.");
        });
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.spawn_event_rid = Some(
            res.fetch_mut::<EventChannel<SpawnEvent>>()
                .register_reader(),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use amethyst::{
        ecs::{Builder, Entity, World},
        shrev::EventChannel,
        Error,
    };
    use application_test_support::AutexousiousApplication;
    use asset_model::config::AssetSlug;
    use energy_loading::EnergyLoadingStatus;
    use energy_model::loaded::Energy;
    use energy_prefab::EnergyPrefab;
    use kinematic_model::config::{Position, Velocity};
    use loading::ObjectAssetLoadingSystem;
    use object_model::play::Mirrored;
    use spawn_model::{config::Spawn, play::SpawnEvent};
    use typename::TypeName;

    use super::SpawnGameObjectRectifySystem;

    #[test]
    fn sets_position_and_velocity_relative_to_parent() -> Result<(), Error> {
        AutexousiousApplication::config_base()
            .with_system(
                SpawnGameObjectRectifySystem::new(),
                SpawnGameObjectRectifySystem::type_name(),
                &[ObjectAssetLoadingSystem::<
                    Energy,
                    EnergyPrefab,
                    EnergyLoadingStatus,
                >::type_name()],
            )
            .with_setup(|world| spawn_entity(world, false))
            .with_assertion(|world| {
                assert_spawn_values(
                    world,
                    Position::<f32>::new(11., 22., 33.),
                    Velocity::<f32>::new(44., 55., 66.),
                    Mirrored(false),
                )
            })
            .run()
    }

    #[test]
    fn sets_mirrored_position_and_velocity_when_parent_mirrored() -> Result<(), Error> {
        AutexousiousApplication::config_base()
            .with_system(
                SpawnGameObjectRectifySystem::new(),
                SpawnGameObjectRectifySystem::type_name(),
                &[ObjectAssetLoadingSystem::<
                    Energy,
                    EnergyPrefab,
                    EnergyLoadingStatus,
                >::type_name()],
            )
            .with_setup(|world| spawn_entity(world, true))
            .with_assertion(|world| {
                assert_spawn_values(
                    world,
                    Position::<f32>::new(-9., 22., 33.),
                    Velocity::<f32>::new(-36., 55., 66.),
                    Mirrored(true),
                )
            })
            .run()
    }

    fn spawn_entity(world: &mut World, mirrored: bool) {
        let position_parent = Position::<f32>::new(1., 2., 3.);
        let velocity_parent = Velocity::<f32>::new(4., 5., 6.);
        let entity_parent = world
            .create_entity()
            .with(position_parent)
            .with(velocity_parent)
            .with(Mirrored(mirrored))
            .build();

        let entity_spawned = world.create_entity().build();
        world.add_resource(entity_spawned);

        let spawn = Spawn::new(
            AssetSlug::from_str("default/fireball")
                .expect("Expected `default/fireball` to be a valid asset slug."),
            Position::<i32>::new(10, 20, 30),
            Velocity::<i32>::new(40, 50, 60),
        );

        send_event(world, SpawnEvent::new(spawn, entity_parent, entity_spawned));
    }

    fn send_event(world: &mut World, spawn_event: SpawnEvent) {
        let mut ec = world.write_resource::<EventChannel<SpawnEvent>>();
        ec.single_write(spawn_event);
    } // kcov-ignore

    fn assert_spawn_values(
        world: &mut World,
        position: Position<f32>,
        velocity: Velocity<f32>,
        mirrored: Mirrored,
    ) {
        let entity_spawned = *world.read_resource::<Entity>();
        let positions = world.read_storage::<Position<f32>>();
        let velocities = world.read_storage::<Velocity<f32>>();
        let mirroreds = world.read_storage::<Mirrored>();

        let position_actual = positions
            .get(entity_spawned)
            .expect("Expected entity to have `Position<f32>` component.");
        let velocity_actual = velocities
            .get(entity_spawned)
            .expect("Expected entity to have `Velocity<f32>` component.");
        let mirrored_actual = mirroreds
            .get(entity_spawned)
            .expect("Expected entity to have `Mirrored` component.");
        assert_eq!(&position, position_actual);
        assert_eq!(&velocity, velocity_actual);
        assert_eq!(&mirrored, mirrored_actual);
    }
}