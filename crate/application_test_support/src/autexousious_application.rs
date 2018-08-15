use std::path::Path;

use amethyst::{
    animation::AnimationBundle, assets::ProgressCounter, core::transform::TransformBundle,
    prelude::*, renderer::SpriteRender,
};
use amethyst_test_support::prelude::*;
use application::resource::dir::ASSETS;
use character_selection::CharacterSelectionBundle;
use game_input::{PlayerActionControl, PlayerAxisControl};
use loading::AssetLoader;
use map_loading::MapLoadingBundle;
use object_loading::ObjectLoadingBundle;
use object_model::config::object::CharacterSequenceId;

// Copied from `amethyst_test_support`
type StatePlaceholder = EmptyState;
type FnStatePlaceholder = &'static fn() -> StatePlaceholder;
type FnEffectPlaceholder = &'static fn(&mut World);
type FnAssertPlaceholder = &'static fn(&mut World);

/// Baselines for building Amethyst applications with Autexousious types.
#[derive(Debug)]
pub struct AutexousiousApplication;

impl AutexousiousApplication {
    /// Returns an application with the Transform, Input, and UI bundles.
    ///
    /// This also adds a `ScreenDimensions` resource to the `World` so that UI calculations can be
    /// done.
    ///
    /// This has the same effect as calling `AmethystApplication::base::<PlayerAxisControl,
    /// PlayerActionControl>()`.
    pub fn ui_base() -> AmethystApplication<
        StatePlaceholder,
        GameData<'static, 'static>,
        FnStatePlaceholder,
        FnEffectPlaceholder,
        FnAssertPlaceholder,
    > {
        AmethystApplication::ui_base::<PlayerAxisControl, PlayerActionControl>()
    }

    /// Returns an application with the Animation, Transform, and Render bundles.
    ///
    /// The difference between this and `AmethystApplication::render_base()` is the type parameters
    /// to the Input and UI bundles are the `PlayerAxisControl` and `PlayerActionControl`, and the
    /// Animation bundle uses the object type sequence IDs for animation control sets.
    ///
    /// # Parameters
    ///
    /// * `test_name`: Name of the test, used to populate the window title.
    /// * `visibility`: Whether the window should be visible.
    pub fn render_base<'name, N>(
        test_name: N,
        visibility: bool,
    ) -> AmethystApplication<
        StatePlaceholder,
        GameData<'static, 'static>,
        FnStatePlaceholder,
        FnEffectPlaceholder,
        FnAssertPlaceholder,
    >
    where
        N: Into<&'name str>,
    {
        // Unfortunately we cannot re-use `AmethystApplication::render_base` because we need to
        // specify the `TransformBundle`'s dependencies.
        AmethystApplication::blank()
            .with_bundle(AnimationBundle::<CharacterSequenceId, SpriteRender>::new(
                "character_animation_control_system",
                "character_sampler_interpolation_system",
            )).with_bundle(TransformBundle::new().with_dep(&[
                "character_animation_control_system",
                "character_sampler_interpolation_system",
            ])).with_render_bundle(test_name, visibility)
    }

    /// Returns an application with the Animation, Transform, Input, UI, and Render bundles.
    ///
    /// This function does not load any game configuration as it is meant to be used to test types
    /// that load game configuration. If you want test objects and maps to be loaded, please use the
    /// `game_base` function.
    ///
    /// # Parameters
    ///
    /// * `test_name`: Name of the test, used to populate the window title.
    /// * `visibility`: Whether the window should be visible.
    pub fn object_base<'name, N>(
        test_name: N,
        visibility: bool,
    ) -> AmethystApplication<
        StatePlaceholder,
        GameData<'static, 'static>,
        FnStatePlaceholder,
        FnEffectPlaceholder,
        FnAssertPlaceholder,
    >
    where
        N: Into<&'name str>,
    {
        AutexousiousApplication::render_base(test_name, visibility)
            .with_ui_bundles::<PlayerAxisControl, PlayerActionControl>()
            .with_bundle(MapLoadingBundle::new())
            .with_bundle(ObjectLoadingBundle::new())
            .with_bundle(CharacterSelectionBundle::new())
    }

    /// Returns an application set up to load objects and maps.
    ///
    /// This function uses the setup state on `AmethystApplication`, so you cannot use the
    /// `.with_setup(F)` function. However, you can wrap any setup you still need to do with a
    /// `SequencerState` and `FunctionState`.
    ///
    /// # Parameters
    ///
    /// * `test_name`: Name of the test, used to populate the window title.
    /// * `visibility`: Whether the window should be visible.
    pub fn game_base<'name, N>(
        test_name: N,
        visibility: bool,
    ) -> AmethystApplication<
        StatePlaceholder,
        GameData<'static, 'static>,
        FnStatePlaceholder,
        FnEffectPlaceholder,
        FnAssertPlaceholder,
    >
    where
        N: Into<&'name str>,
    {
        AutexousiousApplication::object_base(test_name, visibility).with_setup(|world| {
            let mut progress_counter = ProgressCounter::new();
            AssetLoader::load_game_config(
                world,
                &Path::new(env!("CARGO_MANIFEST_DIR")).join(ASSETS),
                &mut progress_counter,
            );
            world.add_resource(progress_counter);
        })
    }
}

#[cfg(test)]
mod test {
    use amethyst::{assets::ProgressCounter, input::InputHandler, ui::MouseReactive};
    use amethyst_test_support::SpriteRenderAnimationFixture;
    use game_input::{PlayerActionControl, PlayerAxisControl};
    use map_model::loaded::MapHandle;
    use object_model::loaded::CharacterHandle;

    use super::AutexousiousApplication;

    // TODO: Allow users to specify their own type parameters to `AmethystApplication::base()`.
    //
    // This will make the dev experience better for crates that need strong types for the Input and
    // UI bundles, but are not able to depend on `AutexousiousApplication`, as the
    // `autexousious_test_support` crate would be depending on *that crate* (for better dev
    // experience for higher level crates).
    #[test]
    fn ui_base_uses_strong_types_for_input_and_ui_bundles() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::ui_base()
                .with_assertion(|world| {
                    // Panics if the type parameters used are not these ones.
                    world.read_resource::<InputHandler<PlayerAxisControl, PlayerActionControl>>();
                    world.read_storage::<MouseReactive>();
                }).run()
                .is_ok()
        );
    }

    #[test]
    fn render_base_application_can_load_sprite_render_animations() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::render_base(
                "render_base_application_can_load_sprite_render_animations",
                false
            ).with_effect(SpriteRenderAnimationFixture::effect)
            .with_assertion(SpriteRenderAnimationFixture::assertion)
            .run()
            .is_ok()
        );
    }

    #[test]
    fn object_base_uses_strong_types_for_input_and_ui_bundles() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::object_base(
                "object_base_uses_strong_types_for_input_and_ui_bundles",
                false
            ).with_assertion(|world| {
                // Panics if the type parameters used are not these ones.
                world.read_resource::<InputHandler<PlayerAxisControl, PlayerActionControl>>();
                world.read_storage::<MouseReactive>();
            }).run()
            .is_ok()
        );
    }

    #[test]
    fn game_base_loads_assets_from_self_crate_directory() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::game_base(
                "game_base_loads_assets_from_self_crate_directory",
                false
            ).with_assertion(|world| {
                let progress_counter = &*world.read_resource::<ProgressCounter>();
                assert!(progress_counter.is_complete());

                // Panics if the resources have not been populated
                world.read_resource::<Vec<MapHandle>>();
                assert!(!world.read_resource::<Vec<CharacterHandle>>().is_empty());
            }).run()
            .is_ok()
        );
    }
}
