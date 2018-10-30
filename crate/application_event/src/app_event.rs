use amethyst::{core::EventReader, ecs::*, shrev::EventChannel, StateEvent};
use character_selection_model::CharacterSelectionEvent;
use game_mode_selection_model::GameModeSelectionEvent;
use map_selection_model::MapSelectionEvent;

/// Type encompassing all state event types.
//
// === Non-Rustdoc === //
//
// Development note: Remember to update the `FromAppEvent` trait implementations when adding
// variants here.
#[derive(Clone, Derivative, Display, EnumDiscriminants, EventReader, From)]
#[derivative(Debug, PartialEq = "feature_allow_slow_enum")]
#[strum_discriminants(
    name(AppEventVariant),
    derive(Display, EnumIter, EnumString),
    strum(serialize_all = "snake_case")
)]
#[reader(AppEventReader)]
pub enum AppEvent {
    /// `character_selection` events.
    CharacterSelection(CharacterSelectionEvent),
    /// `game_mode_selection` events.
    GameModeSelection(GameModeSelectionEvent),
    /// `map_selection` events.
    MapSelection(MapSelectionEvent),
    /// Amethyst `State` events.
    State(#[derivative(Debug = "ignore", PartialEq = "ignore")] StateEvent),
}