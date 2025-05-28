use std::time::Duration;

use models::v2_0_0::{Effect, Percentage, PlayerTarget, TargetCondition, Tier};

use crate::{CombatEvent, GameTicks, SkipReason};

use super::{CardModification, GlobalCardId};

#[derive(Clone, Debug)]
pub struct Card {
    pub id_for_simulation: GlobalCardId,
    pub inner: models::v2_0_0::Card,
    pub modifications: Vec<CardModification>,
    pub tier: Tier,
    pub cooldown_effects: Vec<Effect>,
    pub cooldown: GameTicks,
    pub cooldown_counter: u128,
    pub freeze_ticks: GameTicks,
    pub slow_ticks: GameTicks,
    pub position: u8,
    pub owner: PlayerTarget,
    pub crit_chance: Percentage,
}

impl Card {
    #[inline(always)]
    pub fn tick(&mut self) -> Vec<CombatEvent> {
        if self.freeze_ticks.0 > 0 {
            self.freeze_ticks.0 -= 1;
            return vec![CombatEvent::Skip(SkipReason::IsFrozen)];
        }
        let mut events: Vec<CombatEvent> = Vec::new();
        if self.cooldown.0 > 0 {
            if self.cooldown_counter % self.cooldown.0 == 0 {
                for effect in &self.cooldown_effects {
                    let mut effect_events: Vec<CombatEvent> =
                        self.effect_to_combat_events(effect.clone());
                    events.append(&mut effect_events);
                }
            }
        }
        self.cooldown_counter += 1;
        events
    }

    pub fn freeze(&mut self, duration: GameTicks) {
        self.freeze_ticks = duration
    }

    pub fn matches(&self, condition: &TargetCondition, target_candidate: Option<&Card>) -> bool {
        match condition {
            TargetCondition::Always => true,
            TargetCondition::Never => false,
            TargetCondition::HasCooldown => target_candidate
                .map(|t| t.cooldown > GameTicks(0))
                .unwrap_or(false),
            TargetCondition::Adjacent => target_candidate
                .map(|t| self.owner == t.owner && self.position.abs_diff(t.position) == 1)
                .unwrap_or(false),
            TargetCondition::IsSelf => target_candidate.map(|t| self == t).unwrap_or(false),
            TargetCondition::HasOwner(condition_owner) => match self.owner {
                PlayerTarget::Player => target_candidate
                    .map(|t| t.owner == *condition_owner)
                    .unwrap_or(false),
                PlayerTarget::Opponent => target_candidate
                    .map(|t| t.owner.inverse() == *condition_owner)
                    .unwrap_or(false),
            },
            TargetCondition::HasTag(tag) => self.inner.tags.iter().find(|t| *t == tag).is_some(),
            TargetCondition::HasSize(size) => self.inner.size == *size,
            TargetCondition::And(a, b) => {
                self.matches(a, target_candidate) && self.matches(b, target_candidate)
            }
            TargetCondition::Or(a, b) => {
                self.matches(a, target_candidate) || self.matches(b, target_candidate)
            }
            TargetCondition::Not(a) => !self.matches(a, target_candidate),
            TargetCondition::Raw(s) => {
                eprintln!("skipping raw target condition: '{s}'");
                false
            }
            TargetCondition::NameIncludes(s) => target_candidate
                .map(|t| t.inner.name.to_lowercase().contains(&s.to_lowercase()))
                .unwrap_or(false),
        }
    }

    pub fn effect_to_combat_events(&self, value: Effect) -> Vec<CombatEvent> {
        match value {
            Effect::DealDamage(player_target, damage) => {
                vec![CombatEvent::DealDamage(
                    player_target,
                    damage,
                    self.id_for_simulation,
                )]
            }
            Effect::Burn(player_target, burn) => {
                vec![CombatEvent::ApplyBurn(
                    player_target,
                    burn,
                    self.id_for_simulation,
                )]
            }
            Effect::Poison(player_target, poison) => {
                vec![CombatEvent::ApplyPoison(
                    player_target,
                    poison,
                    self.id_for_simulation,
                )]
            }
            Effect::Shield(player_target, shield) => {
                vec![CombatEvent::ApplyShield(
                    player_target.inverse(),
                    shield,
                    self.id_for_simulation,
                )]
            }
            Effect::Freeze(target, duration_seconds) => {
                let duration: GameTicks = Duration::from_secs_f64(duration_seconds).into();
                vec![CombatEvent::Freeze(
                    target,
                    duration,
                    self.id_for_simulation,
                )]
            }
            _ => vec![CombatEvent::Raw(format!("{value}"))],
        }
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.id_for_simulation == other.id_for_simulation
    }
}

impl Eq for Card {}
