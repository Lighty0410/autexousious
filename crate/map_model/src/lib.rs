#![deny(missing_docs)] // kcov-ignore
#![deny(missing_debug_implementations)]

//! Types used to represent maps and their configuration.
//!
//! This allows us to define the bounds of a map for a game, as well as image layers to render. In
//! contrast to `Object`s, an entity should be created for each map `Layer`.

extern crate amethyst;
#[macro_use]
extern crate derive_new;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate sprite_loading;
extern crate sprite_model;
#[cfg(test)]
extern crate toml;

pub mod config;
pub mod loaded;