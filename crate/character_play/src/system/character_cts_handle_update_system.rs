use amethyst::{
    ecs::{
        storage::ComponentEvent, BitSet, Entities, Join, Read, ReadStorage, ReaderId, System,
        World, WriteStorage,
    },
    shred::{ResourceId, SystemData},
};
use asset_model::loaded::AssetId;
use character_model::loaded::{AssetCharacterCtsHandles, CharacterCtsHandle};
use derivative::Derivative;
use derive_new::new;
use log::error;
use named_type::NamedType;
use named_type_derive::NamedType;
use sequence_model::loaded::SequenceId;

/// Updates the attached `CharacterCtsHandle`s when `SequenceId` changes.
#[derive(Debug, Default, NamedType, new)]
pub struct CharacterCtsHandleUpdateSystem {
    /// Reader ID for sequence ID changes.
    #[new(default)]
    sequence_id_rid: Option<ReaderId<ComponentEvent>>,
    /// Pre-allocated bitset to track insertions and modifications to `SequenceId`s.
    #[new(default)]
    sequence_id_updates: BitSet,
}

#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct CharacterCtsHandleUpdateSystemData<'s> {
    /// `Entities` resource.
    #[derivative(Debug = "ignore")]
    pub entities: Entities<'s>,
    /// `SequenceStatus` components.
    #[derivative(Debug = "ignore")]
    pub sequence_ids: ReadStorage<'s, SequenceId>,
    /// `AssetId` components.
    #[derivative(Debug = "ignore")]
    pub asset_ids: ReadStorage<'s, AssetId>,
    /// `AssetCharacterCtsHandles` resource.
    #[derivative(Debug = "ignore")]
    pub asset_character_cts_handles: Read<'s, AssetCharacterCtsHandles>,
    /// `CharacterCtsHandle` components.
    #[derivative(Debug = "ignore")]
    pub character_cts_handles: WriteStorage<'s, CharacterCtsHandle>,
}

impl<'s> System<'s> for CharacterCtsHandleUpdateSystem {
    type SystemData = CharacterCtsHandleUpdateSystemData<'s>;

    fn run(
        &mut self,
        CharacterCtsHandleUpdateSystemData {
            entities,
            sequence_ids,
            asset_ids,
            asset_character_cts_handles,
            mut character_cts_handles,
        }: Self::SystemData,
    ) {
        self.sequence_id_updates.clear();

        sequence_ids
            .channel()
            .read(
                self.sequence_id_rid
                    .as_mut()
                    .expect("Expected `sequence_id_rid` to be set."),
            )
            .for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self.sequence_id_updates.add(*id);
                }
                ComponentEvent::Removed(_id) => {} // kcov-ignore
            });

        (
            &entities,
            &sequence_ids,
            &asset_ids,
            &self.sequence_id_updates,
        )
            .join()
            .for_each(|(entity, sequence_id, asset_id, _)| {
                let cts_handles = asset_character_cts_handles
                    .get(*asset_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "Expected `CharacterCtsHandles` to exist for `{:?}`.",
                            asset_id
                        )
                    });

                let character_cts_handle = cts_handles
                    .get(**sequence_id)
                    // kcov-ignore-start
                    .unwrap_or_else(|| {
                        let message = format!(
                            "Expected `CharacterCtsHandle` to exist for \
                             sequence ID: `{:?}`. Falling back to default sequence.",
                            sequence_id
                        );
                        error!("{}", message);

                        let default_sequence_id = SequenceId::default();

                        cts_handles.get(*default_sequence_id).unwrap_or_else(|| {
                            let message = format!(
                                "Failed to get `CharacterCtsHandle` \
                                 for sequence ID: `{:?}`.",
                                default_sequence_id
                            );
                            error!("{}", message);
                            panic!(message);
                        })
                    })
                    // kcov-ignore-end
                    .clone();

                character_cts_handles
                    .insert(entity, character_cts_handle)
                    .expect("Failed to insert `CharacterCtsHandle` component.");
            });
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.sequence_id_rid = Some(WriteStorage::<SequenceId>::fetch(world).register_reader());
    }
}

#[cfg(test)]
mod tests {
    use amethyst::{
        ecs::{Builder, Entity, World, WorldExt},
        Error,
    };
    use application_test_support::{AssetQueries, AutexousiousApplication, SequenceQueries};
    use assets_test::CHAR_BAT_SLUG;
    use character_model::loaded::CharacterCtsHandle;
    use sequence_model::loaded::SequenceId;

    use super::CharacterCtsHandleUpdateSystem;

    #[test]
    fn attaches_handle_for_sequence_id_insertions() -> Result<(), Error> {
        AutexousiousApplication::game_base()
            .with_system(CharacterCtsHandleUpdateSystem::new(), "", &[])
            .with_effect(|world| insert_sequence(world, SequenceId::new(5)))
            .with_assertion(|world| expect_cts_handle(world, SequenceId::new(5)))
            .run_isolated()
    }

    #[test]
    fn attaches_handle_for_sequence_id_modifications() -> Result<(), Error> {
        AutexousiousApplication::game_base()
            .with_system(CharacterCtsHandleUpdateSystem::new(), "", &[])
            .with_effect(|world| update_sequence(world, SequenceId::new(5)))
            .with_assertion(|world| expect_cts_handle(world, SequenceId::new(5)))
            .run_isolated()
    }

    fn insert_sequence(world: &mut World, sequence_id: SequenceId) {
        let entity = create_entity(world);

        {
            let mut sequence_ids = world.write_storage::<SequenceId>();
            sequence_ids
                .insert(entity, sequence_id)
                .expect("Failed to insert `SequenceId`.");
        } // kcov-ignore

        world.insert(entity);
    }

    fn update_sequence(world: &mut World, sequence_id: SequenceId) {
        let entity = create_entity(world);

        {
            let mut sequence_ids = world.write_storage::<SequenceId>();
            let sid = sequence_ids
                .get_mut(entity)
                .expect("Expected entity to contain `SequenceId` component.");
            *sid = sequence_id;
        }

        world.insert(entity);
    }

    fn create_entity(world: &mut World) -> Entity {
        let asset_slug = CHAR_BAT_SLUG.clone();
        let asset_id = AssetQueries::id(world, &asset_slug);
        let character_cts_handle =
            SequenceQueries::character_cts_handle(world, &asset_slug, SequenceId::new(0));

        world
            .create_entity()
            .with(asset_id)
            .with(SequenceId::new(0))
            .with(character_cts_handle)
            .build()
    }

    fn expect_cts_handle(world: &mut World, sequence_id: SequenceId) {
        let entity = *world.read_resource::<Entity>();
        let expected_handle =
            SequenceQueries::character_cts_handle(world, &*CHAR_BAT_SLUG, sequence_id);
        let character_cts_handles = world.read_storage::<CharacterCtsHandle>();

        assert_eq!(
            &expected_handle,
            character_cts_handles
                .get(entity)
                .expect("Expected entity to contain `CharacterCtsHandle` component.")
        );
    }
}
