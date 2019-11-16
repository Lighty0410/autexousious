#[cfg(test)]
mod test {
    use std::str::FromStr;

    use game_mode_selection_model::GameModeIndex;
    use indexmap::IndexMap;
    use kinematic_model::config::PositionInit;
    use sequence_model::config::{SequenceEndTransition, SequenceNameString, Wait};
    use serde_yaml;
    use sprite_model::config::{SpriteFrame, SpriteRef};
    use ui_button_model::config::{UiButton, UiButtons};
    use ui_label_model::config::{UiLabel, UiSpriteLabel};
    use ui_menu_item_model::config::{UiMenuItem, UiMenuItems};

    use ui_model::config::{UiDefinition, UiSequence, UiSequences, UiType};

    const UI_MENU_YAML: &str = r#"
menu:
  # First item is active by default. The sequence here should correspond to the active status.
  - index: "start_game"
    label: { text: "Start Game" }
    position: { x: -1, y: -2, z: -3 }
    sprite: { sequence: "active" }

  - index: "exit"
    label: { position: { x: 1, y: 2, z: 3 }, text: "Exit" }
    position: { x: -1, y: -2, z: -3 }
    sprite: { sequence: "exit_inactive" }

buttons:
  - position: { x: -4, y: -5, z: -6 }
    label: { position: { x: -7, y: -8, z: -9 }, text: "Button Zero" }
    sprite: { position: { x: -10, y: -11, z: -12 }, sequence: "button_inactive" }

sequences:
  start_game_inactive:
    next: "none"
    frames:
      - { wait: 2, sprite: { sheet: 0, index: 0 } }

  active:
    next: "repeat"
    frames:
      - { wait: 2, sprite: { sheet: 0, index: 0 } }

  exit_inactive:
    next: "none"
    frames:
      - { wait: 2, sprite: { sheet: 0, index: 0 } }

  button_inactive:
    next: "none"
    frames:
      - { wait: 2, sprite: { sheet: 0, index: 0 } }
"#;

    #[test]
    fn deserialize_ui_definition() {
        let ui_definition = serde_yaml::from_str::<UiDefinition>(UI_MENU_YAML)
            .expect("Failed to deserialize `UiDefinition`.");

        let position_init = PositionInit::new(-1, -2, -3);
        let ui_type = UiType::Menu(UiMenuItems::new(vec![
            UiMenuItem::new(
                position_init,
                UiLabel::new(PositionInit::new(0, 0, 0), String::from("Start Game")),
                UiSpriteLabel::new(
                    PositionInit::new(0, 0, 0),
                    SequenceNameString::String(String::from("active")),
                ),
                GameModeIndex::StartGame,
            ),
            UiMenuItem::new(
                position_init,
                UiLabel::new(PositionInit::new(1, 2, 3), String::from("Exit")),
                UiSpriteLabel::new(
                    PositionInit::new(0, 0, 0),
                    SequenceNameString::from_str("exit_inactive").expect(
                        "Expected `SequenceNameString::from_str(\"exit_inactive\")` to succeed.",
                    ),
                ),
                GameModeIndex::Exit,
            ),
        ]));
        let buttons = UiButtons::new(vec![UiButton::new(
            PositionInit::new(-4, -5, -6),
            UiLabel::new(PositionInit::new(-7, -8, -9), String::from("Button Zero")),
            UiSpriteLabel::new(
                PositionInit::new(-10, -11, -12),
                SequenceNameString::String(String::from("button_inactive")),
            ),
        )]);
        let sequences = {
            let mut sequences = IndexMap::new();
            sequences.insert(
                SequenceNameString::String(String::from("start_game_inactive")),
                UiSequence::new(SequenceEndTransition::None, sprite_frames()),
            );
            sequences.insert(
                SequenceNameString::String(String::from("active")),
                UiSequence::new(SequenceEndTransition::Repeat, sprite_frames()),
            );
            sequences.insert(
                SequenceNameString::String(String::from("exit_inactive")),
                UiSequence::new(SequenceEndTransition::None, sprite_frames()),
            );
            sequences.insert(
                SequenceNameString::String(String::from("button_inactive")),
                UiSequence::new(SequenceEndTransition::None, sprite_frames()),
            );
            UiSequences::new(sequences)
        };
        let ui_definition_expected = UiDefinition {
            ui_type,
            buttons,
            sequences,
        };

        assert_eq!(ui_definition_expected, ui_definition);
    }

    fn sprite_frames() -> Vec<SpriteFrame> {
        vec![SpriteFrame {
            wait: Wait::new(2),
            sprite: SpriteRef::new(0, 0),
            ..Default::default()
        }]
    }
}
