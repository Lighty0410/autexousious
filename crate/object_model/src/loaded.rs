//! Contains the types that represent processed configuration.
//!
//! This differs from the plain configuration types as they would have been processed into the form
//! that will be used in game.

pub use self::{
    animation::{AnimatedComponentAnimation, AnimatedComponentDefault},
    character::{Character, CharacterHandle},
    object::{Object, ObjectHandle},
    sequence::{SequenceEndTransition, SequenceEndTransitions},
};

mod animation;
mod character;
mod object;
mod sequence;