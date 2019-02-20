// We cannot test `ObjectEntityAugmenter` directly in its own module due to Rust's behaviour in the
// following scenario:
//
// 1. Crate `a_crate` provides `AType<T>(pub T)`
// 2. Crate `b_crate` provides `struct BType(pub AType<BSub>)`
// 3. `a_crate` uses `b_crate` in `[dev-dependencies]`, because in order to test
//    `func_something(a_type: AType<T>)`, it needs a `T`, which happens to be `BSub` in the test.
// 4. in `a_crate` in a test function, `match a_type.0` gives the following error:
//
// Upon compilation:
//
// ```
// expected type a_crate::sub_module::AType<BSub>
//    found type          sub_module::AType<BSub>
// ```
//
// `a_crate`'s compilation is separate from `a_crate`'s tests, and in the test, using
// `crate::sub_module::AType` gives us a different *version* of `AType`.
//
// Paraphrased, `AType<BSub>` in the tests is different from `AType<BSub>` from `b_crate`.

use std::env;

use amethyst::{
    animation::AnimationBundle,
    assets::{AssetStorage, Prefab},
    core::transform::Transform,
    ecs::{Builder, SystemData, World},
    renderer::{Flipped, SpriteRender, Transparent},
    Error,
};
use amethyst_test::{AmethystApplication, PopState};
use application_event::{AppEvent, AppEventReader};
use asset_model::loaded::SlugAndHandle;
use assets_test::{ASSETS_CHAR_BAT_SLUG, ASSETS_PATH};
use character_loading::{CharacterLoadingBundle, CharacterPrefab};
use character_model::{
    config::CharacterSequenceId,
    loaded::{Character, CharacterObjectWrapper},
};
use collision_loading::CollisionLoadingBundle;
use collision_model::animation::{BodyFrameActiveHandle, InteractionFrameActiveHandle};
use loading::{LoadingBundle, LoadingState};
use map_loading::MapLoadingBundle;
use object_model::entity::{Mirrored, Position, SequenceStatus, Velocity};
use sprite_loading::SpriteLoadingBundle;

use object_loading::{
    ObjectAnimationStorages, ObjectComponentStorages, ObjectEntityAugmenter, ObjectPrefab,
};

#[test]
fn augments_entity_with_object_components() -> Result<(), Error> {
    env::set_var("APP_DIR", env!("CARGO_MANIFEST_DIR"));

    let assertion = |world: &mut World| {
        let entity = world.create_entity().build();
        {
            let mut object_animation_storages = ObjectAnimationStorages::fetch(&world.res);
            let mut object_component_storages = ObjectComponentStorages::fetch(&world.res);

            let object_wrapper_handle = {
                let slug_and_handle = SlugAndHandle::<Prefab<CharacterPrefab>>::from((
                    &*world,
                    ASSETS_CHAR_BAT_SLUG.clone(),
                ));

                let character_prefab_assets =
                    world.read_resource::<AssetStorage<Prefab<CharacterPrefab>>>();
                let character_prefab = character_prefab_assets
                    .get(&slug_and_handle.handle)
                    .expect("Expected bat character to be loaded.");
                let character_prefab = character_prefab
                    .entities()
                    .next()
                    .expect("Expected one main entity on character prefab.")
                    .data()
                    .expect("Expected character prefab data to be loaded.");
                let object_prefab: &ObjectPrefab<Character> = &character_prefab.object_prefab;

                match object_prefab {
                    ObjectPrefab::Handle(handle) => handle.clone(),
                    _ => panic!("Expected `ObjectPrefab` to be loaded for bat character."),
                }
            };
            let character_object_wrappers =
                world.read_resource::<AssetStorage<CharacterObjectWrapper>>();
            let object_wrapper = character_object_wrappers
                .get(&object_wrapper_handle)
                .expect("Expected `CharacterObjectWrapper` to be loaded for bat.");

            ObjectEntityAugmenter::augment(
                entity,
                &mut object_component_storages,
                &mut object_animation_storages,
                object_wrapper,
            );
        }

        assert!(world.read_storage::<CharacterSequenceId>().contains(entity));
        assert!(world.read_storage::<SequenceStatus>().contains(entity));
        assert!(world.read_storage::<Mirrored>().contains(entity));
        assert!(world.read_storage::<SpriteRender>().contains(entity));
        assert!(world.read_storage::<Flipped>().contains(entity));
        assert!(world.read_storage::<Transparent>().contains(entity));
        assert!(world.read_storage::<Position<f32>>().contains(entity));
        assert!(world.read_storage::<Velocity<f32>>().contains(entity));
        assert!(world.read_storage::<Transform>().contains(entity));
    };

    AmethystApplication::render_base("augments_entity_with_object_components", false)
        .with_custom_event_type::<AppEvent, AppEventReader>()
        .with_setup(|world| {
            <ObjectAnimationStorages<CharacterSequenceId> as SystemData>::setup(&mut world.res);
            <ObjectComponentStorages<CharacterSequenceId> as SystemData>::setup(&mut world.res);
        })
        .with_bundle(
            AnimationBundle::<CharacterSequenceId, BodyFrameActiveHandle>::new(
                "character_body_frame_acs",
                "character_body_frame_sis",
            ),
        )
        .with_bundle(AnimationBundle::<
            CharacterSequenceId,
            InteractionFrameActiveHandle,
        >::new(
            "character_interaction_acs", "character_interaction_sis"
        ))
        .with_bundle(SpriteLoadingBundle::new())
        .with_bundle(LoadingBundle::new(ASSETS_PATH.clone()))
        .with_bundle(CollisionLoadingBundle::new())
        .with_bundle(MapLoadingBundle::new())
        .with_bundle(CharacterLoadingBundle::new())
        .with_state(|| LoadingState::new(PopState))
        .with_assertion(assertion)
        .run()
}