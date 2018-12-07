#![deny(missing_docs)]
// We do not deny missing_debug_implementations because the `lazy_static!` macro generates a
// non-debug implementation struct, and that macro is used widely throughout this crate.

//! Provides assets for testing and asset slugs.
//!
//! **WARNING:** This crate is intended strictly for testing, and should not be used in any
//! non-`test` code. This is because it exports (and hence exposes) the path of this crate's
//! directory on the machine it is compiled on.
//!
//! For assets that should be compiled into the executable, please use the `assets_built_in` crate.

extern crate game_model;
extern crate heck;
#[macro_use]
extern crate lazy_static;
extern crate object_model;
extern crate strum;

pub use crate::common::{ASSETS_PATH, NAMESPACE_TEST, NAMESPACE_TEST_PATH};
pub use crate::map::{
    ASSETS_MAP_EMPTY_NAME, ASSETS_MAP_EMPTY_PATH, ASSETS_MAP_EMPTY_SLUG, ASSETS_MAP_FADE_NAME,
    ASSETS_MAP_FADE_PATH, ASSETS_MAP_FADE_SLUG,
};
pub use crate::object::{
    ASSETS_CHAR_BAT_NAME, ASSETS_CHAR_BAT_PATH, ASSETS_CHAR_BAT_SLUG,
    ASSETS_CHAR_BAT_SPRITE_BROWN_NAME, ASSETS_CHAR_BAT_SPRITE_GREY_NAME, ASSETS_OBJECT_PATH,
};

mod common;
mod map;
mod object;
