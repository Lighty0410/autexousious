use game_input::ControllerInput;
use object_model::{
    config::object::CharacterSequenceId,
    entity::{CharacterStatus, CharacterStatusUpdate, Kinematics},
};

use character::sequence_handler::{CharacterSequenceHandler, SwitchSequenceOnEnd};

const STAND_ON_SEQUENCE_END: SwitchSequenceOnEnd = SwitchSequenceOnEnd(CharacterSequenceId::Stand);

#[derive(Debug)]
pub(crate) struct StandOnSequenceEnd;

impl CharacterSequenceHandler for StandOnSequenceEnd {
    fn update(
        controller_input: &ControllerInput,
        character_status: &CharacterStatus,
        kinematics: &Kinematics<f32>,
    ) -> CharacterStatusUpdate {
        STAND_ON_SEQUENCE_END.update(controller_input, character_status, kinematics)
    }
}