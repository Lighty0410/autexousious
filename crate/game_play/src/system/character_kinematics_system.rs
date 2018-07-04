use amethyst::{assets::AssetStorage, ecs::prelude::*};
use object_model::{
    config::object::CharacterSequenceId,
    entity::{CharacterInput, Kinematics, ObjectStatus},
    loaded::{Character, CharacterHandle},
};

/// Updates `Character` sequence based on input
#[derive(Debug, Default, new)]
pub(crate) struct CharacterKinematicsSystem;

type CharacterKinematicsSystemData<'s, 'c> = (
    Read<'s, AssetStorage<Character>>,
    ReadStorage<'s, CharacterHandle>,
    ReadStorage<'s, CharacterInput>,
    ReadStorage<'s, ObjectStatus<CharacterSequenceId>>,
    WriteStorage<'s, Kinematics<f32>>,
);

impl<'s> System<'s> for CharacterKinematicsSystem {
    type SystemData = CharacterKinematicsSystemData<'s, 's>;

    fn run(
        &mut self,
        (
            characters,
            handle_storage,
            character_input_storage,
            status_storage,
            mut kinematics_storage,
        ): Self::SystemData,
    ) {
        for (character_handle, character_input, status, mut kinematics) in (
            &handle_storage,
            &character_input_storage,
            &status_storage,
            &mut kinematics_storage,
        ).join()
        {
            // TODO: Character stats should be configuration.
            // Use the stats from the character definition.
            let _character = characters
                .get(character_handle)
                .expect("Expected character to be loaded.");

            match status.sequence_id {
                CharacterSequenceId::Walk => {
                    kinematics.velocity[0] = character_input.x_axis_value as f32 * 3.5;
                    kinematics.velocity[2] = character_input.z_axis_value as f32 * -2.;
                }
                CharacterSequenceId::Stand => {
                    kinematics.velocity[0] = 0.;
                    kinematics.velocity[2] = 0.;
                }
            };
        }
    }
}