use std::{rc::Rc, time::Duration};

use models::v2_0_0::{Effect, Modifier, Percentage, PlayerTarget, TargetCondition, Tier, Tooltip};
use tracing::Level;

use crate::{CombatEvent, GameTicks, GlobalCardId, SkipReason};

#[derive(Clone, Debug)]
pub struct Card {
    pub id_for_simulation: GlobalCardId,
    pub inner: models::v2_0_0::Card,
    pub tier: Tier,
    pub cooldown_effects: Vec<Effect>,
    pub cooldown_counter: u128,
    pub freeze_ticks: GameTicks,
    pub slow_ticks: GameTicks,
    pub haste_ticks: GameTicks,
    pub position: u8,
    pub owner: PlayerTarget,
    pub cooldown: GameTicks,
    pub modification_tooltips: Vec<Tooltip>,

    pub freeze_guard: Option<Rc<tracing::span::EnteredSpan>>,
    pub slow_guard: Option<Rc<tracing::span::EnteredSpan>>,
    pub haste_guard: Option<Rc<tracing::span::EnteredSpan>>,
}

impl Card {
    pub fn tick(&mut self) -> Vec<CombatEvent> {
        let cooldown_increment = match (self.slow_ticks.0 > 0, self.haste_ticks.0 > 0) {
            (true, false) => 1, // 0.5 * 2
            (false, true) => 4, // 2.0 * 2
            _ => 2,             // 1.0 * 2 (covers both haste/slow or neither)
        };

        if self.freeze_ticks.0 > 0 {
            self.freeze_ticks.0 -= 1;
            if self.freeze_ticks.0 == 0 {
                let _ = self.freeze_guard.take();
            }
            return vec![CombatEvent::Skip(SkipReason::IsFrozen)];
        }
        if self.haste_ticks.0 > 0 {
            self.haste_ticks.0 -= 1;
            if self.haste_ticks.0 == 0 {
                let _ = self.haste_guard.take();
            }
        }
        if self.slow_ticks.0 > 0 {
            panic!("AA");
            self.slow_ticks.0 -= 1;
            if self.slow_ticks.0 == 0 {
                let _ = self.slow_guard.take();
            }
        }

        let mut events: Vec<CombatEvent> = Vec::new();
        if self.cooldown > GameTicks(0) {
            let threshold = self.cooldown.0 * 2;
            while self.cooldown_counter > threshold {
                self.cooldown_counter -= threshold;
                for effect in &self.cooldown_effects {
                    let mut combat_events: Vec<CombatEvent> =
                        self.effect_to_combat_events(effect.clone());
                    events.append(&mut combat_events);
                }
            }
        }
        self.cooldown_counter += cooldown_increment;
        events
    }

    pub fn compute_cost(&self) -> u32 {
        let base_cost = self.tier.scale_cost(self.inner.size.base_cost());
        let todo = true; //TODO check fi this applies
        // Tooltip::ConditionalModifier(condition, Modifier) => todo!(),
        let todo = true; //TODO check if this applies
        // Tooltip::SellsForGold => todo!(),
        let modification_cost = self
            .tier
            .select(&self.inner.tiers)
            .iter()
            .filter_map(|t| match t {
                Tooltip::StaticModifier(Modifier::IncreasedValue(v)) => Some(*v),
                _ => None,
            })
            .sum::<u32>();
        base_cost + modification_cost
    }
    #[tracing::instrument]
    pub fn freeze(&mut self, duration: GameTicks) {
        tracing::event!(
            name: "freeze item",
            Level::INFO,
            id = ?self.id_for_simulation,
            ?duration,
        );
        let span = tracing::info_span!("frozen", id = %self.id_for_simulation);
        self.freeze_guard = Some(Rc::new(span.entered()));
        self.freeze_ticks += duration
    }

    pub fn slow(&mut self, duration: GameTicks) {
        tracing::event!(
            name: "slow item",
            Level::INFO,
            id = ?self.id_for_simulation,
            ?duration,
        );
        let span = tracing::info_span!("slowed", id = %self.id_for_simulation);
        self.slow_guard = Some(Rc::new(span.entered()));
        self.slow_ticks += duration
    }

    pub fn haste(&mut self, duration: GameTicks) {
        tracing::event!(
            name: "haste item",
            Level::INFO,
            id = ?self.id_for_simulation,
            ?duration,
        );
        let span = tracing::info_span!("hasted", id = %self.id_for_simulation);
        self.haste_guard = Some(Rc::new(span.entered()));
        self.haste_ticks += duration
    }

    pub fn matches(&self, condition: &TargetCondition, target_candidate: Option<&Card>) -> bool {
        match condition {
            TargetCondition::Always => true,
            TargetCondition::Never => false,
            TargetCondition::HasCooldown => self.cooldown > GameTicks(0),
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
            TargetCondition::Raw(condition) => {
                tracing::event!(name: "raw condition", Level::WARN, ?condition);
                false
            }
            TargetCondition::NameIncludes(s) => target_candidate
                .map(|t| t.inner.name.to_lowercase().contains(&s.to_lowercase()))
                .unwrap_or(false),
        }
    }

    pub fn compute_crit_chance(&self) -> Percentage {
        let fraction = self
            .tier
            .select(&self.inner.tiers)
            .iter()
            .filter_map(|t| match t {
                Tooltip::StaticModifier(Modifier::CritChance(c)) => Some(c.as_fraction()),
                _ => None,
            })
            .sum::<f64>();
        Percentage::from_fraction(fraction)
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
            Effect::Heal(player_target, heal) => {
                vec![CombatEvent::Heal(
                    player_target,
                    heal,
                    self.id_for_simulation,
                )]
            }
            Effect::Shield(player_target, shield) => {
                vec![CombatEvent::ApplyShield(
                    player_target,
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
            Effect::Slow(target, duration_seconds) => {
                let duration: GameTicks = Duration::from_secs_f64(duration_seconds).into();
                vec![CombatEvent::Slow(target, duration, self.id_for_simulation)]
            }
            Effect::Haste(target, duration_seconds) => {
                let duration: GameTicks = Duration::from_secs_f64(duration_seconds).into();
                vec![CombatEvent::Haste(target, duration, self.id_for_simulation)]
            }
            _ => {
                tracing::event!(Level::ERROR, ?value, "effect could not become combatevent");
                vec![CombatEvent::Raw(format!("{value}"))]
            }
        }
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.id_for_simulation == other.id_for_simulation
    }
}

impl Eq for Card {}
