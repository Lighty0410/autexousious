//! Types that represent processed configuration.
//!
//! This differs from the plain configuration types as they would have been processed into the form
//! that will be used in game.

pub use self::{body_sequence::BodySequence, interactions_sequence::InteractionsSequence};

mod body_sequence;
mod interactions_sequence;
