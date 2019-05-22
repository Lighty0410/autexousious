use amethyst::{
    assets::AssetStorage,
    core::transform::Transform,
    ecs::{Join, Read, ReadStorage, System, WriteStorage},
    renderer::{SpriteRender, SpriteSheet},
};
use derive_new::new;
use object_model::play::{Mirrored, Position};
use typename_derive::TypeName;

/// Updates each entity's `Transform` based on their `Position` in game.
///
/// This system should be run after all other systems that affect kinematics have run.
#[derive(Debug, Default, TypeName, new)]
pub(crate) struct ObjectTransformUpdateSystem;

type ObjectTransformUpdateSystemData<'s> = (
    ReadStorage<'s, Position<f32>>,
    ReadStorage<'s, Mirrored>,
    ReadStorage<'s, SpriteRender>,
    Read<'s, AssetStorage<SpriteSheet>>,
    WriteStorage<'s, Transform>,
);

impl<'s> System<'s> for ObjectTransformUpdateSystem {
    type SystemData = ObjectTransformUpdateSystemData<'s>;

    fn run(
        &mut self,
        (positions, mirroreds, sprite_renders, sprite_sheet_assets, mut transform_storage): Self::SystemData,
    ) {
        for (position, mirrored, sprite_render, transform) in (
            &positions,
            mirroreds.maybe(),
            sprite_renders.maybe(),
            &mut transform_storage,
        )
            .join()
        {
            // Hack: Visual correction when sprites are mirrored.
            if let (Some(mirrored), Some(sprite_render)) = (mirrored, sprite_render) {
                if mirrored.0 {
                    let sprite_sheet = sprite_sheet_assets
                        .get(&sprite_render.sprite_sheet)
                        .expect("Expected sprite sheet to be loaded.");
                    let sprite = &sprite_sheet.sprites[sprite_render.sprite_number];
                    transform.set_translation_x(position.x + sprite.offsets[0]);
                } else {
                    transform.set_translation_x(position.x);
                }
            } else {
                transform.set_translation_x(position.x);
            }

            // We subtract z from the y translation as the z axis increases "out of the screen".
            // Entities that have a larger Z value are transformed downwards.
            transform.set_translation_y(position.y - position.z);
            transform.set_translation_z(position.z);
        }
    }
}

#[cfg(test)]
mod test {
    use amethyst::{
        core::{math::Vector3, transform::Transform, Float},
        ecs::{Builder, Entity, World},
        Error,
    };
    use amethyst_test::AmethystApplication;
    use object_model::play::Position;
    use typename::TypeName;

    use super::ObjectTransformUpdateSystem;

    #[test]
    fn updates_transform_with_x_and_yz() -> Result<(), Error> {
        let setup = |world: &mut World| {
            // Create entity with position
            let position = Position::<f32>::new(-5., -3., -4.);

            let mut transform = Transform::default();
            transform.set_translation(Vector3::new(
                Float::from(10.),
                Float::from(20.),
                Float::from(0.),
            ));

            let entity = world.create_entity().with(position).with(transform).build();

            world.add_resource(entity);
        };

        let assertion = |world: &mut World| {
            let entity = *world.read_resource::<Entity>();
            let transforms = world.read_storage::<Transform>();

            let expected_translation =
                Vector3::new(Float::from(-5.), Float::from(1.), Float::from(-4.));

            let transform = transforms
                .get(entity)
                .expect("Expected entity to have `Transform` component.");
            assert_eq!(&expected_translation, transform.translation());
        };

        AmethystApplication::ui_base::<String, String>()
            .with_system(
                ObjectTransformUpdateSystem::new(),
                ObjectTransformUpdateSystem::type_name(),
                &[],
            ) // kcov-ignore
            .with_setup(setup)
            .with_assertion(assertion)
            .run()
    }
}
