use amethyst::{core::transform::Transform, ecs::Entity, renderer::transparent::Transparent};
use kinematic_model::config::{Position, Velocity};
use logic_clock::LogicClock;
use object_model::{loaded::ObjectWrapper, play::Mirrored};
use sequence_model::play::{FrameIndexClock, FrameWaitClock, SequenceStatus};

use crate::ObjectComponentStorages;

/// Placeholder constant for
const UNINITIALIZED: usize = 99;

/// Augments an entity with `Object` components.
#[derive(Debug)]
pub struct ObjectEntityAugmenter;

impl ObjectEntityAugmenter {
    /// Augments an entity with `Object` components.
    ///
    /// # Parameters
    ///
    /// * `entity`: The entity to augment.
    /// * `object_component_storages`: Non-frame-dependent `Component` storages for objects.
    /// * `object_wrapper`: Slug and handle of the object to spawn.
    pub fn augment<'s, W>(
        entity: Entity,
        ObjectComponentStorages {
            ref mut transparents,
            ref mut positions,
            ref mut velocities,
            ref mut transforms,
            ref mut mirroreds,
            ref mut sequence_end_transitionses,
            ref mut sequence_ids,
            ref mut sequence_statuses,
            ref mut frame_index_clocks,
            ref mut frame_wait_clocks,
        }: &mut ObjectComponentStorages<'s, W::SequenceId>,
        object_wrapper: &W,
    ) where
        W: ObjectWrapper,
    {
        let sequence_end_transitions = &object_wrapper.inner().sequence_end_transitions;

        let sequence_id = W::SequenceId::default();

        transparents
            .insert(entity, Transparent)
            .expect("Failed to insert transparent component.");
        positions
            .insert(entity, Position::default())
            .expect("Failed to insert position component.");
        velocities
            .insert(entity, Velocity::default())
            .expect("Failed to insert velocity component.");
        transforms
            .insert(entity, Transform::default())
            .expect("Failed to insert transform component.");
        mirroreds
            .insert(entity, Mirrored::default())
            .expect("Failed to insert mirrored component.");
        sequence_end_transitionses
            .insert(entity, sequence_end_transitions.clone())
            .expect("Failed to insert sequence_end_transitions component.");
        sequence_ids
            .insert(entity, sequence_id)
            .expect("Failed to insert sequence_id component.");
        sequence_statuses
            .insert(entity, SequenceStatus::default())
            .expect("Failed to insert sequence_status component.");
        frame_index_clocks
            .insert(entity, FrameIndexClock::new(LogicClock::new(UNINITIALIZED)))
            .expect("Failed to insert frame_index_clock component.");
        frame_wait_clocks
            .insert(entity, FrameWaitClock::new(LogicClock::new(UNINITIALIZED)))
            .expect("Failed to insert frame_wait_clock component.");
    }
}

#[cfg(test)]
mod tests {
    use amethyst::{
        core::transform::Transform,
        ecs::{Builder, SystemData, World},
        renderer::transparent::Transparent,
        Error,
    };
    use kinematic_model::config::{Position, Velocity};
    use object_model::play::Mirrored;
    use object_test::{ObjectBuilder, ObjectTest};
    use sequence_model::play::{FrameIndexClock, FrameWaitClock, SequenceStatus};
    use test_object_model::{
        config::TestObjectSequenceId,
        loaded::{TestObject, TestObjectObjectWrapper},
    };

    use super::ObjectEntityAugmenter;
    use crate::{FrameComponentStorages, ObjectComponentStorages};

    #[test]
    fn augments_entity_with_object_components() -> Result<(), Error> {
        let assert_components_augmented = |world: &mut World| {
            let entity = world.create_entity().build();
            {
                let object_wrapper = world.read_resource::<TestObjectObjectWrapper>();

                let mut object_component_storages = ObjectComponentStorages::fetch(&world.res);
                ObjectEntityAugmenter::augment(
                    entity,
                    &mut object_component_storages,
                    &*object_wrapper,
                );
            }

            assert!(world
                .read_storage::<TestObjectSequenceId>()
                .contains(entity));
            assert!(world.read_storage::<SequenceStatus>().contains(entity));
            assert!(world.read_storage::<Mirrored>().contains(entity));
            assert!(world.read_storage::<Transparent>().contains(entity));
            assert!(world.read_storage::<Position<f32>>().contains(entity));
            assert!(world.read_storage::<Velocity<f32>>().contains(entity));
            assert!(world.read_storage::<Transform>().contains(entity));
            assert!(world.read_storage::<FrameIndexClock>().contains(entity));
            assert!(world.read_storage::<FrameWaitClock>().contains(entity));
        };

        ObjectTest::application()
            .with_setup(|world| {
                <FrameComponentStorages as SystemData>::setup(&mut world.res);
                <ObjectComponentStorages<TestObjectSequenceId> as SystemData>::setup(
                    &mut world.res,
                );
            })
            .with_setup(setup_object_wrapper)
            .with_assertion(assert_components_augmented)
            .run_isolated()
    }

    fn setup_object_wrapper(world: &mut World) {
        let object_wrapper = ObjectBuilder::<TestObject>::new().build_wrapper(&*world);
        world.add_resource(object_wrapper);
    }
}
