use amethyst::{
    ecs::{Entities, Join, Read, ReadStorage, Resources, System, SystemData, Write},
    input::{InputEvent, InputHandler},
    shrev::{EventChannel, ReaderId},
};
use approx::relative_ne;
use derivative::Derivative;
use derive_new::new;
use game_input::{ControllerInput, InputControlled};
use game_input_model::{
    Axis, AxisEventData, ControlActionEventData, ControlBindings, ControlInputEvent, InputConfig,
    PlayerActionControl, PlayerAxisControl,
};
use log::debug;
use shred_derive::SystemData;
use strum::IntoEnumIterator;
use typename_derive::TypeName;

/// Sends `ControlInputEvent`s based on the `InputHandler` state.
#[derive(Debug, Default, TypeName, new)]
pub struct InputToControlInputSystem {
    /// All controller input configuration.
    input_config: InputConfig,
    /// Reader ID for the `InputEvent` channel.
    #[new(default)]
    input_event_rid: Option<ReaderId<InputEvent<PlayerActionControl>>>,
    /// Pre-allocated vector
    #[new(value = "Vec::with_capacity(64)")]
    control_input_events: Vec<ControlInputEvent>,
}

#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct InputToControlInputSystemData<'s> {
    /// `InputEvent<PlayerActionControl>` channel.
    #[derivative(Debug = "ignore")]
    pub input_ec: Read<'s, EventChannel<InputEvent<PlayerActionControl>>>,
    /// `InputHandler` resource.
    #[derivative(Debug = "ignore")]
    pub input_handler: Read<'s, InputHandler<ControlBindings>>,
    /// `Entities` resource.
    #[derivative(Debug = "ignore")]
    pub entities: Entities<'s>,
    /// `InputControlled` components.
    #[derivative(Debug = "ignore")]
    pub input_controlleds: ReadStorage<'s, InputControlled>,
    /// `ControllerInput` components.
    #[derivative(Debug = "ignore")]
    pub controller_inputs: ReadStorage<'s, ControllerInput>,
    /// `ControlInputEvent` channel.
    #[derivative(Debug = "ignore")]
    pub control_input_ec: Write<'s, EventChannel<ControlInputEvent>>,
}

impl<'s> System<'s> for InputToControlInputSystem {
    type SystemData = InputToControlInputSystemData<'s>;

    fn run(
        &mut self,
        InputToControlInputSystemData {
            input_ec,
            input_handler,
            entities,
            input_controlleds,
            controller_inputs,
            mut control_input_ec,
        }: Self::SystemData,
    ) {
        // This does not send events when there is no existing `ControllerInput` component attached
        // to the entity. This is to prevent events from being sent when we are restoring state,
        // e.g. in a saveload scenario.
        for (entity, input_controlled, controller_input) in
            (&*entities, &input_controlleds, &controller_inputs).join()
        {
            let controller_id = input_controlled.controller_id;

            Axis::iter().for_each(|axis| {
                if let Some(value) =
                    input_handler.axis_value(&PlayerAxisControl::new(controller_id, axis))
                {
                    let previous_value = match axis {
                        Axis::X => controller_input.x_axis_value,
                        Axis::Z => controller_input.z_axis_value,
                    };

                    if relative_ne!(previous_value, value) {
                        self.control_input_events
                            .push(ControlInputEvent::Axis(AxisEventData {
                                entity,
                                axis,
                                value,
                            }))
                    }
                }
            });
        }

        let input_event_rid = self
            .input_event_rid
            .as_mut()
            .expect("Expected `input_event_rid` field to be set.");

        input_ec.read(input_event_rid).for_each(|ev| {
            let control_input_event = match ev {
                InputEvent::ActionPressed(PlayerActionControl { player, action }) => {
                    // Find the entity has the `player` control id in its `InputControlled`
                    // component.

                    if let Some((entity, _)) = (&entities, &input_controlleds).join().find(
                        |(_entity, input_controlled)| input_controlled.controller_id == *player,
                    ) {
                        debug!(
                            "Sending control input event for action: {:?}, entity: {:?}",
                            *action, entity
                        );
                        Some(ControlInputEvent::ControlAction(ControlActionEventData {
                            entity,
                            control_action: *action,
                            value: true,
                        }))
                    } else {
                        None
                    }
                }
                InputEvent::ActionReleased(PlayerActionControl { player, action }) => {
                    if let Some((entity, _)) = (&entities, &input_controlleds).join().find(
                        |(_entity, input_controlled)| input_controlled.controller_id == *player,
                    ) {
                        Some(ControlInputEvent::ControlAction(ControlActionEventData {
                            entity,
                            control_action: *action,
                            value: false,
                        }))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(control_input_event) = control_input_event {
                self.control_input_events.push(control_input_event);
            }
        });

        control_input_ec.drain_vec_write(&mut self.control_input_events);
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        // TODO: figure out how to implement controller configuration updates, because we need to
        // update the resource and what this system stores.
        res.insert(self.input_config.clone());

        self.input_event_rid = Some(
            res.fetch_mut::<EventChannel<InputEvent<PlayerActionControl>>>()
                .register_reader(),
        );
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, convert::TryFrom};

    use amethyst::{
        ecs::{Builder, Entity},
        input::{Axis as InputAxis, Bindings, Button, InputEvent, InputHandler},
        shrev::{EventChannel, ReaderId},
        Error,
    };
    use amethyst_test::{AmethystApplication, HIDPI};
    use game_input::{ControllerInput, InputControlled};
    use game_input_model::{
        Axis, AxisEventData, ControlAction, ControlActionEventData, ControlBindings,
        ControlInputEvent, ControllerConfig, InputConfig, PlayerActionControl,
    };
    use hamcrest::prelude::*;
    use typename::TypeName;
    use winit::{
        DeviceId, ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent,
        WindowId,
    };

    use super::InputToControlInputSystem;

    const ACTION_JUMP: VirtualKeyCode = VirtualKeyCode::Key1;
    const AXIS_POSITIVE: VirtualKeyCode = VirtualKeyCode::D;
    const AXIS_NEGATIVE: VirtualKeyCode = VirtualKeyCode::A;

    #[test]
    fn sends_control_input_events_for_key_presses() -> Result<(), Error> {
        run_test(
            ControllerInput::default(),
            vec![key_press(AXIS_POSITIVE), key_press(ACTION_JUMP)],
            |entity| {
                vec![
                    ControlInputEvent::Axis(AxisEventData {
                        entity,
                        axis: Axis::X,
                        value: 1.,
                    }),
                    ControlInputEvent::ControlAction(ControlActionEventData {
                        entity,
                        control_action: ControlAction::Jump,
                        value: true,
                    }),
                ]
            },
        )
    }

    #[test]
    fn sends_control_input_events_for_key_releases() -> Result<(), Error> {
        let mut controller_input = ControllerInput::default();
        controller_input.x_axis_value = 1.;

        run_test(
            controller_input,
            vec![
                key_release(AXIS_POSITIVE),
                key_press(ACTION_JUMP),
                key_release(ACTION_JUMP),
            ],
            |entity| {
                vec![
                    ControlInputEvent::Axis(AxisEventData {
                        entity,
                        axis: Axis::X,
                        value: 0.,
                    }),
                    ControlInputEvent::ControlAction(ControlActionEventData {
                        entity,
                        control_action: ControlAction::Jump,
                        value: true,
                    }),
                    ControlInputEvent::ControlAction(ControlActionEventData {
                        entity,
                        control_action: ControlAction::Jump,
                        value: false,
                    }),
                ]
            },
        )
    }

    #[test]
    fn does_not_send_control_input_events_for_non_state_change() -> Result<(), Error> {
        let mut controller_input = ControllerInput::default();
        controller_input.x_axis_value = 1.;

        run_test(
            controller_input,
            vec![key_press(AXIS_POSITIVE)],
            |_entity| vec![],
        )
    }

    fn run_test<F>(
        controller_input: ControllerInput,
        key_events: Vec<Event>,
        expected_control_input_events: F,
    ) -> Result<(), Error>
    where
        F: Send + Sync + Fn(Entity) -> Vec<ControlInputEvent> + 'static,
    {
        let input_config = input_config();
        let bindings = Bindings::<ControlBindings>::try_from(&input_config)?;

        AmethystApplication::ui_base::<ControlBindings>()
            .with_system(
                InputToControlInputSystem::new(input_config),
                InputToControlInputSystem::type_name(),
                &[],
            ) // kcov-ignore
            .with_setup(move |world| {
                // HACK: This is what `InputSystem` does from `amethyst::input::InputBundle` in the
                // system setup phase.
                // TODO: Update `amethyst_test` to take in `InputBindings`.
                world
                    .write_resource::<InputHandler<ControlBindings>>()
                    .bindings = bindings.clone();

                let reader_id = world
                    .write_resource::<EventChannel<ControlInputEvent>>()
                    .register_reader(); // kcov-ignore
                world.add_resource(reader_id);

                let controller_id = 0;
                let entity = world
                    .create_entity()
                    .with(InputControlled::new(controller_id))
                    .with(controller_input)
                    .build();
                world.add_resource(entity);

                // Use the same closure so that the system does not send events before we send the
                // key events.

                let mut input_handler = world.write_resource::<InputHandler<ControlBindings>>();
                let mut input_events_ec =
                    world.write_resource::<EventChannel<InputEvent<PlayerActionControl>>>();

                key_events.iter().for_each(|ev| {
                    input_handler.send_event(ev, &mut input_events_ec, HIDPI as f32)
                });
            })
            .with_assertion(move |world| {
                let input_events = {
                    let input_events_ec = world.read_resource::<EventChannel<ControlInputEvent>>();
                    let mut input_events_id = world.write_resource::<ReaderId<ControlInputEvent>>();
                    input_events_ec
                        .read(&mut input_events_id)
                        .map(|ev| *ev)
                        .collect::<Vec<ControlInputEvent>>()
                };
                let entity = world.read_resource::<Entity>().clone();

                assert_that!(
                    &input_events,
                    contains(expected_control_input_events(entity))
                        .exactly()
                        .in_order()
                );
            })
            .run()
    }

    fn input_config() -> InputConfig {
        let controller_config_0 = controller_config([AXIS_NEGATIVE, AXIS_POSITIVE, ACTION_JUMP]);
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

    fn key_press(virtual_keycode: VirtualKeyCode) -> Event {
        key_event(virtual_keycode, ElementState::Pressed)
    }

    fn key_release(virtual_keycode: VirtualKeyCode) -> Event {
        key_event(virtual_keycode, ElementState::Released)
    }

    fn key_event(virtual_keycode: VirtualKeyCode, state: ElementState) -> Event {
        Event::WindowEvent {
            window_id: unsafe { WindowId::dummy() },
            event: WindowEvent::KeyboardInput {
                device_id: unsafe { DeviceId::dummy() },
                input: KeyboardInput {
                    scancode: 404,
                    state,
                    virtual_keycode: Some(virtual_keycode),
                    modifiers: ModifiersState {
                        shift: false,
                        ctrl: false,
                        alt: false,
                        logo: false,
                    },
                },
            },
        }
    }
}
