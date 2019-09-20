use amethyst::{
    ecs::{Read, ReadExpect, ReadStorage, System, World, WriteStorage},
    shred::{ResourceId, SystemData},
    shrev::{EventChannel, ReaderId},
};
use derivative::Derivative;
use derive_new::new;
use kinematic_model::config::Position;
use map_model::{
    loaded::AssetMargins,
    play::{BoundaryFace, MapBoundaryEvent, MapBoundaryEventData, MapBounded},
};
use map_selection_model::MapSelection;
use typename_derive::TypeName;

/// Keeps entities within map bounds.
#[derive(Debug, Default, TypeName, new)]
pub struct KeepWithinMapBoundsSystem {
    /// Reader ID for the `MapBoundaryEvent` channel.
    #[new(default)]
    map_boundary_event_rid: Option<ReaderId<MapBoundaryEvent>>,
}

#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct KeepWithinMapBoundsSystemData<'s> {
    /// `MapBoundaryEvent` channel.
    #[derivative(Debug = "ignore")]
    pub map_boundary_ec: Read<'s, EventChannel<MapBoundaryEvent>>,
    /// `MapSelection` resource.
    #[derivative(Debug = "ignore")]
    pub map_selection: ReadExpect<'s, MapSelection>,
    /// `AssetMargins` resource.
    #[derivative(Debug = "ignore")]
    pub asset_margins: Read<'s, AssetMargins>,
    /// `MapBounded` components.
    #[derivative(Debug = "ignore")]
    pub map_boundeds: ReadStorage<'s, MapBounded>,
    /// `Position<f32>` components.
    #[derivative(Debug = "ignore")]
    pub positions: WriteStorage<'s, Position<f32>>,
}

impl<'s> System<'s> for KeepWithinMapBoundsSystem {
    type SystemData = KeepWithinMapBoundsSystemData<'s>;

    fn run(
        &mut self,
        KeepWithinMapBoundsSystemData {
            map_boundary_ec,
            map_selection,
            asset_margins,
            map_boundeds,
            mut positions,
        }: Self::SystemData,
    ) {
        let map_margins = asset_margins
            .get(
                map_selection
                    .asset_id()
                    .expect("Expected `MapSelection` asset ID to exist."),
            )
            .expect("Expected `Margins` to be loaded.");

        let map_boundary_event_rid = self
            .map_boundary_event_rid
            .as_mut()
            .expect("Expected `map_boundary_event_rid` field to be set.");

        map_boundary_ec.read(map_boundary_event_rid).for_each(|ev| {
            if let MapBoundaryEvent::Exit(MapBoundaryEventData {
                entity,
                boundary_faces,
            }) = ev
            {
                let entity = *entity;
                if let (Some(_), Some(position)) =
                    (map_boundeds.get(entity), positions.get_mut(entity))
                {
                    if boundary_faces.contains(BoundaryFace::Left) {
                        position[0] = map_margins.left;
                    } else if boundary_faces.contains(BoundaryFace::Right) {
                        position[0] = map_margins.right;
                    }

                    if boundary_faces.contains(BoundaryFace::Bottom) {
                        position[1] = map_margins.bottom;
                    } else if boundary_faces.contains(BoundaryFace::Top) {
                        position[1] = map_margins.top;
                    }

                    if boundary_faces.contains(BoundaryFace::Back) {
                        position[2] = map_margins.back;
                    } else if boundary_faces.contains(BoundaryFace::Front) {
                        position[2] = map_margins.front;
                    }
                }
            }
        });
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.map_boundary_event_rid = Some(
            world
                .fetch_mut::<EventChannel<MapBoundaryEvent>>()
                .register_reader(),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use amethyst::{
        ecs::{Builder, Entity, System, SystemData, World, WorldExt},
        shrev::EventChannel,
        Error,
    };
    use amethyst_test::AmethystApplication;
    use asset_model::{config::AssetSlug, loaded::AssetIdMappings};
    use enumflags2::BitFlags;
    use kinematic_model::config::Position;
    use map_loading::MapLoadingBundle;
    use map_model::{
        config::MapBounds,
        loaded::{AssetMargins, Margins},
        play::{BoundaryFace, MapBoundaryEvent, MapBoundaryEventData, MapBounded},
    };
    use map_selection_model::MapSelection;
    use typename::TypeName;

    use super::KeepWithinMapBoundsSystem;

    #[test]
    fn does_not_change_position_when_no_map_boundary_event() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(100., 300., 100.),
                map_boundary_event_fn: None,
            },
            ExpectedParams {
                position: Position::new(100., 300., 100.),
            },
        )
    }

    #[test]
    fn does_not_change_position_on_enter_event() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(100., 300., 100.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces =
                        BoundaryFace::Left | BoundaryFace::Bottom | BoundaryFace::Back;
                    MapBoundaryEvent::Enter(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(100., 300., 100.),
            },
        )
    }

    #[test]
    fn sets_x_to_left_margin_on_exit_event_left() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(-10., 300., 100.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BitFlags::from(BoundaryFace::Left);
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(0., 300., 100.),
            },
        )
    }

    #[test]
    fn sets_x_to_right_margin_on_exit_event_right() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(810., 300., 100.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BitFlags::from(BoundaryFace::Right);
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(800., 300., 100.),
            },
        )
    }

    #[test]
    fn sets_y_to_bottom_margin_on_exit_event_bottom() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(100., 190., 100.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BitFlags::from(BoundaryFace::Bottom);
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(100., 200., 100.),
            },
        )
    }

    #[test]
    fn sets_y_to_top_margin_on_exit_event_top() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(100., 810., 100.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BitFlags::from(BoundaryFace::Top);
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(100., 800., 100.),
            },
        )
    }

    #[test]
    fn sets_z_to_back_margin_on_exit_event_back() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(100., 300., -10.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BitFlags::from(BoundaryFace::Back);
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(100., 300., 0.),
            },
        )
    }

    #[test]
    fn sets_z_to_front_margin_on_exit_event_front() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(100., 300., 210.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BitFlags::from(BoundaryFace::Front);
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(100., 300., 200.),
            },
        )
    }

    #[test]
    fn aligns_with_left_and_bottom_margins_on_exit_event_left_bottom() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(-10., 190., 100.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BoundaryFace::Left | BoundaryFace::Bottom;
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(0., 200., 100.),
            },
        )
    }

    #[test]
    fn aligns_with_right_and_top_margins_on_exit_event_right_top() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(810., 810., 100.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BoundaryFace::Right | BoundaryFace::Top;
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(800., 800., 100.),
            },
        )
    }

    #[test]
    fn aligns_with_bottom_and_back_margins_on_exit_event_bottom_back() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(100., 190., -10.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BoundaryFace::Bottom | BoundaryFace::Back;
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(100., 200., 0.),
            },
        )
    }

    #[test]
    fn aligns_with_right_top_front_margins_on_exit_event_right_top_front() -> Result<(), Error> {
        run_test(
            SetupParams {
                position: Position::new(810., 810., 210.),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces =
                        BoundaryFace::Right | BoundaryFace::Top | BoundaryFace::Front;
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                position: Position::new(800., 800., 200.),
            },
        )
    }

    fn run_test(
        SetupParams {
            position,
            map_boundary_event_fn,
        }: SetupParams,
        ExpectedParams {
            position: position_expected,
        }: ExpectedParams,
    ) -> Result<(), Error> {
        AmethystApplication::blank()
            .with_bundle(MapLoadingBundle::new())
            .with_system(
                KeepWithinMapBoundsSystem::new(),
                KeepWithinMapBoundsSystem::type_name(),
                &[String::from("map_processor")],
            ) // kcov-ignore
            .with_setup(setup_system_data)
            .with_setup(setup_map_selection)
            .with_effect(move |world| {
                let entity = world
                    .create_entity()
                    .with(position)
                    .with(MapBounded)
                    .build();

                if let Some(map_boundary_event_fn) = map_boundary_event_fn {
                    let map_boundary_event = map_boundary_event_fn(entity);
                    let mut map_boundary_ec =
                        world.write_resource::<EventChannel<MapBoundaryEvent>>();

                    map_boundary_ec.single_write(map_boundary_event);
                }

                world.insert(entity);
            })
            .with_assertion(move |world| {
                let entity = *world.read_resource::<Entity>();
                let positions = world.read_storage::<Position<f32>>();
                let position_actual = positions
                    .get(entity)
                    .copied()
                    .expect("Expected entity to have `Position<f32>` component.");

                assert_eq!(position_expected, position_actual);
            })
            .run()
    }

    fn setup_system_data(world: &mut World) {
        <KeepWithinMapBoundsSystem as System<'_>>::SystemData::setup(world);
    }

    fn setup_map_selection(world: &mut World) {
        let map_selection = {
            let map_bounds = MapBounds::new(0, 0, 0, 800, 600, 200);
            let map_margins = Margins::from(map_bounds);

            let mut asset_id_mappings = world.write_resource::<AssetIdMappings>();
            let mut asset_margins = world.write_resource::<AssetMargins>();
            let slug =
                AssetSlug::from_str("test/empty_map").expect("Expected asset slug to be valid.");

            let asset_id = asset_id_mappings.insert(slug);
            asset_margins.insert(asset_id, map_margins);

            MapSelection::Id(asset_id)
        };

        world.insert(map_selection);
    }

    struct SetupParams {
        position: Position<f32>,
        map_boundary_event_fn: Option<fn(Entity) -> MapBoundaryEvent>,
    }

    struct ExpectedParams {
        position: Position<f32>,
    }
}
