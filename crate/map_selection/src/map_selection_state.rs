use std::fmt::Debug;
use std::marker::PhantomData;

use amethyst::{ecs::prelude::*, prelude::*, shrev::EventChannel};
use game_model::loaded::{MapAssets, SlugAndHandle};
use typename::TypeName;

use MapSelection;
use MapSelectionEvent;
use MapSelectionStatus;
use MapSelectionSystem;

/// `State` where map selection takes place.
///
/// # Type Parameters
///
/// * `F`: Function to construct the state to return after map selection is complete.
/// * `S`: State to return.
#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct MapSelectionState<'a, 'b, F, S, E>
where
    F: Fn() -> Box<S>,
    S: State<GameData<'a, 'b>, E> + 'static,
    E: Send + Sync + 'static,
{
    /// The `State` that follows this one.
    #[derivative(Debug(bound = "F: Debug"))]
    next_state_fn: Box<F>,
    /// State specific dispatcher.
    #[derivative(Debug = "ignore")]
    #[new(default)]
    dispatcher: Option<Dispatcher<'static, 'static>>,
    /// Data type used by this state and the returned state (see `StateData`).
    state_data: PhantomData<(GameData<'a, 'b>, E)>,
}

impl<'a, 'b, F, S, E> MapSelectionState<'a, 'b, F, S, E>
where
    F: Fn() -> Box<S>,
    S: State<GameData<'a, 'b>, E> + 'static,
    E: Send + Sync + 'static,
{
    fn reset_map_selection_state(&self, world: &mut World) {
        let mut map_selection_status = world.write_resource::<MapSelectionStatus>();
        *map_selection_status = MapSelectionStatus::Pending;
    }
}

impl<'a, 'b, F, S, E> State<GameData<'a, 'b>, E> for MapSelectionState<'a, 'b, F, S, E>
where
    F: Fn() -> Box<S>,
    S: State<GameData<'a, 'b>, E> + 'static,
    E: Send + Sync + 'static,
{
    fn on_start(&mut self, data: StateData<GameData<'a, 'b>>) {
        let mut dispatcher_builder = DispatcherBuilder::new();
        dispatcher_builder.add(
            MapSelectionSystem::new(),
            &MapSelectionSystem::type_name(),
            &[],
        );
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(&mut data.world.res);
        self.dispatcher = Some(dispatcher);

        self.reset_map_selection_state(data.world);
    }

    fn on_resume(&mut self, data: StateData<GameData<'a, 'b>>) {
        self.reset_map_selection_state(data.world);
    }

    fn on_stop(&mut self, _data: StateData<GameData<'a, 'b>>) {
        self.dispatcher = None;
    }

    fn fixed_update(&mut self, data: StateData<GameData<'a, 'b>>) -> Trans<GameData<'a, 'b>, E> {
        data.data.update(&data.world);
        self.dispatcher.as_mut().unwrap().dispatch(&data.world.res);

        let map_selection_status = data.world.read_resource::<MapSelectionStatus>();
        if *map_selection_status == MapSelectionStatus::Confirmed {
            let map_selection = data.world.read_resource::<MapSelection>();

            info!("Map selection: `{}`", *map_selection);

            // TODO: `Trans:Push` when we have a proper map selection menu.
            Trans::Switch((self.next_state_fn)())
        } else {
            // TODO: Implement menu.
            let mut selection_event_channel = data
                .world
                .write_resource::<EventChannel<MapSelectionEvent>>();
            let map_selection = data
                .world
                .read_resource::<MapAssets>()
                .iter()
                .nth(1) // Skip built-in map.
                .map(SlugAndHandle::from)
                .expect("Expect at least one map to be loaded.");
            selection_event_channel.single_write(MapSelectionEvent::Select {
                map_selection: MapSelection::Id(map_selection),
            });

            Trans::None
        }
    }
}
