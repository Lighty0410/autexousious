use object_model::{
    config::object::{CharacterSequenceId, SequenceStatus},
    entity::ObjectStatusUpdate,
};

use character::sequence_handler::CharacterSequenceHandler;
use CharacterSequenceUpdateComponents;

#[derive(Debug)]
pub(crate) struct Jump;

impl CharacterSequenceHandler for Jump {
    fn update<'c>(
        components: CharacterSequenceUpdateComponents<'c>,
    ) -> ObjectStatusUpdate<CharacterSequenceId> {
        let mut object_status_update = ObjectStatusUpdate::default();
        if components.object_status.sequence_status == SequenceStatus::End {
            object_status_update.sequence_id = Some(CharacterSequenceId::JumpOff);
            object_status_update.sequence_status = Some(SequenceStatus::Begin);
        }

        object_status_update
    }
}

#[cfg(test)]
mod test {
    use game_input::ControllerInput;
    use object_model::{
        config::object::{CharacterSequenceId, SequenceStatus},
        entity::{
            CharacterStatus, Grounding, Kinematics, Mirrored, ObjectStatus, ObjectStatusUpdate,
            RunCounter,
        },
    };

    use super::Jump;
    use character::sequence_handler::CharacterSequenceHandler;
    use CharacterSequenceUpdateComponents;

    #[test]
    fn no_update_when_sequence_not_ended() {
        let input = ControllerInput::new(0., 0., false, false, false, false);

        assert_eq!(
            ObjectStatusUpdate::default(),
            Jump::update(CharacterSequenceUpdateComponents::new(
                &input,
                &CharacterStatus::default(),
                &ObjectStatus {
                    sequence_id: CharacterSequenceId::Jump,
                    ..Default::default()
                },
                &Kinematics::default(),
                Mirrored::default(),
                Grounding::default(),
                RunCounter::default()
            ))
        );
    }

    #[test]
    fn switches_to_jump_off_when_sequence_ends() {
        let input = ControllerInput::new(0., 0., false, false, false, false);
        let mut kinematics = Kinematics::default();
        kinematics.velocity[1] = 1.;

        assert_eq!(
            ObjectStatusUpdate {
                sequence_id: Some(CharacterSequenceId::JumpOff),
                sequence_status: Some(SequenceStatus::Begin),
            },
            Jump::update(CharacterSequenceUpdateComponents::new(
                &input,
                &CharacterStatus::default(),
                &ObjectStatus {
                    sequence_id: CharacterSequenceId::Jump,
                    sequence_status: SequenceStatus::End,
                },
                &kinematics,
                Mirrored::default(),
                Grounding::default(),
                RunCounter::default()
            ))
        );
    }
}
