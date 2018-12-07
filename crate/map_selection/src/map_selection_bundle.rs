use amethyst::{
    core::bundle::{Result, SystemBundle},
    ecs::prelude::*,
};
use typename::TypeName;

use crate::MapSelectionSystem;

/// Adds the `MapSelectionSystem` to the `World`.
#[derive(Debug, new)]
pub struct MapSelectionBundle {
    /// System names that the `MapSelectionSystem` should depend on.
    #[new(default)]
    system_dependencies: Option<Vec<String>>,
}

impl MapSelectionBundle {
    /// Specifies system dependencies for the `MapSelectionSystem`.
    ///
    /// # Parameters
    ///
    /// * `dependencies`: Names of the systems to depend on.
    pub fn with_system_dependencies(mut self, dependencies: &[String]) -> Self {
        self.system_dependencies = Some(Vec::from(dependencies));
        self
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for MapSelectionBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        let deps = self
            .system_dependencies
            .as_ref()
            .map_or_else(Vec::new, |deps| {
                deps.iter().map(|dep| dep.as_ref()).collect::<Vec<&str>>()
            });

        builder.add(
            MapSelectionSystem::new(),
            &MapSelectionSystem::type_name(),
            &deps,
        ); // kcov-ignore

        Ok(())
    }
}

// TODO: Custom state dispatcher with bundles.
// See <https://gitlab.com/azriel91/autexousious/issues/74>.
