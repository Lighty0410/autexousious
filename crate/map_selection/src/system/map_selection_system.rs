use amethyst::{ecs::prelude::*, shrev::EventChannel};

use game_model::loaded::{MapAssets, SlugAndHandle};
use map_selection_model::{MapSelection, MapSelectionEvent};

use MapSelectionStatus;

/// Updates the `MapSelection` resource based on user selection.
#[derive(Debug, Default, TypeName, new)]
pub(crate) struct MapSelectionSystem {
    /// ID for reading map selection events.
    #[new(default)]
    reader_id: Option<ReaderId<MapSelectionEvent>>,
}

type MapSelectionSystemData<'s, 'c> = (
    Write<'s, MapSelectionStatus>,
    Read<'s, EventChannel<MapSelectionEvent>>,
    WriteExpect<'s, MapSelection>,
);

impl<'s> System<'s> for MapSelectionSystem {
    type SystemData = MapSelectionSystemData<'s, 's>;

    fn run(
        &mut self,
        (mut map_selection_status, selection_event_channel, mut map_selection): Self::SystemData,
    ) {
        if let MapSelectionStatus::Confirmed = *map_selection_status {
            return;
        }

        let mut events = selection_event_channel.read(self.reader_id.as_mut().unwrap());

        if let Some(MapSelectionEvent::Select {
            map_selection: selection,
        }) = events.next()
        {
            *map_selection_status = MapSelectionStatus::Confirmed;
            *map_selection = selection.clone();

            // Discard additional events, and log a message
            let additional_events = events.count();
            if additional_events > 0 {
                warn!(
                    "Discarding `{}` additional map selection events.",
                    additional_events
                );
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        if res.try_fetch::<MapSelection>().is_none() {
            let slug_and_handle = res
                .fetch::<MapAssets>()
                .iter()
                .next()
                .map(SlugAndHandle::from)
                .expect("Expected at least one map to be loaded.");

            res.insert(MapSelection::Random(slug_and_handle));
        }

        let mut selection_event_channel = res.fetch_mut::<EventChannel<MapSelectionEvent>>();
        self.reader_id = Some(selection_event_channel.register_reader());
    }
}

#[cfg(test)]
mod test {
    use amethyst::{assets::ProgressCounter, ecs::SystemData, prelude::*, shrev::EventChannel};
    use amethyst_test_support::prelude::*;
    use asset_loading::AssetDiscovery;
    use assets_test::{ASSETS_MAP_EMPTY_SLUG, ASSETS_MAP_FADE_SLUG, ASSETS_PATH};
    use game_model::loaded::SlugAndHandle;
    use loading::AssetLoader;
    use map_loading::MapLoadingBundle;
    use map_selection_model::{MapSelection, MapSelectionEvent};
    use typename::TypeName;

    use super::{MapSelectionSystem, MapSelectionSystemData};
    use MapSelectionStatus;

    #[test]
    fn returns_when_map_selection_status_confirmed() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AmethystApplication::render_base("returns_when_map_selection_status_confirmed", false)
                .with_bundle(MapLoadingBundle::new())
                .with_resource(MapSelectionStatus::Confirmed)
                .with_setup(setup_components)
                .with_setup(load_maps)
                .with_setup(|world| {
                    let fade_snh = SlugAndHandle::from((&*world, ASSETS_MAP_FADE_SLUG.clone()));
                    let map_selection = MapSelection::Id(fade_snh);
                    world.add_resource(map_selection);

                    // Send event, if the event is not responded to, then we know the system returns
                    // early.
                    let empty_snh = SlugAndHandle::from((&*world, ASSETS_MAP_EMPTY_SLUG.clone()));
                    let map_selection = MapSelection::Id(empty_snh);
                    send_event(world, MapSelectionEvent::Select { map_selection })
                })
                .with_system_single(
                    MapSelectionSystem::new(),
                    MapSelectionSystem::type_name(),
                    &[],
                )
                .with_assertion(|world| {
                    let fade_snh = SlugAndHandle::from((&*world, ASSETS_MAP_FADE_SLUG.clone()));

                    let map_selection = world.read_resource::<MapSelection>();
                    assert_eq!(MapSelection::Id(fade_snh), *map_selection);
                })
                .run()
                .is_ok()
        );
    }

    #[test]
    #[ignore]
    // TODO: Fails because the reader ID is registered after the event is sent.
    // See <https://gitlab.com/azriel91/autexousious/issues/74>
    fn selects_map_when_select_event_is_sent() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AmethystApplication::render_base("returns_when_map_selection_status_confirmed", false)
                .with_bundle(MapLoadingBundle::new())
                .with_setup(setup_components)
                .with_setup(load_maps)
                .with_setup(|world| {
                    let fade_snh = SlugAndHandle::from((&*world, ASSETS_MAP_FADE_SLUG.clone()));
                    let map_selection = MapSelection::Id(fade_snh);
                    world.add_resource(map_selection);

                    // Send event, if the event is responded to, then we know the system has read
                    // it.
                    let empty_snh = SlugAndHandle::from((&*world, ASSETS_MAP_EMPTY_SLUG.clone()));
                    let map_selection = MapSelection::Id(empty_snh);
                    send_event(world, MapSelectionEvent::Select { map_selection })
                })
                .with_system_single(
                    MapSelectionSystem::new(),
                    MapSelectionSystem::type_name(),
                    &[],
                )
                .with_assertion(|world| {
                    let empty_snh = SlugAndHandle::from((&*world, ASSETS_MAP_EMPTY_SLUG.clone()));

                    let map_selection = world.read_resource::<MapSelection>();
                    assert_eq!(MapSelection::Id(empty_snh), *map_selection);

                    let map_selection_status = world.read_resource::<MapSelectionStatus>();
                    assert_eq!(MapSelectionStatus::Confirmed, *map_selection_status);
                })
                .run()
                .is_ok()
        );
    }

    fn setup_components(world: &mut World) {
        MapSelectionSystemData::setup(&mut world.res);
    }

    fn load_maps(world: &mut World) {
        let asset_index = AssetDiscovery::asset_index(&ASSETS_PATH);

        let mut progress_counter = ProgressCounter::new();
        AssetLoader::load_maps(world, &mut progress_counter, asset_index.maps);
    }

    fn send_event(world: &mut World, event: MapSelectionEvent) {
        world
            .write_resource::<EventChannel<MapSelectionEvent>>()
            .single_write(event);
    }
}