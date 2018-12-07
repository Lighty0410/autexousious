#![deny(missing_docs)]
// We do not deny missing_debug_implementations because the `lazy_static!` macro generates a
// non-debug implementation struct.

//! Provides built-in (compiled) assets and asset slugs.

extern crate game_model;
#[macro_use]
extern crate lazy_static;
extern crate map_model;

pub use crate::common::NAMESPACE_BUILT_IN;
pub use crate::map::{MAP_BLANK, MAP_BLANK_NAME, MAP_BLANK_SLUG};

mod common;
mod map;
