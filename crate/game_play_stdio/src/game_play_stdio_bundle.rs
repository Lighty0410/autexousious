use amethyst::{
    core::bundle::{Result, SystemBundle},
    ecs::prelude::*,
};
use application_event::AppEventVariant;
use stdio_spi::MapperSystem;
use typename::TypeName;

use crate::GamePlayEventStdinMapper;

/// Adds a `MapperSystem<GamePlayEventStdinMapper>` to the `World`.
#[derive(Debug, new)]
pub struct GamePlayStdioBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GamePlayStdioBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(
            MapperSystem::<GamePlayEventStdinMapper>::new(AppEventVariant::GamePlay),
            &MapperSystem::<GamePlayEventStdinMapper>::type_name(),
            &[],
        ); // kcov-ignore
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use amethyst::shrev::EventChannel;
    use amethyst_test::prelude::*;
    use game_play_model::GamePlayEvent;

    use super::GamePlayStdioBundle;

    #[test]
    fn bundle_should_add_mapper_system_to_dispatcher() {
        env::set_var("APP_DIR", env!("CARGO_MANIFEST_DIR"));
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AmethystApplication::blank()
                .with_bundle(GamePlayStdioBundle::new())
                // kcov-ignore-start
                .with_effect(|world| {
                    world.read_resource::<EventChannel<GamePlayEvent>>();
                })
                // kcov-ignore-end
                .run()
                .is_ok()
        );
    }
}