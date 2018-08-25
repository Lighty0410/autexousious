use amethyst::ecs::prelude::{Component, DenseVecStorage};

use CharacterSelection;
use WidgetState;

/// Component to tag entities that are character selection widgets.
#[derive(Copy, Clone, Debug, PartialEq, Eq, new)]
pub(crate) struct CharacterSelectionWidget {
    /// Character selection state.
    pub state: WidgetState,
    /// Selected character ID or random.
    pub selection: CharacterSelection,
}

impl Component for CharacterSelectionWidget {
    type Storage = DenseVecStorage<Self>;
}
