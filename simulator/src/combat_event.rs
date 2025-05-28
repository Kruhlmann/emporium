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
    DealDamage(PlayerTarget, DerivedValue<u32>, GlobalCardId),
    ApplyBurn(PlayerTarget, DerivedValue<u32>, GlobalCardId),
    ApplyPoison(PlayerTarget, DerivedValue<u32>, GlobalCardId),
    ApplyShield(PlayerTarget, DerivedValue<u32>, GlobalCardId),
    Heal(PlayerTarget, DerivedValue<u32>, GlobalCardId),
    Freeze(CardTarget, GameTicks, GlobalCardId),
    Slow(CardTarget, GameTicks, GlobalCardId),
    Haste(CardTarget, GameTicks, GlobalCardId),
    Tick(u128),
}

#[derive(Clone, Debug)]
pub struct TaggedCombatEvent(pub PlayerTarget, pub CombatEvent);
