#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! Extension to enable `map_selection` to be controlled by stdio.

pub use crate::{
    map_selection_event_args::MapSelectionEventArgs,
    map_selection_event_stdin_mapper::MapSelectionEventStdinMapper,
    map_selection_stdio_bundle::MapSelectionStdioBundle,
};

mod map_selection_event_args;
mod map_selection_event_stdin_mapper;
mod map_selection_stdio_bundle;
