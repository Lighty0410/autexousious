use std::collections::HashMap;

use amethyst::audio::SourceHandle;
use derive_deref::{Deref, DerefMut};
use derive_new::new;

use crate::config::UiSfxId;

/// Map of `UiSfxId` to the loaded SFX data.
#[derive(Debug, Default, Deref, DerefMut, PartialEq, Eq, new)]
pub struct UiSfxMap(HashMap<UiSfxId, SourceHandle>);
