use amethyst::{assets::AssetStorage, ecs::prelude::*};
use map_model::loaded::Map;
use map_selection_model::MapSelection;
use object_model::entity::{Grounding, Position, Velocity};

/// Updates `Character` kinematics based on sequence.
#[derive(Debug, Default, TypeName, new)]
pub(crate) struct CharacterGroundingSystem;

type CharacterGroundingSystemData<'s> = (
    ReadExpect<'s, MapSelection>,
    Read<'s, AssetStorage<Map>>,
    WriteStorage<'s, Position<f32>>,
    WriteStorage<'s, Velocity<f32>>,
    WriteStorage<'s, Grounding>,
);

impl<'s> System<'s> for CharacterGroundingSystem {
    type SystemData = CharacterGroundingSystemData<'s>;

    fn run(
        &mut self,
        (map_selection, maps, mut positions, mut velocities, mut groundings): Self::SystemData,
    ) {
        let map_margins = {
            maps.get(map_selection.handle())
                .map(|map| map.margins)
                .expect("Expected map to be loaded.")
        };

        for (mut position, mut velocity, mut grounding) in
            (&mut positions, &mut velocities, &mut groundings).join()
        {
            // X axis
            if position[0] < map_margins.left {
                position[0] = map_margins.left;
            } else if position[0] > map_margins.right {
                position[0] = map_margins.right;
            }

            // Y axis
            if position[1] > map_margins.bottom {
                velocity[1] += -1.7;
                *grounding = Grounding::Airborne;

                if position[1] > map_margins.top {
                    position[1] = map_margins.top;
                }
            } else {
                position[1] = map_margins.bottom;
                velocity[1] = 0.;
                *grounding = Grounding::OnGround;
            }

            // Z axis
            if position[2] < map_margins.back {
                position[2] = map_margins.back;
            } else if position[2] > map_margins.front {
                position[2] = map_margins.front;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use amethyst::ecs::prelude::*;
    use application_test_support::AutexousiousApplication;
    use object_model::entity::{Grounding, Position};
    use typename::TypeName;

    use super::CharacterGroundingSystem;

    #[test]
    fn keeps_character_within_lower_map_bounds() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::game_base("keeps_character_within_lower_map_bounds", false)
                .with_setup(|world| {
                    world.exec(
                        |(mut positions, groundings): (
                            WriteStorage<Position<f32>>,
                            ReadStorage<Grounding>,
                        )| {
                            for (position, _) in (&mut positions, &groundings).join() {
                                position[0] = -10.;
                                position[1] = -10.;
                                position[2] = -10.;
                            }
                        },
                    );
                })
                .with_system_single(
                    CharacterGroundingSystem::new(),
                    CharacterGroundingSystem::type_name(),
                    &[]
                )
                .with_assertion(|world| {
                    world.exec(
                        |(positions, groundings): (
                            ReadStorage<Position<f32>>,
                            ReadStorage<Grounding>,
                        )| {
                            for (position, _) in (&positions, &groundings).join() {
                                assert_eq!(1., position[0]);

                                // Map margins are shifted by z and depth. See
                                // `map_model::loaded::Margins`
                                assert_eq!(205., position[1]);
                                assert_eq!(3., position[2]);
                            }
                        },
                    );
                })
                .run()
                .is_ok()
        );
    }

    #[test]
    fn keeps_character_within_upper_map_bounds() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::game_base("keeps_character_within_upper_map_bounds", false)
                .with_setup(|world| {
                    world.exec(
                        |(mut positions, groundings): (
                            WriteStorage<Position<f32>>,
                            ReadStorage<Grounding>,
                        )| {
                            for (position, _) in (&mut positions, &groundings).join() {
                                position[0] = 2000.;
                                position[1] = 2000.;
                                position[2] = 2000.;
                            }
                        },
                    );
                })
                .with_system_single(
                    CharacterGroundingSystem::new(),
                    CharacterGroundingSystem::type_name(),
                    &[]
                )
                .with_assertion(|world| {
                    world.exec(
                        |(positions, groundings): (
                            ReadStorage<Position<f32>>,
                            ReadStorage<Grounding>,
                        )| {
                            for (position, _) in (&positions, &groundings).join() {
                                assert_eq!(801., position[0]);

                                // Map margins are shifted by z and depth. See
                                // `map_model::loaded::Margins`
                                assert_eq!(605., position[1]);
                                assert_eq!(203., position[2]);
                            }
                        },
                    );
                })
                .run()
                .is_ok()
        );
    }

    #[test]
    fn grounding_set_to_airborne_when_above_ground() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::game_base(
                "grounding_set_to_airborne_when_above_ground",
                false
            )
            .with_setup(|world| {
                world.exec(
                    |(mut positions, mut groundings): (
                        WriteStorage<Position<f32>>,
                        WriteStorage<Grounding>,
                    )| {
                        for (position, grounding) in (&mut positions, &mut groundings).join() {
                            position[1] = 300.;
                            *grounding = Grounding::OnGround;
                        }
                    },
                );
            })
            .with_system_single(
                CharacterGroundingSystem::new(),
                CharacterGroundingSystem::type_name(),
                &[]
            )
            .with_assertion(|world| {
                world.exec(
                    |(positions, groundings): (
                        ReadStorage<Position<f32>>,
                        ReadStorage<Grounding>,
                    )| {
                        for (_, grounding) in (&positions, &groundings).join() {
                            assert_eq!(Grounding::Airborne, *grounding);
                        }
                    },
                );
            })
            .run()
            .is_ok()
        );
    }

    #[test]
    fn grounding_set_to_on_ground_when_on_ground() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::game_base("grounding_set_to_on_ground_when_on_ground", false)
                .with_setup(|world| {
                    world.exec(
                        |(mut positions, mut groundings): (
                            WriteStorage<Position<f32>>,
                            WriteStorage<Grounding>,
                        )| {
                            for (position, grounding) in (&mut positions, &mut groundings).join() {
                                position[1] = 200.;
                                *grounding = Grounding::Airborne;
                            }
                        },
                    );
                })
                .with_system_single(
                    CharacterGroundingSystem::new(),
                    CharacterGroundingSystem::type_name(),
                    &[]
                )
                .with_assertion(|world| {
                    world.exec(
                        |(positions, groundings): (
                            ReadStorage<Position<f32>>,
                            ReadStorage<Grounding>,
                        )| {
                            for (_, grounding) in (&positions, &groundings).join() {
                                assert_eq!(Grounding::OnGround, *grounding);
                            }
                        },
                    );
                })
                .run()
                .is_ok()
        );
    }
}
