use object_model::{config::object::CharacterSequenceId, entity::SequenceStatus};

use character::sequence_handler::CharacterSequenceHandler;
use CharacterSequenceUpdateComponents;

#[derive(Debug)]
pub(crate) struct StandAttack;

impl CharacterSequenceHandler for StandAttack {
    fn update<'c>(
        components: CharacterSequenceUpdateComponents<'c>,
    ) -> Option<CharacterSequenceId> {
        if *components.sequence_status == SequenceStatus::End {
            Some(CharacterSequenceId::Stand)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use game_input::ControllerInput;
    use object_model::{
        config::object::CharacterSequenceId,
        entity::{
            CharacterStatus, Grounding, Mirrored, Position, RunCounter, SequenceStatus, Velocity,
        },
    };

    use super::StandAttack;
    use character::sequence_handler::CharacterSequenceHandler;
    use CharacterSequenceUpdateComponents;

    #[test]
    fn no_update_when_sequence_not_ended() {
        let input = ControllerInput::new(0., 0., false, false, false, false);

        assert_eq!(
            None,
            StandAttack::update(CharacterSequenceUpdateComponents::new(
                &input,
                &CharacterStatus::default(),
                &CharacterSequenceId::StandAttack,
                &SequenceStatus::default(),
                &Position::default(),
                &Velocity::default(),
                &Mirrored::default(),
                &Grounding::default(),
                &RunCounter::default()
            ))
        );
    }

    #[test]
    fn reverts_to_stand_when_sequence_ended() {
        let input = ControllerInput::new(0., 0., false, false, false, false);

        assert_eq!(
            Some(CharacterSequenceId::Stand),
            StandAttack::update(CharacterSequenceUpdateComponents::new(
                &input,
                &CharacterStatus::default(),
                &CharacterSequenceId::StandAttack,
                &SequenceStatus::End,
                &Position::default(),
                &Velocity::default(),
                &Mirrored::default(),
                &Grounding::default(),
                &RunCounter::default()
            ))
        );
    }
}
