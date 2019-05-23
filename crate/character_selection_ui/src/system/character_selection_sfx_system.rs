use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    ecs::{Read, Resources, System, SystemData},
    shrev::{EventChannel, ReaderId},
};
use character_selection_model::CharacterSelectionEvent;
use derive_new::new;
use typename_derive::TypeName;
use ui_audio_model::{config::UiSfxId, loaded::UiSfxMap};

/// Default volume to play sounds at.
const VOLUME: f32 = 1.0;

/// Plays sounds for the character selection UI.
#[derive(Debug, Default, TypeName, new)]
pub struct CharacterSelectionSfxSystem {
    /// Reader ID for the `CharacterSelectionEvent` event channel.
    #[new(default)]
    character_selection_event_rid: Option<ReaderId<CharacterSelectionEvent>>,
}

type CharacterSelectionSfxSystemData<'s> = (
    Read<'s, EventChannel<CharacterSelectionEvent>>,
    Read<'s, UiSfxMap>,
    Read<'s, AssetStorage<Source>>,
    Option<Read<'s, Output>>,
);

impl<'s> System<'s> for CharacterSelectionSfxSystem {
    type SystemData = CharacterSelectionSfxSystemData<'s>;

    fn run(
        &mut self,
        (character_selection_ec, ui_sfx_map, source_assets, output): Self::SystemData,
    ) {
        // Make sure we empty the event channel, even if we don't have an output device.
        let events_iterator = character_selection_ec.read(
            self.character_selection_event_rid
                .as_mut()
                .expect("Expected reader ID to exist for CharacterSelectionSfxSystem."),
        );

        if let Some(output) = output {
            events_iterator.for_each(|ev| {
                let ui_sfx_id = match ev {
                    CharacterSelectionEvent::Return => UiSfxId::Cancel,
                    CharacterSelectionEvent::Join { .. } => UiSfxId::Select,
                    CharacterSelectionEvent::Switch { .. } => UiSfxId::Switch,
                    CharacterSelectionEvent::Select { .. } => UiSfxId::Select,
                    CharacterSelectionEvent::Deselect { .. } => UiSfxId::Deselect,
                    CharacterSelectionEvent::Leave { .. } => UiSfxId::Deselect,
                    CharacterSelectionEvent::Confirm => UiSfxId::Confirm,
                };

                ui_sfx_map
                    .get(&ui_sfx_id)
                    .and_then(|ui_sfx_handle| source_assets.get(ui_sfx_handle))
                    .map(|ui_sfx| output.play_once(ui_sfx, VOLUME));
            });
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.character_selection_event_rid = Some(
            res.fetch_mut::<EventChannel<CharacterSelectionEvent>>()
                .register_reader(),
        );
    }
}

#[cfg(test)]
mod tests {
    use amethyst::{ecs::World, shrev::EventChannel, Error};
    use application_test_support::AutexousiousApplication;
    use asset_model::loaded::SlugAndHandle;
    use assets_test::ASSETS_CHAR_BAT_SLUG;
    use character_selection_model::{CharacterSelection, CharacterSelectionEvent};

    use super::CharacterSelectionSfxSystem;

    #[test]
    fn plays_sound_on_return_event() -> Result<(), Error> {
        run_test("plays_sound_on_return_event", |_world| {
            CharacterSelectionEvent::Return
        })
    }

    #[test]
    fn plays_sound_on_join_event() -> Result<(), Error> {
        run_test("plays_sound_on_join_event", |_world| {
            CharacterSelectionEvent::Join { controller_id: 123 }
        })
    }

    #[test]
    fn plays_sound_on_switch_event() -> Result<(), Error> {
        run_test("plays_sound_on_witch_event", |world| {
            let snh = SlugAndHandle::from((&*world, ASSETS_CHAR_BAT_SLUG.clone()));
            let character_selection = CharacterSelection::Id(snh);
            CharacterSelectionEvent::Switch {
                controller_id: 123,
                character_selection,
            }
        })
    }

    #[test]
    fn plays_sound_on_select_event() -> Result<(), Error> {
        run_test("plays_sound_on_select_event", |world| {
            let snh = SlugAndHandle::from((&*world, ASSETS_CHAR_BAT_SLUG.clone()));
            let character_selection = CharacterSelection::Id(snh);
            CharacterSelectionEvent::Select {
                controller_id: 123,
                character_selection,
            }
        })
    }

    #[test]
    fn plays_sound_on_deselect_event() -> Result<(), Error> {
        run_test("plays_sound_on_deselect_event", |_world| {
            CharacterSelectionEvent::Deselect { controller_id: 123 }
        })
    }

    #[test]
    fn plays_sound_on_leave_event() -> Result<(), Error> {
        run_test("plays_sound_on_leave_event", |_world| {
            CharacterSelectionEvent::Leave { controller_id: 123 }
        })
    }

    #[test]
    fn plays_sound_on_confirm_event() -> Result<(), Error> {
        run_test("plays_sound_on_confirm_event", |_world| {
            CharacterSelectionEvent::Confirm
        })
    }

    fn run_test<F>(test_name: &str, event_fn: F) -> Result<(), Error>
    where
        F: Fn(&mut World) -> CharacterSelectionEvent + Send + Sync + 'static,
    {
        AutexousiousApplication::config_base(test_name, false)
            .with_system(CharacterSelectionSfxSystem::new(), "", &[])
            .with_effect(move |world| {
                let event = event_fn(world);
                send_event(world, event);
            })
            .with_assertion(|_world| {})
            .run()
    }

    fn send_event(world: &mut World, event: CharacterSelectionEvent) {
        let mut ec = world.write_resource::<EventChannel<CharacterSelectionEvent>>();
        ec.single_write(event)
    } // kcov-ignore
}