use std::marker::PhantomData;
use std::sync::Mutex;
use std::thread;

use amethyst::{
    animation::AnimationBundle,
    core::{transform::TransformBundle, SystemBundle},
    ecs::prelude::*,
    input::InputBundle,
    prelude::*,
    renderer::{
        ColorMask, DisplayConfig, DrawFlat, Material, Pipeline, PosTex, RenderBundle,
        ScreenDimensions, Stage, ALPHA,
    },
    shred::Resource,
    ui::{DrawUi, UiBundle},
    Result,
};
use boxfnonce::SendBoxFnOnce;

use AssertionState;
use EffectState;
use EmptyState;
use SystemInjectionBundle;

type BundleAddFn = SendBoxFnOnce<
    'static,
    (GameDataBuilder<'static, 'static>,),
    Result<GameDataBuilder<'static, 'static>>,
>;
// Hack: Ideally we want a `SendBoxFnOnce`. However implementing it got too crazy:
//
// * When taking in `ApplicationBuilder<SLocal>` as a parameter, I couldn't get the type parameters
//   to be happy. `SLocal` had to change depending on the first state, but it couldn't be
//   consolidated with `T`.
// * When using `SendBoxFnOnce<'w, (&'w mut World,)>`, the lifetime parameter for the function and
//   the `World` could not agree &mdash; you can't coerce a `SendBoxFnOnce<'longer>` into a
//   `SendBoxFnOnce<'shorter>`, which was necessary to indicate the length of the borrow of `World`
//   for the function is not the `'w` needed to store the function in `AmethystApplication`.
//   In addition, it requires the `World` (and hence the `ApplicationBuilder`) to be instantiated
//   in a scope greater than the `AmethystApplication`'s lifetime, which detracts from the
//   ergonomics of this test harness.
type FnResourceAdd = Box<FnMut(&mut World) + Send>;

// Hacks for ergonomics so users don't have to specify the type parameter if they don't specify an
// assertion function such as `AmethystApplication::<fn(&mut World)>`.
//
// See <https://stackoverflow.com/questions/37310941/default-generic-parameter>
type StatePlaceholder = EmptyState;
type FnStatePlaceholder = &'static fn() -> StatePlaceholder;
type FnEffectPlaceholder = &'static fn(&mut World);
type FnAssertPlaceholder = &'static fn(&mut World);

// Use a mutex to prevent multiple tests that open GL windows from running simultaneously, due to
// race conditions causing failures in X.
// <https://github.com/tomaka/glutin/issues/1038>
lazy_static! {
    static ref X11_GL_MUTEX: Mutex<()> = Mutex::new(());
}

/// Builder for an Amethyst application.
///
/// This provides varying levels of setup so that users do not have to register common bundles.
#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct AmethystApplication<S, T, FnState, FnEffect, FnAssert>
where
    S: State<T>,
    FnState: Fn() -> S + Send,
    FnEffect: Fn(&mut World) + Send,
    FnAssert: Fn(&mut World) + Send,
{
    /// Functions to add bundles to the game data.
    ///
    /// This is necessary because `System`s are not `Send`, and so we cannot send `GameDataBuilder`
    /// across a thread boundary, necessary to run the `Application` in a sub thread to avoid a
    /// segfault caused by mesa and the software GL renderer.
    #[derivative(Debug = "ignore")]
    bundle_add_fns: Vec<BundleAddFn>,
    /// Functions to add bundles to the game data.
    ///
    /// This is necessary because `System`s are not `Send`, and so we cannot send `GameDataBuilder`
    /// across a thread boundary, necessary to run the `Application` in a sub thread to avoid a
    /// segfault caused by mesa and the software GL renderer.
    #[derivative(Debug = "ignore")]
    resource_add_fns: Vec<FnResourceAdd>,
    /// Function to create user specified state to use for the application.
    first_state_fn: Option<FnState>,
    /// Effect function to run.
    effect_fn: Option<FnEffect>,
    /// Assertion function to run.
    assertion_fn: Option<FnAssert>,
    /// State data.
    state_data: PhantomData<T>,
    /// Whether or not this application uses the `RenderBundle`.
    render: bool,
}

impl
    AmethystApplication<
        StatePlaceholder,
        GameData<'static, 'static>,
        FnStatePlaceholder,
        FnEffectPlaceholder,
        FnAssertPlaceholder,
    >
{
    /// Returns an Amethyst application without any bundles.
    pub fn blank() -> AmethystApplication<
        StatePlaceholder,
        GameData<'static, 'static>,
        FnStatePlaceholder,
        FnEffectPlaceholder,
        FnAssertPlaceholder,
    > {
        AmethystApplication {
            bundle_add_fns: Vec::new(),
            resource_add_fns: Vec::new(),
            first_state_fn: None,
            effect_fn: None,
            assertion_fn: None,
            state_data: PhantomData,
            render: false,
        }
    }

    /// Returns an application with the Transform, Input, and UI bundles.
    ///
    /// This also adds a `ScreenDimensions` resource to the `World` so that UI calculations can be
    /// done.
    pub fn base() -> AmethystApplication<
        StatePlaceholder,
        GameData<'static, 'static>,
        FnStatePlaceholder,
        FnEffectPlaceholder,
        FnAssertPlaceholder,
    > {
        AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(InputBundle::<String, String>::new())
            .with_bundle(UiBundle::<String, String>::new())
            .with_resource(ScreenDimensions::new(1280, 800, 1.))
    }

    /// Returns an application with the Animation, Transform, Input, UI, and Render bundles.
    ///
    /// **Note:** The type parameters for the Animation, Input, and UI bundles are [stringly-typed]
    /// (http://wiki.c2.com/?StringlyTyped). It is recommended that you use proper type parameters
    /// and register the bundles yourself if the unit you are testing uses them.
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
        // TODO: We can default to the function name once this RFC is implemented:
        // <https://github.com/rust-lang/rfcs/issues/1743>
        // <https://github.com/rust-lang/rfcs/pull/1719>
        let title = test_name.into().to_string();
        let render_bundle_fn = move || {
            let display_config = DisplayConfig {
                title,
                fullscreen: false,
                dimensions: Some((800, 600)),
                min_dimensions: Some((400, 300)),
                max_dimensions: None,
                vsync: false,
                multisampling: 0, // Must be multiple of 2, use 0 to disable
                visibility,
            };
            let pipe = Pipeline::build().with_stage(
                Stage::with_backbuffer()
                    .clear_target([0., 0., 0., 0.], 1.)
                    .with_pass(DrawFlat::<PosTex>::new().with_transparency(
                        ColorMask::all(),
                        ALPHA,
                        None,
                    ))
                    .with_pass(DrawUi::new()),
            );
            RenderBundle::new(pipe, Some(display_config))
        };

        AmethystApplication::blank()
            .with_bundle(AnimationBundle::<u32, Material>::new(
                "animation_control_system",
                "sampler_interpolation_system",
            ))
            .with_bundle(
                TransformBundle::new()
                    .with_dep(&["animation_control_system", "sampler_interpolation_system"]),
            )
            .with_bundle(InputBundle::<String, String>::new())
            .with_bundle(UiBundle::<String, String>::new())
            .with_bundle_fn(render_bundle_fn)
            .mark_render()
    }
}

impl<S, FnState, FnEffect, FnAssert>
    AmethystApplication<S, GameData<'static, 'static>, FnState, FnEffect, FnAssert>
where
    S: State<GameData<'static, 'static>> + 'static,
    FnState: Fn() -> S + Send + 'static,
    FnEffect: Fn(&mut World) + Send + 'static,
    FnAssert: Fn(&mut World) + Send + 'static,
{
    /// Returns the built Application.
    ///
    /// If you are intending to call `.run()` on the `Application` in a test, be aware that on
    /// Linux, this will cause a segfault when `RenderBundle` is added and GL is using software
    /// rendering, such as when using Xvfb or when the following environmental variable is set:
    /// `LIBGL_ALWAYS_SOFTWARE=1`.
    ///
    /// To avoid this, please call `.run()` instead of this method, which runs the application in a
    /// separate thread and waits for it to end before returning.
    ///
    /// See <https://users.rust-lang.org/t/trouble-identifying-cause-of-segfault/18096>
    pub fn build(self) -> Result<Application<'static, GameData<'static, 'static>>> {
        let params = (
            self.bundle_add_fns,
            self.resource_add_fns,
            self.first_state_fn,
            self.effect_fn,
            self.assertion_fn,
        );
        Self::build_internal(params)
    }

    // Hack to get around `S` or `T` not being `Send`
    // We take a function that constructs `S`, and the function itself is `Send`.
    // However, `Self` has `PhantomData<T>`, which means we cannot send `self` to a thread. Instead
    // we have to take all of the other fields and send those through.
    //
    // Need to `#[allow(type_complexity)]` because the type declaration would have unused type
    // parameters which causes a compilation failure.
    #[allow(unknown_lints, type_complexity)]
    fn build_internal(
        (bundle_add_fns, resource_add_fns, first_state_fn, effect_fn, assertion_fn): (
            Vec<BundleAddFn>,
            Vec<FnResourceAdd>,
            Option<FnState>,
            Option<FnEffect>,
            Option<FnAssert>,
        ),
    ) -> Result<Application<'static, GameData<'static, 'static>>> {
        let game_data = bundle_add_fns.into_iter().fold(
            Ok(GameDataBuilder::default()),
            |game_data: Result<GameDataBuilder>, function: BundleAddFn| {
                game_data.and_then(|game_data| function.call(game_data))
            },
        )?;

        // eww
        if assertion_fn.is_some() {
            let assertion_state = AssertionState::new(assertion_fn.unwrap());
            if first_state_fn.is_some() {
                let first_state = first_state_fn.unwrap()();
                if effect_fn.is_some() {
                    let effect_state = EffectState::new(effect_fn.unwrap(), assertion_state)
                        .with_stack_state(first_state);
                    Self::build_application(effect_state, game_data, resource_add_fns)
                } else {
                    let assertion_state = assertion_state.with_stack_state(first_state);
                    Self::build_application(assertion_state, game_data, resource_add_fns)
                }
            } else if effect_fn.is_some() {
                let first_state = EffectState::new(effect_fn.unwrap(), assertion_state);
                Self::build_application(first_state, game_data, resource_add_fns)
            } else {
                Self::build_application(assertion_state, game_data, resource_add_fns)
            }
        } else if let Some(first_state_fn) = first_state_fn {
            let first_state = first_state_fn();
            if effect_fn.is_some() {
                // There's a first state and an effect function, but no assertion function.
                // Perhaps we should warn the user that assertions should be registered using
                // `.with_assertion(F)`.
                let effect_state =
                    EffectState::new(effect_fn.unwrap(), EmptyState).with_stack_state(first_state);
                Self::build_application(effect_state, game_data, resource_add_fns)
            } else {
                Self::build_application(first_state, game_data, resource_add_fns)
            }
        } else {
            Self::build_application(EmptyState, game_data, resource_add_fns)
        }
    }

    fn build_application<SLocal>(
        first_state: SLocal,
        game_data: GameDataBuilder<'static, 'static>,
        resource_add_fns: Vec<FnResourceAdd>,
    ) -> Result<Application<'static, GameData<'static, 'static>>>
    where
        SLocal: State<GameData<'static, 'static>> + 'static,
    {
        let assets_dir = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
        let mut application_builder = Application::build(assets_dir, first_state)?;
        {
            let world = &mut application_builder.world;
            for mut function in resource_add_fns {
                function(world);
            }
        }
        application_builder.build(game_data)
    }

    /// Runs the application and returns `Ok(())` if nothing went wrong.
    ///
    /// This method should be called instead of the `.build()` method if the application is to be
    /// run, as this avoids a segfault on Linux when using the GL software renderer.
    pub fn run(self) -> Result<()> {
        let params = (
            self.bundle_add_fns,
            self.resource_add_fns,
            self.first_state_fn,
            self.effect_fn,
            self.assertion_fn,
        );

        let render = self.render;

        // Run in a sub thread due to mesa's threading issues with GL software rendering
        // See: <https://users.rust-lang.org/t/trouble-identifying-cause-of-segfault/18096>
        thread::spawn(move || -> Result<()> {
            if render {
                let guard = X11_GL_MUTEX.lock().unwrap();

                // Note: if this panics, the Mutex is poisoned.
                // Unfortunately we cannot catch panics, as the application is `!UnwindSafe`
                //
                // We have to build the application after acquiring the lock because the window is
                // already instantiated during the build.
                //
                // The mutex greatly reduces, but does not eliminate X11 window initialization
                // errors from happening:
                //
                // * <https://github.com/tomaka/glutin/issues/1034> can still happen
                // * <https://github.com/tomaka/glutin/issues/1038> may be completely removed
                Self::build_internal(params)?.run();

                drop(guard);
            } else {
                Self::build_internal(params)?.run();
            }

            Ok(())
        }).join()
            .expect("Failed to run Amethyst application")
    }
}

impl<S, T, FnState, FnEffect, FnAssert> AmethystApplication<S, T, FnState, FnEffect, FnAssert>
where
    S: State<T> + 'static,
    FnState: Fn() -> S + Send + 'static,
    FnEffect: Fn(&mut World) + Send + 'static,
    FnAssert: Fn(&mut World) + Send + 'static,
{
    /// Adds a bundle to the list of bundles.
    ///
    /// **Note:** If you are adding the `RenderBundle`, you need to use `.with_bundle_fn(F)` as the
    /// `Pipeline` type used by the bundle is `!Send`. Furthermore, you must also invoke
    /// `.mark_render()` to avoid a race condition that causes render tests to fail.
    ///
    /// # Parameters
    ///
    /// * `bundle`: Bundle to add.
    pub fn with_bundle<B>(mut self, bundle: B) -> Self
    where
        B: SystemBundle<'static, 'static> + Send + 'static,
    {
        // We need to use `SendBoxFnOnce` because:
        //
        // * `FnOnce` takes itself by value when you call it.
        // * To pass a `FnOnce` around (transferring ownership), it must be boxed, since it's not
        //   `Sized`.
        // * A `Box<FnOnce()>` is a `Sized` type with a reference to the `FnOnce`
        // * To call the function inside the `Box<FnOnce()>`, it must be moved out of the box
        //   because we need to own the `FnOnce` to be able to call it by value, whereas the `Box`
        //   only holds the reference.
        // * To own it, we would have to move it onto the stack.
        // * However, since it's not `Sized`, we can't do that.
        //
        // To make this work, we can implement a trait for `FnOnce` with a trait function which
        // takes `Box<Self>` and can invoke the `FnOnce` whilst inside the Box.
        // `SendBoxFnOnce` is an implementation this.
        //
        // See <https://users.rust-lang.org/t/move-a-boxed-function-inside-a-closure/18199>
        self.bundle_add_fns.push(SendBoxFnOnce::from(
            |game_data: GameDataBuilder<'static, 'static>| game_data.with_bundle(bundle),
        ));
        self
    }

    /// Adds a bundle to the list of bundles.
    ///
    /// This provides an alternative to `.with_bundle(B)` where `B` is `!Send`. The function that
    /// instantiates the bundle must be `Send`.
    ///
    /// **Note:** If you are adding the `RenderBundle`, you must also invoke `.mark_render()` to
    /// avoid a race condition that causes render tests to fail.
    ///
    /// # Parameters
    ///
    /// * `bundle_function`: Function to instantiate the Bundle.
    pub fn with_bundle_fn<FnBundle, B>(mut self, bundle_function: FnBundle) -> Self
    where
        FnBundle: FnOnce() -> B + Send + 'static,
        B: SystemBundle<'static, 'static> + 'static,
    {
        self.bundle_add_fns.push(SendBoxFnOnce::from(
            move |game_data: GameDataBuilder<'static, 'static>| {
                game_data.with_bundle(bundle_function())
            },
        ));
        self
    }

    /// Adds a resource to the `World`.
    ///
    /// # Parameters
    ///
    /// * `resource`: Bundle to add.
    pub fn with_resource<R>(mut self, resource: R) -> Self
    where
        R: Resource,
    {
        let mut resource_opt = Some(resource);
        self.resource_add_fns
            .push(Box::new(move |world: &mut World| {
                let resource = resource_opt.take();
                if resource.is_some() {
                    world.add_resource(resource.unwrap());
                }
            }));
        self
    }

    /// Sets the state for the Amethyst application.
    ///
    /// **NOTE:** This must be called before any calls to `.with_resource()`, as complexities with
    /// type parameters disallows us to store earlier resource registrations.
    ///
    /// **NOTE:** This function is exclusive of `.with_effect()`, as each of them are ways to test
    /// an effect before an assertion.
    ///
    /// # Parameters
    ///
    /// * `state`: `State` to use.
    pub fn with_state<SLocal, TLocal, FnStateLocal>(
        self,
        state: FnStateLocal,
    ) -> AmethystApplication<SLocal, TLocal, FnStateLocal, FnEffect, FnAssert>
    where
        SLocal: State<TLocal>,
        FnStateLocal: Fn() -> SLocal + Send,
    {
        if self.first_state_fn.is_some() {
            panic!(
                "`.with_state(S)` has previously been called. The current implementation only \
                 supports one starting state."
            );
        } else if !self.resource_add_fns.is_empty() {
            panic!(
                "`.with_state(S)` called after `.with_resource(R)`. Due to restrictions on type \
                 parameter specification, you must register resources after `.with_state(S)`."
            );
        } else {
            AmethystApplication {
                bundle_add_fns: self.bundle_add_fns,
                resource_add_fns: Vec::new(),
                first_state_fn: Some(state),
                effect_fn: self.effect_fn,
                assertion_fn: self.assertion_fn,
                state_data: PhantomData,
                render: self.render,
            }
        }
    }

    /// Registers a `System` to be tested in this application.
    ///
    /// # Parameters
    ///
    /// * `system`: The `System` to be tested.
    pub fn with_system<SysLocal>(
        self,
        system: SysLocal,
        name: &'static str,
        deps: &'static [&'static str],
    ) -> AmethystApplication<S, T, FnState, FnEffect, FnAssert>
    where
        SysLocal: for<'sys_local> System<'sys_local> + Send + 'static,
    {
        self.with_bundle_fn(move || SystemInjectionBundle::new(system, name, deps))
    }

    /// Registers a function to assert an expected outcome.
    ///
    /// The function will be run in an [`AssertionState`](struct.AssertionState.html)
    ///
    /// # Parameters
    ///
    /// * `assertion_fn`: Function that asserts the expected state.
    pub fn with_effect<FnEffectLocal>(
        self,
        effect_fn: FnEffectLocal,
    ) -> AmethystApplication<S, T, FnState, FnEffectLocal, FnAssert>
    where
        FnEffectLocal: Fn(&mut World) + Send,
    {
        if self.effect_fn.is_some() {
            panic!(
                "`.with_effect(F)` has previously been called. The current implementation only \
                 supports one effect function."
            );
        } else {
            AmethystApplication {
                bundle_add_fns: self.bundle_add_fns,
                resource_add_fns: self.resource_add_fns,
                first_state_fn: self.first_state_fn,
                effect_fn: Some(effect_fn),
                assertion_fn: self.assertion_fn,
                state_data: self.state_data,
                render: self.render,
            }
        }
    }

    /// Registers a function to assert an expected outcome.
    ///
    /// The function will be run in an [`AssertionState`](struct.AssertionState.html)
    ///
    /// # Parameters
    ///
    /// * `assertion_fn`: Function that asserts the expected state.
    pub fn with_assertion<FnAssertLocal>(
        self,
        assertion_fn: FnAssertLocal,
    ) -> AmethystApplication<S, T, FnState, FnEffect, FnAssertLocal>
    where
        FnAssertLocal: Fn(&mut World) + Send,
    {
        if self.assertion_fn.is_some() {
            panic!(
                "`.with_assertion(F)` has previously been called. The current implementation only \
                 supports one assertion function."
            );
        } else {
            AmethystApplication {
                bundle_add_fns: self.bundle_add_fns,
                resource_add_fns: self.resource_add_fns,
                first_state_fn: self.first_state_fn,
                effect_fn: self.effect_fn,
                assertion_fn: Some(assertion_fn),
                state_data: self.state_data,
                render: self.render,
            }
        }
    }

    /// Marks that this application uses the `RenderBundle`.
    ///
    /// This is used to avoid a window initialization race condition that causes tests to fail.
    /// See <https://github.com/tomaka/glutin/issues/1038>.
    pub fn mark_render(mut self) -> Self {
        self.render = true;
        self
    }
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use amethyst::{
        self,
        animation::{
            Animation, InterpolationFunction, MaterialChannel, MaterialPrimitive, Sampler,
        },
        assets::{self, Asset, AssetStorage, Handle, Loader, ProcessingState, Processor},
        core::bundle::{self, SystemBundle},
        ecs::prelude::*,
        prelude::*,
        renderer::{Material, ScreenDimensions},
        ui::FontAsset,
    };

    use super::AmethystApplication;
    use AssertionState;
    use EffectReturn;
    use EmptyState;

    #[test]
    fn bundle_build_is_ok() {
        assert!(
            AmethystApplication::blank()
                .with_bundle(BundleZero)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn load_multiple_bundles() {
        assert!(
            AmethystApplication::blank()
                .with_bundle(BundleZero)
                .with_bundle(BundleOne)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn assertion_when_resource_is_added_succeeds() {
        let assertion_fn = |world: &mut World| {
            world.read_resource::<ApplicationResource>();
            world.read_resource::<ApplicationResourceNonDefault>();
        };
        assert!(
            AmethystApplication::blank()
                .with_bundle(BundleZero)
                .with_bundle(BundleOne)
                .with_assertion(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    #[should_panic(expected = "Failed to run Amethyst application")]
    fn assertion_when_resource_is_not_added_should_panic() {
        let assertion_fn = |world: &mut World| {
            // Panics if `ApplicationResource` was not added.
            world.read_resource::<ApplicationResource>();
        };

        assert!(
            AmethystApplication::blank()
                    // without BundleOne
                    .with_assertion(assertion_fn)
                    .run()
                    .is_ok()
        );
    }

    #[test]
    fn assertion_switch_with_loading_state_with_add_resource_succeeds() {
        let first_state_fn = || {
            let assertion_fn = |world: &mut World| {
                world.read_resource::<LoadResource>();
            };

            // Necessary if the State being tested is a loading state that returns `Trans::Switch`
            let assertion_state = AssertionState::new(assertion_fn);
            LoadingState::new(assertion_state)
        };

        assert!(
            AmethystApplication::blank()
                .with_state(first_state_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn assertion_push_with_loading_state_with_add_resource_succeeds() {
        // Alternative to embedding the `AssertionState` is to switch to an `EmptyState` but still
        // provide the assertion function
        let first_state_fn = || LoadingState::new(EmptyState);
        let assertion_fn = |world: &mut World| {
            world.read_resource::<LoadResource>();
        };

        assert!(
            AmethystApplication::blank()
                .with_state(first_state_fn)
                .with_assertion(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    #[should_panic(expected = "Failed to run Amethyst application")]
    fn assertion_switch_with_loading_state_without_add_resource_should_panic() {
        let first_state_fn = || {
            let assertion_fn = |world: &mut World| {
                world.read_resource::<LoadResource>();
            };

            SwitchState::new(AssertionState::new(assertion_fn))
        };

        assert!(
            AmethystApplication::blank()
                .with_state(first_state_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    #[should_panic(expected = "Failed to run Amethyst application")]
    fn assertion_push_with_loading_state_without_add_resource_should_panic() {
        // Alternative to embedding the `AssertionState` is to switch to an `EmptyState` but still
        // provide the assertion function
        let first_state_fn = || SwitchState::new(EmptyState);
        let assertion_fn = |world: &mut World| {
            world.read_resource::<LoadResource>();
        };

        assert!(
            AmethystApplication::blank()
                .with_state(first_state_fn)
                .with_assertion(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn game_data_must_update_before_assertion() {
        let effect_fn = |world: &mut World| {
            let handles = vec![
                AssetZeroLoader::load(world, AssetZero(10)).unwrap(),
                AssetZeroLoader::load(world, AssetZero(20)).unwrap(),
            ];

            world.add_resource::<Vec<AssetZeroHandle>>(handles);
        };
        let assertion_fn = |world: &mut World| {
            let asset_zero_handles = world.read_resource::<Vec<AssetZeroHandle>>();

            let store = world.read_resource::<AssetStorage<AssetZero>>();
            assert_eq!(Some(&AssetZero(10)), store.get(&asset_zero_handles[0]));
            assert_eq!(Some(&AssetZero(20)), store.get(&asset_zero_handles[1]));
        };

        assert!(
            AmethystApplication::blank()
                .with_bundle(BundleAsset)
                .with_effect(effect_fn)
                .with_assertion(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn state_runs_before_effect() {
        let first_state_fn = || LoadingState::new(EmptyState);
        let effect_fn = |world: &mut World| {
            // If `LoadingState` is not run before this, this will panic
            world.read_resource::<LoadResource>();

            let handles = vec![AssetZeroLoader::load(world, AssetZero(10)).unwrap()];
            world.add_resource(handles);
        };
        let assertion_fn = |world: &mut World| {
            let asset_zero_handles = world.read_resource::<Vec<AssetZeroHandle>>();

            let store = world.read_resource::<AssetStorage<AssetZero>>();
            assert_eq!(Some(&AssetZero(10)), store.get(&asset_zero_handles[0]));
        };

        assert!(
            AmethystApplication::blank()
                .with_bundle(BundleAsset)
                .with_state(first_state_fn)
                .with_effect(effect_fn)
                .with_assertion(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn base_application_can_load_ui() {
        let assertion_fn = |world: &mut World| {
            // Next line would panic if `UiBundle` wasn't added.
            world.read_resource::<AssetStorage<FontAsset>>();
            // `.base()` should add `ScreenDimensions` as this is necessary for `UiBundle` to
            // initialize properly.
            world.read_resource::<ScreenDimensions>();
        };

        assert!(
            AmethystApplication::base()
                .with_assertion(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn render_base_application_can_load_material_animations() {
        let effect_fn = |world: &mut World| {
            // Load the animation.
            let animation_handle = {
                let texture_sampler = Sampler {
                    input: vec![0.0],
                    output: vec![MaterialPrimitive::Texture(0)],
                    function: InterpolationFunction::Step,
                };
                let sprite_offset_sampler = Sampler {
                    input: vec![0.0],
                    output: vec![MaterialPrimitive::Offset((0.0, 1.0), (1.0, 0.0))],
                    function: InterpolationFunction::Step,
                };

                let loader = world.read_resource::<Loader>();
                let texture_animation_handle =
                    loader.load_from_data(texture_sampler, (), &world.read_resource());
                let sampler_animation_handle =
                    loader.load_from_data(sprite_offset_sampler, (), &world.read_resource());

                let animation = Animation::<Material> {
                    nodes: vec![
                        (0, MaterialChannel::AlbedoTexture, texture_animation_handle),
                        (0, MaterialChannel::AlbedoOffset, sampler_animation_handle),
                    ],
                };

                loader.load_from_data::<Animation<Material>, ()>(
                    animation,
                    (),
                    &world.read_resource(),
                )
            };
            world.add_resource(EffectReturn(animation_handle));
        };
        let assertion_fn = |world: &mut World| {
            // Read the animation.
            let animation_handle = &world
                .read_resource::<EffectReturn<Handle<Animation<Material>>>>()
                .0;

            let store = world.read_resource::<AssetStorage<Animation<Material>>>();
            assert!(store.get(animation_handle).is_some());
        };

        assert!(
            AmethystApplication::render_base("render_base_application_can_load_materials", false)
                .with_effect(effect_fn)
                .with_assertion(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn system_increases_component_value_by_one() {
        let effect_fn = |world: &mut World| {
            let entity = world.create_entity().with(ComponentZero(0)).build();

            world.add_resource(EffectReturn(entity));
        };
        let assertion_fn = |world: &mut World| {
            let entity = world.read_resource::<EffectReturn<Entity>>().0.clone();

            let component_zero_storage = world.read_storage::<ComponentZero>();
            let component_zero = component_zero_storage
                .get(entity)
                .expect("Entity should have a `ComponentZero` component.");

            // If the system ran, the value in the `ComponentZero` should be 1.
            assert_eq!(1, component_zero.0);
        };

        assert!(
            AmethystApplication::blank()
                .with_system(SystemEffect, "system_effect", &[])
                .with_effect(effect_fn)
                .with_assertion(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn with_system_invoked_twice_should_not_panic() {
        AmethystApplication::blank()
            .with_system(SystemZero, "zero", &[])
            .with_system(SystemOne, "one", &["zero"]);
    }

    // Incorrect usage tests

    #[test]
    #[should_panic(expected = "`.with_assertion(F)` has previously been called.")]
    fn with_assertion_invoked_twice_should_panic() {
        AmethystApplication::blank()
            .with_assertion(|_world: &mut World| {})
            .with_assertion(|_world: &mut World| {});
    }

    #[test]
    #[should_panic(expected = "`.with_effect(F)` has previously been called.")]
    fn with_effect_invoked_twice_should_panic() {
        AmethystApplication::blank()
            .with_effect(|_world: &mut World| {})
            .with_effect(|_world: &mut World| {});
    }

    #[test]
    #[should_panic(expected = "`.with_state(S)` has previously been called.")]
    fn with_state_invoked_twice_should_panic() {
        AmethystApplication::blank()
            .with_state::<_, (), _>(|| EmptyState)
            .with_state::<_, (), _>(|| EmptyState);
    }

    #[test]
    #[should_panic(expected = "`.with_state(S)` called after `.with_resource(R)`.")]
    fn with_state_invoked_after_with_resource_should_panic() {
        AmethystApplication::blank()
            .with_resource(ApplicationResource)
            .with_state::<_, (), _>(|| EmptyState);
    }

    // === Resources === //
    #[derive(Debug, Default)]
    struct ApplicationResource;
    #[derive(Debug)]
    struct ApplicationResourceNonDefault;
    #[derive(Debug)]
    struct LoadResource;

    // === States === //
    struct LoadingState<'a, 'b, S>
    where
        S: State<GameData<'a, 'b>> + 'static,
    {
        next_state: Option<S>,
        state_data: PhantomData<State<GameData<'a, 'b>>>,
    }
    impl<'a, 'b, S> LoadingState<'a, 'b, S>
    where
        S: State<GameData<'a, 'b>> + 'static,
    {
        fn new(next_state: S) -> Self {
            LoadingState {
                next_state: Some(next_state),
                state_data: PhantomData,
            }
        }
    }
    impl<'a, 'b, S> State<GameData<'a, 'b>> for LoadingState<'a, 'b, S>
    where
        S: State<GameData<'a, 'b>> + 'static,
    {
        fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
            data.data.update(&data.world);
            data.world.add_resource(LoadResource);
            Trans::Switch(Box::new(self.next_state.take().unwrap()))
        }
    }

    struct SwitchState<S, T>
    where
        S: State<T>,
    {
        next_state: Option<S>,
        state_data: PhantomData<T>,
    }
    impl<S, T> SwitchState<S, T>
    where
        S: State<T>,
    {
        fn new(next_state: S) -> Self {
            SwitchState {
                next_state: Some(next_state),
                state_data: PhantomData,
            }
        }
    }
    impl<S, T> State<T> for SwitchState<S, T>
    where
        S: State<T> + 'static,
    {
        fn update(&mut self, _data: StateData<T>) -> Trans<T> {
            Trans::Switch(Box::new(self.next_state.take().unwrap()))
        }
    }

    // === Systems === //
    #[derive(Debug)]
    struct SystemZero;
    impl<'s> System<'s> for SystemZero {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    #[derive(Debug)]
    struct SystemOne;
    type SystemOneData<'s> = Read<'s, ApplicationResource>;
    impl<'s> System<'s> for SystemOne {
        type SystemData = SystemOneData<'s>;
        fn run(&mut self, _: Self::SystemData) {}
    }

    #[derive(Debug)]
    struct SystemNonDefault;
    type SystemNonDefaultData<'s> = ReadExpect<'s, ApplicationResourceNonDefault>;
    impl<'s> System<'s> for SystemNonDefault {
        type SystemData = SystemNonDefaultData<'s>;
        fn run(&mut self, _: Self::SystemData) {}

        fn setup(&mut self, res: &mut Resources) {
            // Must be called when we override `.setup()`
            SystemNonDefaultData::setup(res);

            // Need to manually insert this when the resource is `!Default`
            res.insert(ApplicationResourceNonDefault);
        }
    }

    #[derive(Debug)]
    struct SystemEffect;
    type SystemEffectData<'s> = WriteStorage<'s, ComponentZero>;
    impl<'s> System<'s> for SystemEffect {
        type SystemData = SystemEffectData<'s>;
        fn run(&mut self, mut component_zero_storage: Self::SystemData) {
            for mut component_zero in (&mut component_zero_storage).join() {
                component_zero.0 += 1
            }
        }
    }

    // === Bundles === //
    #[derive(Debug)]
    struct BundleZero;
    impl<'a, 'b> SystemBundle<'a, 'b> for BundleZero {
        fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
            builder.add(SystemZero, "system_zero", &[]);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct BundleOne;
    impl<'a, 'b> SystemBundle<'a, 'b> for BundleOne {
        fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
            builder.add(SystemOne, "system_one", &["system_zero"]);
            builder.add(SystemNonDefault, "system_non_default", &[]);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct BundleAsset;
    impl<'a, 'b> SystemBundle<'a, 'b> for BundleAsset {
        fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
            builder.add(Processor::<AssetZero>::new(), "asset_zero_processor", &[]);
            Ok(())
        }
    }

    // === Assets === //
    #[derive(Debug, PartialEq)]
    struct AssetZero(u32);
    impl Asset for AssetZero {
        const NAME: &'static str = "amethyst_test_support::AssetZero";
        type Data = Self;
        type HandleStorage = VecStorage<Handle<Self>>;
    }
    impl Component for AssetZero {
        type Storage = DenseVecStorage<Self>;
    }
    impl From<AssetZero> for Result<ProcessingState<AssetZero>, assets::Error> {
        fn from(asset_zero: AssetZero) -> Result<ProcessingState<AssetZero>, assets::Error> {
            Ok(ProcessingState::Loaded(asset_zero))
        }
    }
    type AssetZeroHandle = Handle<AssetZero>;

    // === System delegates === //
    struct AssetZeroLoader;
    impl AssetZeroLoader {
        fn load(world: &World, asset_zero: AssetZero) -> Result<AssetZeroHandle, amethyst::Error> {
            let loader = world.read_resource::<Loader>();
            Ok(loader.load_from_data(
                asset_zero,
                (),
                &world.read_resource::<AssetStorage<AssetZero>>(),
            ))
        }
    }

    // === Components === //
    struct ComponentZero(pub i32);
    impl Component for ComponentZero {
        type Storage = DenseVecStorage<Self>;
    }
}