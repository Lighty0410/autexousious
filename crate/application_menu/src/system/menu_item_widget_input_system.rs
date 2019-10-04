use std::{fmt::Debug, marker::PhantomData};

use amethyst::{
    ecs::{Entities, Entity, Join, Read, ReadStorage, System, World, Write, WriteStorage},
    shred::{ResourceId, SystemData},
    shrev::{EventChannel, ReaderId},
};
use derivative::Derivative;
use derive_new::new;
use game_input_model::{
    Axis, AxisMoveEventData, ControlAction, ControlActionEventData, ControlInputEvent,
};
use log::debug;
use typename::TypeName as TypeNameTrait;
use typename_derive::TypeName;

use crate::{MenuEvent, MenuItem, MenuItemWidgetState, Siblings};

/// System that processes controller input and generates `MenuEvent<I>`s.
#[derive(Debug, Default, TypeName, new)]
pub struct MenuItemWidgetInputSystem<I>
where
    I: Clone + Copy + Debug + PartialEq + Send + Sync + TypeNameTrait + 'static,
{
    /// Reader ID for the `ControlInputEvent` channel.
    #[new(default)]
    control_input_event_rid: Option<ReaderId<ControlInputEvent>>,
    /// PhantomData.
    phantom_data: PhantomData<I>,
}

/// `MenuItemWidgetInputResources`.
#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct MenuItemWidgetInputResources<'s, I>
where
    I: Clone + Copy + Debug + PartialEq + Send + Sync + TypeNameTrait + 'static,
{
    /// `Entities` resource.
    #[derivative(Debug = "ignore")]
    pub entities: Entities<'s>,
    /// `MenuItem` components.
    #[derivative(Debug = "ignore")]
    pub menu_items: ReadStorage<'s, MenuItem<I>>,
    /// `MenuItemWidgetState` components.
    #[derivative(Debug = "ignore")]
    pub menu_item_widget_states: WriteStorage<'s, MenuItemWidgetState>,
    /// `Siblings` components.
    #[derivative(Debug = "ignore")]
    pub siblingses: ReadStorage<'s, Siblings>,
    /// `MenuEvent<I>` channel.
    #[derivative(Debug = "ignore")]
    pub menu_ec: Write<'s, EventChannel<MenuEvent<I>>>,
}

/// `MenuItemWidgetInputSystemData`.
#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct MenuItemWidgetInputSystemData<'s, I>
where
    I: Clone + Copy + Debug + PartialEq + Send + Sync + TypeNameTrait + 'static,
{
    /// `ControlInputEvent` channel.
    #[derivative(Debug = "ignore")]
    pub control_input_ec: Read<'s, EventChannel<ControlInputEvent>>,
    /// `MenuItemWidgetInputResources`.
    pub menu_item_widget_input_resources: MenuItemWidgetInputResources<'s, I>,
}

impl<I> MenuItemWidgetInputSystem<I>
where
    I: Clone + Copy + Debug + PartialEq + Send + Sync + TypeNameTrait + 'static,
{
    fn select_previous_menu_item(
        menu_item_widget_states: &mut WriteStorage<'_, MenuItemWidgetState>,
        menu_item_entity: Entity,
        siblings: &Siblings,
    ) {
        if let Some(previous_menu_item) = siblings.previous.as_ref() {
            {
                let menu_item_widget_state = menu_item_widget_states
                    .get_mut(menu_item_entity)
                    .expect("Expected `MenuItemWidgetState` component to exist.");
                *menu_item_widget_state = MenuItemWidgetState::Idle;
            }
            {
                let menu_item_widget_state = menu_item_widget_states
                    .get_mut(*previous_menu_item)
                    .expect("Expected `MenuItemWidgetState` component to exist.");
                *menu_item_widget_state = MenuItemWidgetState::Active;
            }
        }
    }

    fn select_next_menu_item(
        menu_item_widget_states: &mut WriteStorage<'_, MenuItemWidgetState>,
        menu_item_entity: Entity,
        siblings: &Siblings,
    ) {
        if let Some(next_menu_item) = siblings.next.as_ref() {
            {
                let menu_item_widget_state = menu_item_widget_states
                    .get_mut(menu_item_entity)
                    .expect("Expected `MenuItemWidgetState` component to exist.");
                *menu_item_widget_state = MenuItemWidgetState::Idle;
            }
            {
                let menu_item_widget_state = menu_item_widget_states
                    .get_mut(*next_menu_item)
                    .expect("Expected `MenuItemWidgetState` component to exist.");
                *menu_item_widget_state = MenuItemWidgetState::Active;
            }
        }
    }

    fn handle_event(
        MenuItemWidgetInputResources {
            ref entities,
            ref menu_items,
            ref mut menu_item_widget_states,
            ref siblingses,
            ref mut menu_ec,
        }: &mut MenuItemWidgetInputResources<I>,
        event: ControlInputEvent,
    ) {
        // Need to get from `menu_item_widget_states` separately, so that we do not hold an
        // immutable reference. This will then allow us to pass it to lower level functions.
        if let Some((menu_item_entity, siblings)) = (entities, siblingses)
            .join()
            .filter_map(|(entity, siblings)| {
                if let Some(menu_item_widget_state) = menu_item_widget_states.get(entity) {
                    if *menu_item_widget_state == MenuItemWidgetState::Active {
                        return Some((entity, siblings));
                    }
                }
                None
            })
            .next()
        {
            match event {
                ControlInputEvent::AxisMoved(axis_move_event_data) => Self::handle_axis_event(
                    menu_item_widget_states,
                    menu_item_entity,
                    siblings,
                    axis_move_event_data,
                ),
                ControlInputEvent::ControlActionPress(control_action_event_data) => {
                    Self::handle_control_action_event(
                        menu_items,
                        menu_ec,
                        menu_item_entity,
                        control_action_event_data,
                    )
                }
                ControlInputEvent::ControlActionRelease(..) => {}
            }
        }
    }

    fn handle_axis_event(
        menu_item_widget_states: &mut WriteStorage<'_, MenuItemWidgetState>,
        menu_item_entity: Entity,
        siblings: &Siblings,
        axis_move_event_data: AxisMoveEventData,
    ) {
        let menu_item_widget_state = *menu_item_widget_states
            .get(menu_item_entity)
            .expect("Expected `MenuItemWidgetState` component to exist.");
        match (menu_item_widget_state, axis_move_event_data.axis) {
            (MenuItemWidgetState::Active, Axis::Z) if axis_move_event_data.value < 0. => {
                Self::select_previous_menu_item(
                    menu_item_widget_states,
                    menu_item_entity,
                    siblings,
                );
            }
            (MenuItemWidgetState::Active, Axis::Z) if axis_move_event_data.value > 0. => {
                Self::select_next_menu_item(menu_item_widget_states, menu_item_entity, siblings);
            }
            _ => {}
        }
    }

    fn handle_control_action_event(
        menu_items: &ReadStorage<'_, MenuItem<I>>,
        menu_ec: &mut EventChannel<MenuEvent<I>>,
        menu_item_entity: Entity,
        control_action_event_data: ControlActionEventData,
    ) {
        let game_mode_selection_event = match control_action_event_data.control_action {
            ControlAction::Jump => Some(MenuEvent::Close),
            ControlAction::Attack => {
                let menu_item = menu_items
                    .get(menu_item_entity)
                    .expect("Expected `MenuItem` component to exist.");

                Some(MenuEvent::Select(menu_item.index))
            }
            _ => None,
        };

        if let Some(game_mode_selection_event) = game_mode_selection_event {
            debug!(
                "Sending game_mode selection event: {:?}",
                &game_mode_selection_event // kcov-ignore
            );
            menu_ec.single_write(game_mode_selection_event);
        }
    }
}

impl<'s, I> System<'s> for MenuItemWidgetInputSystem<I>
where
    I: Clone + Copy + Debug + PartialEq + Send + Sync + TypeNameTrait + 'static,
{
    type SystemData = MenuItemWidgetInputSystemData<'s, I>;

    fn run(
        &mut self,
        MenuItemWidgetInputSystemData {
            control_input_ec,
            mut menu_item_widget_input_resources,
        }: Self::SystemData,
    ) {
        let control_input_event_rid = self
            .control_input_event_rid
            .as_mut()
            .expect("Expected `control_input_event_rid` field to be set.");

        control_input_ec
            .read(control_input_event_rid)
            .for_each(|ev| {
                Self::handle_event(&mut menu_item_widget_input_resources, *ev);
            });
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.control_input_event_rid = Some(
            world
                .fetch_mut::<EventChannel<ControlInputEvent>>()
                .register_reader(),
        );
    }
}
