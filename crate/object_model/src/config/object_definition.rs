use std::collections::HashMap;

use derivative::Derivative;
use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::config::GameObjectSequence;

/// Contains all of the sequences for an `Object`.
///
/// This type is not intended to be instantiated by consumers directly. Instead, consumers should
/// instante the various definition types for each object type, such as [`CharacterDefinition`]
/// [char_definition] for characters.
///
/// [char_definition]: ../character/struct.CharacterDefinition.html
#[derive(Clone, Debug, Derivative, Deserialize, PartialEq, Serialize, new)]
#[derivative(Default(bound = ""))] // Don't require `ObjSeq: Default`
pub struct ObjectDefinition<ObjSeq>
where
    ObjSeq: GameObjectSequence,
    ObjSeq::SequenceId: for<'des> Deserialize<'des> + Serialize,
{
    /// Sequences of actions this object can perform.
    pub sequences: HashMap<ObjSeq::SequenceId, ObjSeq>,
}
