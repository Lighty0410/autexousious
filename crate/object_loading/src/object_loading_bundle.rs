use amethyst::{
    assets::Processor,
    core::bundle::{Result, SystemBundle},
    ecs::prelude::*,
};
use object_model::loaded::Character;

/// Adds `Processor::<`*`ObjectType`*`>` to the `World` with id
/// `"`*`object_type`*`_processor"`.
///
/// This is needed to allow the `loaded::*` types to be stored in `AssetStorage`.
#[derive(Debug, new)]
pub struct ObjectLoadingBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for ObjectLoadingBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(Processor::<Character>::new(), "character_processor", &[]);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use amethyst::assets::AssetStorage;
    use amethyst_test_support::AmethystApplication;
    use object_model::{loaded::Character, ObjectType};

    use super::ObjectLoadingBundle;

    #[test]
    fn bundle_build_adds_character_processor() {
        assert!(
            AmethystApplication::blank()
                .with_bundle(ObjectLoadingBundle)
                .with_assertion(|world| {
                    ObjectType::variants().iter().for_each(|object_type| {
                        match *object_type {
                            ObjectType::Character => {
                                // Next line will panic if the Processor wasn't added
                                world.read_resource::<AssetStorage<Character>>();
                            }
                        }
                    });
                })
                .run()
                .is_ok()
        );
    }
}