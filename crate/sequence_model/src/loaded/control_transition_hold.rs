use derive_new::new;
use game_input_model::ControlAction;

use crate::config::SequenceId;

/// Transition to a specified sequence on control input enabled state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, new)]
pub struct ControlTransitionHold<SeqId>
where
    SeqId: SequenceId,
{
    /// Control button that this transition applies to.
    pub action: ControlAction,
    /// ID of the sequence to switch to after this one has completed.
    pub next: SeqId,
}
