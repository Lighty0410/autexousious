use amethyst::{
    ecs::{World, WriteStorage},
    shred::{ResourceId, SystemData},
};
use character_model::{loaded::CharacterHitTransitions, play::RunCounter};
use charge_model::{
    config::{ChargeDelay, ChargeLimit, ChargeUseMode},
    play::{ChargeRetention, ChargeTrackerClock},
};
use derivative::Derivative;
use game_input_model::play::ControllerInput;
use map_model::play::MapBounded;
use object_model::{config::Mass, play::HealthPoints};
use object_status_model::config::StunPoints;

/// Character specific `Component` storages.
///
/// These are the storages for the components specific to character objects. See also
/// `ObjectComponentStorages`.
#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct CharacterComponentStorages<'s> {
    /// `ControllerInput` component storage.
    #[derivative(Debug = "ignore")]
    pub controller_inputs: WriteStorage<'s, ControllerInput>,
    /// `HealthPoints` component storage.
    #[derivative(Debug = "ignore")]
    pub health_pointses: WriteStorage<'s, HealthPoints>,
    /// `StunPoints` component storage.
    #[derivative(Debug = "ignore")]
    pub stun_pointses: WriteStorage<'s, StunPoints>,
    /// `RunCounter` component storage.
    #[derivative(Debug = "ignore")]
    pub run_counters: WriteStorage<'s, RunCounter>,
    /// `Mass` component storage.
    #[derivative(Debug = "ignore")]
    pub masses: WriteStorage<'s, Mass>,
    /// `MapBounded` component storage.
    #[derivative(Debug = "ignore")]
    pub map_boundeds: WriteStorage<'s, MapBounded>,
    /// `ChargeTrackerClock` components.
    #[derivative(Debug = "ignore")]
    pub charge_tracker_clocks: WriteStorage<'s, ChargeTrackerClock>,
    /// `ChargeLimit` components.
    #[derivative(Debug = "ignore")]
    pub charge_limits: WriteStorage<'s, ChargeLimit>,
    /// `ChargeDelay` components.
    #[derivative(Debug = "ignore")]
    pub charge_delays: WriteStorage<'s, ChargeDelay>,
    /// `ChargeUseMode` components.
    #[derivative(Debug = "ignore")]
    pub charge_use_modes: WriteStorage<'s, ChargeUseMode>,
    /// `ChargeRetention` components.
    #[derivative(Debug = "ignore")]
    pub charge_retentions: WriteStorage<'s, ChargeRetention>,
    /// `CharacterHitTransitions` components.
    #[derivative(Debug = "ignore")]
    pub character_hit_transitionses: WriteStorage<'s, CharacterHitTransitions>,
}
