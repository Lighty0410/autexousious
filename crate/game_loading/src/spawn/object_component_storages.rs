use amethyst::{
    animation::AnimationControlSet,
    core::Transform,
    ecs::prelude::*,
    renderer::{SpriteRender, Transparent},
};
use object_model::entity::Kinematics;

/// Common game object `Component` storages.
///
/// These are the storages for the components common to all game objects.
pub type ObjectComponentStorages<'s, SeqId> = (
    WriteStorage<'s, SpriteRender>,
    WriteStorage<'s, Transparent>,
    WriteStorage<'s, Kinematics<f32>>,
    WriteStorage<'s, Transform>,
    WriteStorage<'s, AnimationControlSet<SeqId, SpriteRender>>,
);