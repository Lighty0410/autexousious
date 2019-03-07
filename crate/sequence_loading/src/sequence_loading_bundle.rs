use amethyst::{assets::Processor, core::bundle::SystemBundle, ecs::DispatcherBuilder, Error};
use derive_new::new;
use sequence_model::loaded::ComponentSequences;

/// Adds the following processor `System`s to the world:
///
/// * `Processor::<ComponentSequences>`
#[derive(Debug, new)]
pub struct SequenceLoadingBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for SequenceLoadingBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            Processor::<ComponentSequences>::new(),
            "component_sequences_processor",
            &[],
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use amethyst::{assets::AssetStorage, Error};
    use amethyst_test::AmethystApplication;
    use sequence_model::loaded::ComponentSequences;

    use super::SequenceLoadingBundle;

    #[test]
    fn bundle_build_adds_sequence_processor() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_bundle(SequenceLoadingBundle)
            .with_assertion(|world| {
                // Panics if the Processors are not added.
                world.read_resource::<AssetStorage<ComponentSequences>>();
            })
            .run()
    }
}