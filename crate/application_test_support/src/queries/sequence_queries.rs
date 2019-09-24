use amethyst::{
    assets::AssetStorage,
    ecs::{World, WorldExt},
};
use asset_model::config::AssetSlug;
use character_model::loaded::{
    AssetCharacterCtsHandles, CharacterControlTransitionsHandle, CharacterCts, CharacterCtsHandle,
    CharacterObjectWrapper,
};
use collision_model::loaded::{BodySequenceHandle, InteractionsSequenceHandle};
use object_model::loaded::ObjectWrapper;
use sequence_model::loaded::{SequenceEndTransition, SequenceId, WaitSequenceHandle};
use spawn_model::loaded::SpawnsSequenceHandle;
use sprite_model::loaded::SpriteRenderSequenceHandle;

use crate::{AssetQueries, ObjectQueries};

/// Functions to retrieve sequence data from a running world.
#[derive(Debug)]
pub struct SequenceQueries;

impl SequenceQueries {
    /// Returns the `CharacterCtsHandle` for the specified sequence ID.
    ///
    /// This function assumes the character for the specified slug is instantiated in the world.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` of the running application.
    /// * `asset_slug`: Object slug whose `Handle<O::ObjectWrapper>` to retrieve.
    /// * `sequence_id`: Sequence ID whose `CharacterCtsHandle` to retrieve.
    pub fn character_cts_handle(
        world: &World,
        asset_slug: &AssetSlug,
        sequence_id: SequenceId,
    ) -> CharacterCtsHandle {
        let character_asset_id = AssetQueries::id(world, &asset_slug);
        let asset_character_cts_handles = world.read_resource::<AssetCharacterCtsHandles>();
        let character_cts_handles = asset_character_cts_handles
            .get(character_asset_id)
            .unwrap_or_else(|| {
                panic!(
                    "Expected `CharacterCtsHandles` to exist for `{:?}`.",
                    character_asset_id
                )
            });

        character_cts_handles
            .get(*sequence_id)
            .unwrap_or_else(|| {
                panic!(
                    "Expected `CharacterCtsHandle` to exist for sequence ID: `{:?}`.",
                    sequence_id
                )
            })
            .clone()
    }

    /// Returns the `CharacterControlTransitionsHandle` for the specified sequence ID.
    ///
    /// This function assumes the character for the specified slug is instantiated in the world.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` of the running application.
    /// * `asset_slug`: Object slug whose `Handle<O::ObjectWrapper>` to retrieve.
    /// * `sequence_id`: Sequence ID whose `CharacterCtsHandle` to retrieve.
    /// * `frame_index`: Frame index within the sequence whose control transitions to retrieve.
    pub fn character_control_transitions_handle(
        world: &World,
        asset_slug: &AssetSlug,
        sequence_id: SequenceId,
        frame_index: usize,
    ) -> CharacterControlTransitionsHandle {
        let character_cts_handle = Self::character_cts_handle(world, asset_slug, sequence_id);

        let character_cts_assets = world.read_resource::<AssetStorage<CharacterCts>>();
        let character_cts = character_cts_assets
            .get(&character_cts_handle)
            .expect("Expected `CharacterCts` to be loaded.");

        character_cts[frame_index].clone()
    }

    /// Returns the `SequenceEndTransition` for the specified sequence ID.
    ///
    /// This function assumes the character for the specified slug is instantiated in the world.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` of the running application.
    /// * `asset_slug`: Asset slug of the `Object`.
    /// * `sequence_id`: Sequence ID of the `SequenceEndTransition`.
    pub fn sequence_end_transition(
        world: &World,
        asset_slug: &AssetSlug,
        sequence_id: SequenceId,
    ) -> SequenceEndTransition {
        let object_wrapper_handle = ObjectQueries::object_wrapper_handle(world, asset_slug);
        let object_wrapper_assets = world.read_resource::<AssetStorage<CharacterObjectWrapper>>();
        let object_wrapper = object_wrapper_assets
            .get(&object_wrapper_handle)
            .unwrap_or_else(|| panic!("Expected `{}` object wrapper to be loaded.", asset_slug));

        let object = object_wrapper.inner();
        object
            .sequence_end_transitions
            .get(*sequence_id)
            .copied()
            .unwrap_or_else(|| {
                panic!(
                    "Expected `SequenceEndTransition` for sequence ID `{:?}` to exist.",
                    sequence_id
                )
            })
    }

    /// Returns the `WaitSequenceHandle` for the specified sequence ID.
    ///
    /// This function assumes the character for the specified slug is instantiated in the world.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` of the running application.
    /// * `asset_slug`: Asset slug of the `Object`.
    /// * `sequence_id`: Sequence ID of the `WaitSequenceHandle`.
    pub fn wait_sequence_handle(
        world: &World,
        asset_slug: &AssetSlug,
        sequence_id: SequenceId,
    ) -> WaitSequenceHandle {
        let object_wrapper_handle = ObjectQueries::object_wrapper_handle(world, asset_slug);
        let object_wrapper_assets = world.read_resource::<AssetStorage<CharacterObjectWrapper>>();
        let object_wrapper = object_wrapper_assets
            .get(&object_wrapper_handle)
            .unwrap_or_else(|| panic!("Expected `{}` object wrapper to be loaded.", asset_slug));

        let object = object_wrapper.inner();
        object
            .wait_sequence_handles
            .get(*sequence_id)
            .unwrap_or_else(|| {
                panic!(
                    "Expected `WaitSequenceHandle` for sequence ID `{:?}` to exist.",
                    sequence_id
                )
            })
            .clone()
    }

    /// Returns the `SpriteRenderSequenceHandle` for the specified sequence ID.
    ///
    /// This function assumes the character for the specified slug is instantiated in the world.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` of the running application.
    /// * `asset_slug`: Asset slug of the `Object`.
    /// * `sequence_id`: Sequence ID of the `SpriteRenderSequenceHandle`.
    pub fn sprite_render_sequence_handle(
        world: &World,
        asset_slug: &AssetSlug,
        sequence_id: SequenceId,
    ) -> SpriteRenderSequenceHandle {
        let object_wrapper_handle = ObjectQueries::object_wrapper_handle(world, asset_slug);
        let object_wrapper_assets = world.read_resource::<AssetStorage<CharacterObjectWrapper>>();
        let object_wrapper = object_wrapper_assets
            .get(&object_wrapper_handle)
            .unwrap_or_else(|| panic!("Expected `{}` object wrapper to be loaded.", asset_slug));

        let object = object_wrapper.inner();
        object
            .sprite_render_sequence_handles
            .get(*sequence_id)
            .unwrap_or_else(|| {
                panic!(
                    "Expected `SpriteRenderSequenceHandle` for sequence ID `{:?}` to exist.",
                    sequence_id
                )
            })
            .clone()
    }

    /// Returns the `BodySequenceHandle` for the specified sequence ID.
    ///
    /// This function assumes the character for the specified slug is instantiated in the world.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` of the running application.
    /// * `asset_slug`: Asset slug of the `Object`.
    /// * `sequence_id`: Sequence ID of the `BodySequenceHandle`.
    pub fn body_sequence_handle(
        world: &World,
        asset_slug: &AssetSlug,
        sequence_id: SequenceId,
    ) -> BodySequenceHandle {
        let object_wrapper_handle = ObjectQueries::object_wrapper_handle(world, asset_slug);
        let object_wrapper_assets = world.read_resource::<AssetStorage<CharacterObjectWrapper>>();
        let object_wrapper = object_wrapper_assets
            .get(&object_wrapper_handle)
            .unwrap_or_else(|| panic!("Expected `{}` object wrapper to be loaded.", asset_slug));

        let object = object_wrapper.inner();
        object
            .body_sequence_handles
            .get(*sequence_id)
            .unwrap_or_else(|| {
                panic!(
                    "Expected `BodySequenceHandle` for sequence ID `{:?}` to exist.",
                    sequence_id
                )
            })
            .clone()
    }

    /// Returns the `InteractionsSequenceHandle` for the specified sequence ID.
    ///
    /// This function assumes the character for the specified slug is instantiated in the world.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` of the running application.
    /// * `asset_slug`: Asset slug of the `Object`.
    /// * `sequence_id`: Sequence ID of the `InteractionsSequenceHandle`.
    pub fn interactions_sequence_handle(
        world: &World,
        asset_slug: &AssetSlug,
        sequence_id: SequenceId,
    ) -> InteractionsSequenceHandle {
        let object_wrapper_handle = ObjectQueries::object_wrapper_handle(world, asset_slug);
        let object_wrapper_assets = world.read_resource::<AssetStorage<CharacterObjectWrapper>>();
        let object_wrapper = object_wrapper_assets
            .get(&object_wrapper_handle)
            .unwrap_or_else(|| panic!("Expected `{}` object wrapper to be loaded.", asset_slug));

        let object = object_wrapper.inner();
        object
            .interactions_sequence_handles
            .get(*sequence_id)
            .unwrap_or_else(|| {
                panic!(
                    "Expected `InteractionsSequenceHandle` for sequence ID `{:?}` to exist.",
                    sequence_id
                )
            })
            .clone()
    }

    /// Returns the `SpawnsSequenceHandle` for the specified sequence ID.
    ///
    /// This function assumes the character for the specified slug is instantiated in the world.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` of the running application.
    /// * `asset_slug`: Asset slug of the `Object`.
    /// * `sequence_id`: Sequence ID of the `SpawnsSequenceHandle`.
    pub fn spawns_sequence_handle(
        world: &World,
        asset_slug: &AssetSlug,
        sequence_id: SequenceId,
    ) -> SpawnsSequenceHandle {
        let object_wrapper_handle = ObjectQueries::object_wrapper_handle(world, asset_slug);
        let object_wrapper_assets = world.read_resource::<AssetStorage<CharacterObjectWrapper>>();
        let object_wrapper = object_wrapper_assets
            .get(&object_wrapper_handle)
            .unwrap_or_else(|| panic!("Expected `{}` object wrapper to be loaded.", asset_slug));

        let object = object_wrapper.inner();
        object
            .spawns_sequence_handles
            .get(*sequence_id)
            .unwrap_or_else(|| {
                panic!(
                    "Expected `SpawnsSequenceHandle` for sequence ID `{:?}` to exist.",
                    sequence_id
                )
            })
            .clone()
    }
}
