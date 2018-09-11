use amethyst::{
    ecs::prelude::*,
    shrev::{EventChannel, ReaderId},
    ui::{Anchor, UiText, UiTransform},
};
use application_ui::{FontVariant, Theme};
use character_selection::{CharacterSelectionEvent, CharacterSelections, CharacterSelectionsState};
use game_input::{ControllerId, ControllerInput, InputConfig, InputControlled};

use CharacterSelectionWidget;
use WidgetState;

const FONT_SIZE: f32 = 20.;

/// System that creates and deletes `CharacterSelectionWidget` entities.
///
/// This is not private because consumers may use `CharacterSelectionWidgetUiSystem::type_name()` to
/// specify this as a dependency of another system.
#[derive(Debug, Default, TypeName, new)]
pub(crate) struct CharacterSelectionWidgetUiSystem {
    /// Whether the UI is initialized.
    #[new(value = "false")]
    ui_initialized: bool,
    /// Reader ID for the `CharacterSelectionEvent` event channel.
    ///
    /// This is used to determine to delete the UI entities, as the `CharacterSelectionsState` is
    /// only updated by the `CharacterSelectionsSystem` which happens after this system runs.
    #[new(default)]
    reader_id: Option<ReaderId<CharacterSelectionEvent>>,
}

type WidgetComponentStorages<'s> = (
    WriteStorage<'s, CharacterSelectionWidget>,
    WriteStorage<'s, InputControlled>,
    WriteStorage<'s, ControllerInput>,
);

type WidgetUiResources<'s> = (
    ReadExpect<'s, Theme>,
    WriteStorage<'s, UiTransform>,
    WriteStorage<'s, UiText>,
);

type CharacterSelectionWidgetUiSystemData<'s> = (
    Read<'s, EventChannel<CharacterSelectionEvent>>,
    Read<'s, CharacterSelections>,
    Read<'s, InputConfig>,
    Entities<'s>,
    WidgetComponentStorages<'s>,
    WidgetUiResources<'s>,
);

impl CharacterSelectionWidgetUiSystem {
    fn initialize_ui(
        &mut self,
        input_config: &InputConfig,
        entities: &Entities,
        (
            character_selection_widgets,
            input_controlleds,
            controller_inputs
        ): &mut WidgetComponentStorages,
        (
            theme,
            ui_transforms,
            ui_texts
        ): &mut WidgetUiResources
){
        if !self.ui_initialized {
            debug!("Initializing Character Selection UI.");

            self.ui_initialized = true;
            let controller_count = input_config.controller_configs.len();

            let text_w = 200.;
            let text_h = 50.;

            let font = theme
                .fonts
                .get(&FontVariant::Regular)
                .expect("Failed to get regular font handle.");

            (0..controller_count).for_each(|index| {
                let controller_id = index as ControllerId;

                let character_selection_widget = CharacterSelectionWidget::default();

                let ui_transform = UiTransform::new(
                    format!("CharacterSelectionWidget#{}", controller_id),
                    Anchor::Middle,
                    0.,
                    (index as f32 * text_h) - (controller_count as f32 * text_h / 2.),
                    1.,
                    text_w,
                    text_h,
                    0,
                );

                let ui_text = UiText::new(
                    font.clone(),
                    "Inactive".to_string(),
                    [1., 1., 1., 1.],
                    FONT_SIZE,
                );

                entities
                    .build_entity()
                    .with(character_selection_widget, character_selection_widgets)
                    .with(InputControlled::new(controller_id), input_controlleds)
                    .with(ControllerInput::default(), controller_inputs)
                    .with(ui_transform, ui_transforms)
                    .with(ui_text, ui_texts)
                    .build();
            });
        }
    }

    fn refresh_ui(
        &mut self,
        character_selection_widgets: &mut WriteStorage<CharacterSelectionWidget>,
        ui_texts: &mut WriteStorage<UiText>,
    ) {
        (character_selection_widgets, ui_texts)
            .join()
            .for_each(|(widget, ui_text)| {
                ui_text.text = match widget.state {
                    WidgetState::Inactive => "Inactive".to_string(),
                    _ => format!("{}", widget.selection),
                }
            });
    }

    fn terminate_ui(
        &mut self,
        entities: &Entities,
        character_selection_widgets: &mut WriteStorage<CharacterSelectionWidget>,
    ) {
        if self.ui_initialized {
            (&**entities, character_selection_widgets)
                .join()
                .for_each(|(entity, _widget)| {
                    entities
                        .delete(entity)
                        .expect("Failed to delete `CharacterSelectionWidget` entity.")
                });
            self.ui_initialized = false;
        }
    }
}

impl<'s> System<'s> for CharacterSelectionWidgetUiSystem {
    type SystemData = CharacterSelectionWidgetUiSystemData<'s>;

    fn run(
        &mut self,
        (
            character_selection_events,
            character_selections,
            input_config,
            entities,
            mut widget_component_storages,
            mut widget_ui_resources,
        ): Self::SystemData,
    ) {
        // We need to do this because the `CharacterSelectionsState` is not updated until after this
        // system has run, and so we don't actually get a chance to delete the UI entities.
        if character_selection_events
            .read(
                self.reader_id
                    .as_mut()
                    .expect("Expected to read `CharacterSelectionEvent`s."),
            ).any(|ev| CharacterSelectionEvent::Confirm == *ev)
        {
            self.terminate_ui(&entities, &mut widget_component_storages.0);
            return;
        }

        match character_selections.state {
            CharacterSelectionsState::Waiting => {
                self.initialize_ui(
                    &input_config,
                    &entities,
                    &mut widget_component_storages,
                    &mut widget_ui_resources,
                );
                self.refresh_ui(&mut widget_component_storages.0, &mut widget_ui_resources.2);
            }
            CharacterSelectionsState::Ready => {
                self.terminate_ui(&entities, &mut widget_component_storages.0)
            }
            _ => self.refresh_ui(&mut widget_component_storages.0, &mut widget_ui_resources.2),
        };
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader_id = Some(
            res.fetch_mut::<EventChannel<CharacterSelectionEvent>>()
                .register_reader(),
        );
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use amethyst::{
        ecs::prelude::*,
        input::{Axis as InputAxis, Button},
        shrev::EventChannel,
        ui::UiText,
        winit::VirtualKeyCode,
    };
    use application_test_support::AutexousiousApplication;
    use assets_test::ASSETS_CHAR_BAT_SLUG;
    use character_selection::{
        CharacterSelection, CharacterSelectionEvent, CharacterSelections, CharacterSelectionsState,
    };
    use game_input::{Axis, ControlAction, ControllerConfig, InputConfig};
    use typename::TypeName;

    use super::CharacterSelectionWidgetUiSystem;
    use CharacterSelectionWidget;
    use WidgetState;

    #[test]
    fn initializes_ui_when_character_selections_waiting() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::config_base(
                "initializes_ui_when_character_selections_waiting",
                false
            ).with_setup(|world| {
                let mut character_selections = CharacterSelections::default();
                character_selections.state = CharacterSelectionsState::Waiting;
                world.add_resource(character_selections);
            }).with_setup(|world| world.add_resource(input_config()))
            .with_system_single(
                CharacterSelectionWidgetUiSystem::new(),
                CharacterSelectionWidgetUiSystem::type_name(),
                &[]
            ).with_assertion(|world| assert_widget_count(world, 2))
            .with_assertion(|world| assert_widget_text(world, "Inactive"))
            .run()
            .is_ok()
        );
    }

    #[test]
    fn refreshes_ui_when_selections_select_random() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::config_base(
                "refreshes_ui_when_selections_select_random",
                false
            ).with_system(
                CharacterSelectionWidgetUiSystem::new(),
                CharacterSelectionWidgetUiSystem::type_name(),
                &[]
            ) // kcov-ignore
            // Set up UI
            .with_resource(input_config())
            .with_resource(CharacterSelections::default())
            .with_assertion(|world| assert_widget_count(world, 2))
            // Select character and send event
            .with_effect(|world| {
                let mut character_selections = CharacterSelections::default();
                character_selections.state = CharacterSelectionsState::CharacterSelect;
                world.add_resource(character_selections);
            }).with_effect(|world| {
                world.exec(|mut widgets: WriteStorage<CharacterSelectionWidget>| {
                    let widget = (&mut widgets)
                        .join()
                        .next()
                        .expect("Expected entity with `CharacterSelectionWidget` component.");

                    widget.state = WidgetState::CharacterSelect;
                    widget.selection = CharacterSelection::Random;
                });

                send_event(
                    world,
                    CharacterSelectionEvent::Select {
                        controller_id: 123,
                        character_selection: CharacterSelection::Random,
                    },
                )
            }).with_effect(|_| {}) // Need an extra update for the event to get through.
            .with_assertion(|world| assert_widget_text(world, "Random"))
            .run()
            .is_ok()
        );
    }

    #[test]
    fn refreshes_ui_when_selections_select_id() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::config_base("refreshes_ui_when_selections_select_id", false)
                .with_system(
                    CharacterSelectionWidgetUiSystem::new(),
                    CharacterSelectionWidgetUiSystem::type_name(),
                    &[]
                ) // kcov-ignore
                // Set up UI
                .with_resource(input_config())
                .with_resource(CharacterSelections::default())
                .with_assertion(|world| assert_widget_count(world, 2))
                // Select character and send event
                .with_effect(|world| {
                    let mut character_selections = CharacterSelections::default();
                    character_selections.state = CharacterSelectionsState::CharacterSelect;
                    world.add_resource(character_selections);
                }).with_effect(|world| {
                    world.exec(|mut widgets: WriteStorage<CharacterSelectionWidget>| {
                        let widget = (&mut widgets)
                            .join()
                            .next()
                            .expect("Expected entity with `CharacterSelectionWidget` component.");

                        widget.state = WidgetState::CharacterSelect;
                        widget.selection = CharacterSelection::Id(ASSETS_CHAR_BAT_SLUG.clone());
                    });

                    send_event(
                        world,
                        CharacterSelectionEvent::Select {
                            controller_id: 123,
                            character_selection: CharacterSelection::Id(
                                ASSETS_CHAR_BAT_SLUG.clone(),
                            ),
                        },
                    )
                }).with_effect(|_| {}) // Need an extra update for the event to get through.
                .with_assertion(|world| assert_widget_text(
                    world,
                    &format!("{}", *ASSETS_CHAR_BAT_SLUG)
                )).run() // kcov-ignore
                .is_ok()
        );
    }

    #[test]
    fn terminates_ui_when_confirm_event_sent() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::config_base("terminates_ui_when_confirm_event_sent", false)
                .with_system(
                    CharacterSelectionWidgetUiSystem::new(),
                    CharacterSelectionWidgetUiSystem::type_name(),
                    &[]
                ) // kcov-ignore
                // Set up UI
                .with_resource(input_config())
                .with_resource(CharacterSelections::default())
                .with_assertion(|world| assert_widget_count(world, 2))
                // Confirm selection and send event
                .with_effect(|world| {
                    let mut character_selections = CharacterSelections::default();
                    character_selections.state = CharacterSelectionsState::Confirmed;
                    world.add_resource(character_selections);
                }).with_effect(|world| send_event(world, CharacterSelectionEvent::Confirm))
                .with_effect(|_| {}) // Need an extra update for the event to get through.
                .with_assertion(|world| assert_widget_count(world, 0))
                .run()
                .is_ok()
        );
    }

    fn input_config() -> InputConfig {
        let controller_config_0 =
            controller_config([VirtualKeyCode::A, VirtualKeyCode::D, VirtualKeyCode::Key1]);
        let controller_config_1 = controller_config([
            VirtualKeyCode::Left,
            VirtualKeyCode::Right,
            VirtualKeyCode::O,
        ]);

        let controller_configs = vec![controller_config_0, controller_config_1];
        InputConfig::new(controller_configs)
    }

    fn controller_config(keys: [VirtualKeyCode; 3]) -> ControllerConfig {
        let mut axes = HashMap::new();
        axes.insert(
            Axis::X,
            InputAxis::Emulated {
                neg: Button::Key(keys[0]),
                pos: Button::Key(keys[1]),
            },
        );
        let mut actions = HashMap::new();
        actions.insert(ControlAction::Jump, Button::Key(keys[2]));
        ControllerConfig::new(axes, actions)
    }

    fn send_event(world: &mut World, event: CharacterSelectionEvent) {
        world
            .write_resource::<EventChannel<CharacterSelectionEvent>>()
            .single_write(event);
    }

    fn assert_widget_count(world: &mut World, count: usize) {
        world.exec(|widgets: ReadStorage<CharacterSelectionWidget>| {
            assert_eq!(count, widgets.join().count());
        });
    }

    fn assert_widget_text(world: &mut World, text: &str) {
        world.exec(|ui_texts: ReadStorage<UiText>| {
            assert_eq!(
                text,
                ui_texts
                    .join()
                    .next()
                    .expect("Expected entity with `UiText` component to exist.")
                    .text
            );
        });
    }
}