use derivative::Derivative;
use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::config::{ControlTransition, SequenceId};

/// Sequence ID to transition to when a `ControlAction` is pressed, held, or released.
#[derive(Clone, Debug, Derivative, Deserialize, Hash, PartialEq, Eq, Serialize, new)]
#[derivative(Default)]
#[serde(deny_unknown_fields)]
pub struct ControlTransitions<SeqId, Extra = ()>
where
    SeqId: SequenceId,
{
    /// Sequence ID to transition to when `Defend` is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub press_defend: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Jump` is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub press_jump: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Attack` is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub press_attack: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Special` is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub press_special: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Defend` is held.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold_defend: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Jump` is held.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold_jump: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Attack` is held.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold_attack: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Special` is held.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold_special: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Defend` is released.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_defend: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Jump` is released.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_jump: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Attack` is released.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_attack: Option<ControlTransition<SeqId, Extra>>,
    /// Sequence ID to transition to when `Special` is released.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_special: Option<ControlTransition<SeqId, Extra>>,
}