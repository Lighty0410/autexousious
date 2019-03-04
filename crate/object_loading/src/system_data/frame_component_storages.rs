use amethyst::{assets::Handle, ecs::WriteStorage, renderer::SpriteRender};
use collision_model::config::{Body, Interactions};
use derivative::Derivative;
use sequence_model::config::Wait;
use shred_derive::SystemData;

/// `Component` storages for components that change per sequence frame.
#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct FrameComponentStorages<'s> {
    /// `Wait` component storage.
    #[derivative(Debug = "ignore")]
    pub waits: WriteStorage<'s, Wait>,
    /// `SpriteRender` component storage.
    #[derivative(Debug = "ignore")]
    pub sprite_renders: WriteStorage<'s, SpriteRender>,
    /// `Body` component storage.
    #[derivative(Debug = "ignore")]
    pub bodies: WriteStorage<'s, Handle<Body>>,
    /// `Interactions` component storage.
    #[derivative(Debug = "ignore")]
    pub interactionses: WriteStorage<'s, Handle<Interactions>>,
}
