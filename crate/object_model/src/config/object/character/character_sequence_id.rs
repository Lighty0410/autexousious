use config::object::SequenceId;

/// Object Sequence IDs.
#[derive(Clone, Copy, Debug, Derivative, Deserialize, Eq, Hash, PartialEq)]
#[derivative(Default)]
#[serde(rename_all = "snake_case")]
pub enum CharacterSequenceId {
    /// Default sequence for characters.
    #[derivative(Default)]
    Stand,
    /// Walking sequence.
    Walk,
    /// Running sequence.
    Run,
    /// Running stop sequence.
    RunStop,
    /// Character is about to jump.
    Jump,
    /// Character has just jumped off the ground.
    JumpOff,
    /// Character is moving upwards from jumping.
    ///
    /// This is distinct from the `JumpDescend` state as this is when the jump velocity is
    /// effective, and characters may have different animations and attacks when moving upwards from
    /// a jump.
    JumpAscend,
    /// Character is descending from a jump.
    ///
    /// This sequence may also be used when the character has walked off a platform.
    JumpDescend,
    /// Character landed from jumping.
    JumpDescendLand,
}

impl SequenceId for CharacterSequenceId {}
