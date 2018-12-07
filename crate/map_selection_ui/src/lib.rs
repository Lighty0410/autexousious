#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! UI to allow the user to select the map.

extern crate amethyst;
#[cfg(test)]
extern crate amethyst_test;
#[cfg(test)]
extern crate application_test_support;
extern crate application_ui;
#[cfg(test)]
extern crate assets_test;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate derive_new;
extern crate game_input;
extern crate game_model;
#[macro_use]
extern crate log;
extern crate map_model;
extern crate map_selection;
extern crate map_selection_model;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate tracker;
extern crate typename;
#[macro_use]
extern crate typename_derive;

pub(crate) use crate::component::{MapSelectionWidget, WidgetState};
pub use crate::map_selection_ui_bundle::MapSelectionUiBundle;
pub(crate) use crate::system::{MapSelectionWidgetInputSystem, MapSelectionWidgetUiSystem};

mod component;
mod map_selection_ui_bundle;
mod system;
