use models::v2_0_0::{CardTarget, DerivedValue, PlayerTarget};

use crate::{GameTicks, GlobalCardId};

#[derive(Clone, Debug)]
pub enum SkipReason {
    IsFrozen,
}

#[derive(Clone, Debug)]
pub enum CombatEvent {
    Raw(String),
    Skip(SkipReason),
    DealDamage(PlayerTarget, u32, GlobalCardId),
    ApplyBurn(PlayerTarget, u32, GlobalCardId),
    ApplyPoison(PlayerTarget, u32, GlobalCardId),
    ApplyShield(PlayerTarget, DerivedValue<u32>, GlobalCardId),
    Freeze(CardTarget, GameTicks, GlobalCardId),
}

// This is useless, I baked in ownership into the simulator::Card - pretty good!!
// ok nvm mby its not
#[derive(Clone, Debug)]
pub struct TaggedCombatEvent(pub PlayerTarget, pub CombatEvent);
