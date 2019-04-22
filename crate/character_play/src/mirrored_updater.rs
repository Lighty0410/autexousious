use character_model::config::CharacterSequenceId;
use game_input::ControllerInput;
use object_model::play::Mirrored;

use crate::sequence_handler::SequenceHandlerUtil;

/// Updates the `Mirrored` component for character entities.
#[derive(Debug)]
pub struct MirroredUpdater;

impl MirroredUpdater {
    /// Returns the updated `Mirrored` value.
    ///
    /// # Parameters
    ///
    /// * `controller_input`: Controller input for this character.
    /// * `character_sequence_id`: Current character sequence ID.
    /// * `mirrored`: Whether the object is mirrored (facing left).
    pub fn update(
        controller_input: &ControllerInput,
        character_sequence_id: CharacterSequenceId,
        mirrored: Mirrored,
    ) -> Mirrored {
        match character_sequence_id {
            CharacterSequenceId::Stand
            | CharacterSequenceId::Walk
            | CharacterSequenceId::JumpAscend
            | CharacterSequenceId::JumpDescend => {}
            _ => return mirrored,
        }

        if SequenceHandlerUtil::input_opposes_direction(controller_input, mirrored) {
            !mirrored
        } else {
            mirrored
        }
    }
}

#[cfg(test)]
mod tests {
    use character_model::config::CharacterSequenceId;
    use game_input::ControllerInput;
    use object_model::play::Mirrored;

    use super::MirroredUpdater;

    #[test]
    fn no_change_when_no_input() {
        let input = ControllerInput::default();

        vec![true, false].into_iter().for_each(|mirrored| {
            verify_for_sequences(mirrored.into(), &input, mirrored.into());
        });
    }

    #[test]
    fn no_change_when_non_applicable_sequences() {
        let input = ControllerInput::default();

        vec![true, false].into_iter().for_each(|mirrored| {
            verify_for_sequences_na(&input, mirrored.into());
        });
    }

    #[test]
    fn true_when_input_is_left() {
        let mut input = ControllerInput::default();
        input.x_axis_value = -1.;

        vec![true, false].into_iter().for_each(|mirrored| {
            verify_for_sequences(Mirrored(true), &input, mirrored.into());
        });
    }

    #[test]
    fn false_when_input_is_right() {
        let mut input = ControllerInput::default();
        input.x_axis_value = 1.;

        vec![true, false].into_iter().for_each(|mirrored| {
            verify_for_sequences(Mirrored(false), &input, mirrored.into());
        });
    }

    fn verify_for_sequences(
        expected: Mirrored,
        controller_input: &ControllerInput,
        mirrored: Mirrored,
    ) {
        vec![
            CharacterSequenceId::Stand,
            CharacterSequenceId::Walk,
            CharacterSequenceId::JumpAscend,
            CharacterSequenceId::JumpDescend,
        ]
        .into_iter()
        .for_each(|sequence_id| {
            assert_eq!(
                expected,
                MirroredUpdater::update(controller_input, sequence_id, mirrored,)
            );
        });
    }

    fn verify_for_sequences_na(controller_input: &ControllerInput, mirrored: Mirrored) {
        vec![CharacterSequenceId::Run, CharacterSequenceId::RunStop]
            .into_iter()
            .for_each(|sequence_id| {
                assert_eq!(
                    mirrored,
                    MirroredUpdater::update(controller_input, sequence_id, mirrored,)
                );
            });
    }
}
