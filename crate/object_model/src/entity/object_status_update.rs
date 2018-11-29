use config::object::SequenceId;
use entity::SequenceStatus;

/// Indicates what fields of an `ObjectStatus` should be updated.
// TODO: Learning exercise - Generate this using a proc macro
// See <https://crates.io/crates/optional_struct>
#[derive(Default, Debug, PartialEq, new)]
pub struct ObjectStatusUpdate<SeqId: SequenceId> {
    /// ID of the current sequence the entity is on.
    pub sequence_id: Option<SeqId>,
}
