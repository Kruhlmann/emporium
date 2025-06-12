use std::time::Instant;

use indexmap::IndexMap;
use models::v2_0_0::{
    CardDerivedProperty, DerivedValue, Effect, Modifier, PlayerTarget, TargetCondition, Tooltip,
};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use tracing::Level;

use crate::{
    Card, CardSummary, CombatEvent, DispatchableEvent, GameTicks, GlobalCardId, Player,
    SimulationDrawType, SimulationResult, SimulationResultInner, SimulationTemplate,
    TaggedCombatEvent, NUMBER_OF_BOARD_SPACES, SIMULATION_TICK_COUNT,
};

#[derive(Clone, Debug)]
pub struct Simulation {
    pub player: Player,
    pub opponent: Player,
    pub event_sender: Option<std::sync::mpsc::Sender<DispatchableEvent>>,
    pub cards: IndexMap<GlobalCardId, Card>,
    pub ticks: u128,
}

impl TryFrom<SimulationTemplate> for Simulation {
    type Error = anyhow::Error;

    fn try_from(template: SimulationTemplate) -> Result<Self, Self::Error> {
        let mut position: u8 = 0;
        let mut player_cards: IndexMap<GlobalCardId, Card> = IndexMap::new();
        for template in &template.player.card_templates {
            let id = GlobalCardId::default();
            let card: Card = template
                .create_card_on_board(position, PlayerTarget::Player, id)
                .map_err(|error| {
                    anyhow::anyhow!("unable to parse player card template {template:?}: {error}")
                })?;
            position += card.inner.size.board_spaces();
            player_cards.insert(id, card);
        }

        if position > *NUMBER_OF_BOARD_SPACES {
            anyhow::bail!("player board too large ({position})")
        }

        let mut position: u8 = 0;
        let mut opponent_cards: IndexMap<GlobalCardId, Card> = IndexMap::new();
        for template in &template.opponent.card_templates {
            let id = GlobalCardId::default();
            let card: Card = template
                .create_card_on_board(position, PlayerTarget::Opponent, id)
                .map_err(|error| {
                    anyhow::anyhow!("unable to parse opponent card template {template:?}: {error}")
                })?;
            position += card.inner.size.board_spaces();
            opponent_cards.insert(id, card);
        }

        if position > *NUMBER_OF_BOARD_SPACES {
            anyhow::bail!("player board too large ({position})")
        }

        let player_card_ids: Vec<GlobalCardId> = player_cards.keys().cloned().collect();
        let opponent_card_ids: Vec<GlobalCardId> = opponent_cards.keys().cloned().collect();
        let mut cards = player_cards;
        cards.extend(opponent_cards.into_iter());

        Ok(Self {
            cards,
            player: template.player.create_player(player_card_ids)?,
            opponent: template.opponent.create_player(opponent_card_ids)?,
            event_sender: None,
            ticks: 0,
        })
    }
}

impl Simulation {
    pub fn get_cards_by_owner(&self, owner: PlayerTarget) -> &Vec<GlobalCardId> {
        match owner {
            PlayerTarget::Player => &self.player.card_ids,
            PlayerTarget::Opponent => &self.opponent.card_ids,
        }
    }

    pub fn get_cards_by_target(
        &self,
        source_id: &GlobalCardId,
        condition: &TargetCondition,
    ) -> Vec<GlobalCardId> {
        let Some(source_card) = self.cards.get(source_id) else {
            self.dispatch_event(&DispatchableEvent::Warning(format!(
                "failed to get card with id {source_id} from {:?}",
                self.cards.keys()
            )));
            return vec![];
        };
        if cfg!(debug_assertions) {
            let cards: Vec<GlobalCardId> = self
                .cards
                .iter()
                .by_ref()
                .filter(|(_, c)| source_card.matches(condition, Some(c)))
                .map(|(id, _)| id)
                .cloned()
                .collect();
            cards
        } else {
            self.cards
                .iter()
                .by_ref()
                .filter(|(_, c)| source_card.matches(condition, Some(c)))
                .map(|(id, _)| id)
                .cloned()
                .collect()
        }
    }

    pub fn with_channel(mut self, sender: std::sync::mpsc::Sender<DispatchableEvent>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    fn dispatch_log(&self, s: String) {
        let event = &DispatchableEvent::Log(s);
        self.dispatch_event(event)
    }

    fn dispatch_event(&self, event: &DispatchableEvent) {
        tracing::event!(name: "event dispatch", Level::INFO, ?event);
        if let Some(ref tx) = self.event_sender {
            let _ = tx.send(event.clone());
        }
    }

    fn tick(&mut self) -> Vec<TaggedCombatEvent> {
        let mut events: Vec<TaggedCombatEvent> = Vec::new();
        tracing::info_span!("player tick").in_scope(|| self.player.tick());
        tracing::info_span!("opponent tick").in_scope(|| self.opponent.tick());
        for (_, card) in &mut self.cards {
            for e in card.tick() {
                events.push(TaggedCombatEvent(card.owner, e));
            }
        }
        events.push(TaggedCombatEvent(
            PlayerTarget::Player,
            CombatEvent::Tick(self.ticks),
        ));
        self.ticks += 1;
        events
    }

    fn apply_event(&mut self, event: &TaggedCombatEvent, rng: &mut StdRng) -> anyhow::Result<()> {
        match event {
            TaggedCombatEvent(.., CombatEvent::Skip(..)) => {}
            TaggedCombatEvent(.., CombatEvent::Raw(s)) => {
                self.dispatch_event(&DispatchableEvent::Warning(format!(
                    "raw event skipped: {s}"
                )));
            }
            TaggedCombatEvent(
                owner,
                CombatEvent::DealDamage(player_target, damage_derivable, source_id),
            ) => {
                if let Some(card) = self.cards.get(source_id) {
                    let did_crit = card.compute_crit_chance().as_fraction() < rng.random::<f64>();
                    let damage = match damage_derivable {
                        DerivedValue::Constant(p) => *p,
                        _ => self.derive_value(damage_derivable.clone(), source_id)? as u32,
                    };
                    let damage = if did_crit {
                        let todo = true; //TODO what about increased crit dmg
                        damage + damage
                    } else {
                        damage
                    };
                    match owner == player_target {
                        true => self.player.take_damage(damage),
                        false => self.opponent.take_damage(damage),
                    }
                } else {
                    let todo = true; //TODO else what?
                }
            }
            TaggedCombatEvent(
                owner,
                CombatEvent::ApplyBurn(player_target, burn_derivable, source_id),
            ) => {
                let todo = true; //TODO burn crit
                let burn = match burn_derivable {
                    DerivedValue::Constant(b) => *b,
                    _ => self.derive_value(burn_derivable.clone(), source_id)? as u32,
                };
                match owner == player_target {
                    true => self.player.burn(burn),
                    false => self.opponent.burn(burn),
                }
            }
            TaggedCombatEvent(
                owner,
                CombatEvent::ApplyPoison(player_target, poison_derivable, source_id),
            ) => {
                let todo = true; //TODO poison crit
                let poison = match poison_derivable {
                    DerivedValue::Constant(p) => *p,
                    _ => self.derive_value(poison_derivable.clone(), source_id)? as u32,
                };
                match owner == player_target {
                    true => self.player.poison(poison),
                    false => self.opponent.poison(poison),
                }
            }
            TaggedCombatEvent(
                owner,
                CombatEvent::ApplyShield(player_target, shield, source_id),
            ) => {
                let todo = true; //TODO shield crit
                let shield_value = match *shield {
                    DerivedValue::Constant(s) => s,
                    _ => self.derive_value(shield.clone(), source_id)? as u32,
                };
                match owner == player_target {
                    true => self.player.shield(shield_value),
                    false => self.opponent.shield(shield_value),
                }
            }
            TaggedCombatEvent(owner, CombatEvent::Heal(player_target, heal, source_id)) => {
                let todo = true; //TODO heal crit
                let heal_value = match *heal {
                    DerivedValue::Constant(s) => s,
                    _ => self.derive_value(heal.clone(), source_id)? as u32,
                };
                match owner == player_target {
                    true => self.player.heal(heal_value),
                    false => self.opponent.heal(heal_value),
                }
            }
            TaggedCombatEvent(.., CombatEvent::Haste(target, duration, source_id)) => {
                let candidate_ids: Vec<GlobalCardId> = self
                    .get_cards_by_target(source_id, target.target_condition())
                    .into_iter()
                    .collect();
                let to_haste = target.number_of_targets();
                self.dispatch_log(format!("Haste request: {}", to_haste));

                let (mut no_haste, mut has_haste): (Vec<_>, Vec<_>) =
                    candidate_ids.into_iter().partition(|&id| {
                        self.cards
                            .get(&id)
                            .map_or(false, |card| card.haste_ticks == GameTicks(0))
                    });

                no_haste.shuffle(rng);
                has_haste.shuffle(rng);

                let mut chosen = Vec::new();
                chosen.extend(no_haste.into_iter().take(to_haste));
                if chosen.len() < to_haste {
                    chosen.extend(has_haste.into_iter().take(to_haste - chosen.len()));
                }

                self.dispatch_log(format!("Selected to haste: {:?}", chosen));

                for id in chosen {
                    if let Some(card_ref) = self.cards.get(&id) {
                        let summary = CardSummary::from(card_ref);
                        self.dispatch_event(&DispatchableEvent::CardHasted(summary, *duration));
                    } else {
                        self.dispatch_event(&DispatchableEvent::Warning(format!(
                            "attempted to haste card with id {id} which isn't on the board"
                        )));
                    }

                    if let Some(card_mut) = self.cards.get_mut(&id) {
                        card_mut.haste(*duration);
                    }
                }
            }
            TaggedCombatEvent(.., CombatEvent::Slow(target, duration, source_id)) => {
                let mut candidate_ids: Vec<GlobalCardId> = self
                    .get_cards_by_target(source_id, target.target_condition())
                    .into_iter()
                    .collect();
                let to_slow = target.number_of_targets();
                self.dispatch_log(format!("Slow request: {}", to_slow));

                candidate_ids.retain(|&id| {
                    let todo = true; //TODO optimize
                    self.cards
                        .get(&id)
                        .and_then(|card| {
                            card.modification_tooltips
                                .iter()
                                .find(|m| matches!(m, Tooltip::StaticModifier(Modifier::Radiant)))
                        })
                        .is_none()
                        && self
                            .cards
                            .get(&id)
                            .and_then(|card| Some(card.cooldown > GameTicks(0)))
                            .is_some()
                });

                let (mut not_slowed, mut already_slowed): (Vec<_>, Vec<_>) =
                    candidate_ids.into_iter().partition(|&id| {
                        self.cards
                            .get(&id)
                            .map_or(false, |card| card.slow_ticks == GameTicks(0))
                    });

                not_slowed.shuffle(rng);
                already_slowed.shuffle(rng);

                let mut chosen = Vec::new();
                chosen.extend(not_slowed.into_iter().take(to_slow));
                if chosen.len() < to_slow {
                    chosen.extend(already_slowed.into_iter().take(to_slow - chosen.len()));
                }

                self.dispatch_log(format!("Selected card to slow: {:?}", chosen));

                for id in chosen {
                    if let Some(card_ref) = self.cards.get(&id) {
                        let summary = CardSummary::from(card_ref);
                        self.dispatch_event(&DispatchableEvent::CardSlowed(summary, *duration));
                    } else {
                        self.dispatch_event(&DispatchableEvent::Warning(format!(
                            "attempted to slow card with id {id} which isn't on the board"
                        )));
                    }

                    if let Some(card_mut) = self.cards.get_mut(&id) {
                        card_mut.slow(*duration);
                    } else {
                        tracing::warn!("slow unknown item {id}");
                    }
                }
            }
            TaggedCombatEvent(.., CombatEvent::Freeze(target, duration, source_id)) => {
                let mut candidate_ids: Vec<GlobalCardId> = self
                    .get_cards_by_target(source_id, target.target_condition())
                    .into_iter()
                    .collect();
                let to_freeze = target.number_of_targets();
                self.dispatch_log(format!("Freeze request: {}", to_freeze));

                candidate_ids.retain(|&id| {
                    let todo = true; //TODO optimize
                    self.cards
                        .get(&id)
                        .and_then(|card| {
                            card.modification_tooltips
                                .iter()
                                .find(|m| matches!(m, Tooltip::StaticModifier(Modifier::Radiant)))
                        })
                        .is_none()
                        && self
                            .cards
                            .get(&id)
                            .and_then(|card| Some(card.cooldown > GameTicks(0)))
                            .is_some()
                });

                let (mut not_frozen, mut already_frozen): (Vec<_>, Vec<_>) =
                    candidate_ids.into_iter().partition(|&id| {
                        self.cards
                            .get(&id)
                            .map_or(false, |card| card.freeze_ticks == GameTicks(0))
                    });

                not_frozen.shuffle(rng);
                already_frozen.shuffle(rng);

                let mut chosen = Vec::new();
                chosen.extend(not_frozen.into_iter().take(to_freeze));
                if chosen.len() < to_freeze {
                    chosen.extend(already_frozen.into_iter().take(to_freeze - chosen.len()));
                }

                self.dispatch_log(format!("Selected card to freeze: {:?}", chosen));

                for id in chosen {
                    if let Some(card_ref) = self.cards.get(&id) {
                        let summary = CardSummary::from(card_ref);
                        self.dispatch_event(&DispatchableEvent::CardFrozen(summary, *duration));
                    } else {
                        self.dispatch_event(&DispatchableEvent::Warning(format!(
                            "attempted to freeze card with id {id} which isn't on the board"
                        )));
                    }

                    if let Some(card_mut) = self.cards.get_mut(&id) {
                        card_mut.freeze(*duration);
                    }
                }
            }
            TaggedCombatEvent(.., CombatEvent::Tick(..)) => {}
            // Keeping this is useful whenever new events are implemented
            #[allow(unreachable_patterns)]
            _ => self.dispatch_event(&DispatchableEvent::Warning(format!(
                "Unable to apply event: {event:?}"
            ))),
        }
        Ok(())
    }

    pub fn derive_value(
        &self,
        value: DerivedValue<u32>,
        source_id: &GlobalCardId,
    ) -> anyhow::Result<f32> {
        let todo = true; //TODO probably wont scale long term with just u32
        let v = value.clone();
        match value {
            DerivedValue::Constant(..) => anyhow::bail!("constants do not need to be derived"),
            DerivedValue::FromCard(card_target, card_derived_property, modifier) => {
                let targets: Vec<&Card> = self
                    .get_cards_by_target(source_id, card_target.target_condition())
                    .iter()
                    .map(|id| self.cards.get(id))
                    .flatten()
                    .collect();
                let todo = true; //TODO: Use the modifications as well
                match card_derived_property {
                    CardDerivedProperty::Value => Ok(modifier
                        * targets
                            .iter()
                            .map(|t| t.tier.scale_cost(t.inner.size.base_cost()) as f32)
                            .sum::<f32>()),
                    CardDerivedProperty::Damage => Ok(modifier
                        * targets
                            .iter()
                            .flat_map(|t| {
                                t.cooldown_effects.iter().map(|e| match e {
                                    Effect::DealDamage(.., d) => match d {
                                        DerivedValue::Constant(v) => Some(*v as f32),
                                        v => {
                                            // SAFETY Careful with recursion
                                            self.derive_value(v.clone(), source_id).ok()
                                        }
                                    },
                                    _ => None,
                                })
                            })
                            .flatten()
                            .sum::<f32>()),
                }
            }
            DerivedValue::FromPlayer(..) => todo!(),
        }
        .inspect(|derived| self.dispatch_log(format!("Derived {derived:?} from {v:?}")))
    }

    pub fn get_exit_condition(
        &self,
        t_now: Instant,
        t_start: Instant,
        events: &Vec<TaggedCombatEvent>,
    ) -> Option<SimulationResult> {
        let player_dead = self.player.health.current() <= 0;
        let opponent_dead = self.opponent.health.current() <= 0;
        if player_dead && opponent_dead {
            return Some(SimulationResult::Draw(
                SimulationDrawType::SimultaneousDefeat,
                SimulationResultInner {
                    events: events.clone(),
                    duration: t_now - t_start,
                    player: self.player.clone(),
                    opponent: self.opponent.clone(),
                },
            ));
        }
        if self.opponent.health.current() <= 0 {
            return Some(SimulationResult::Victory(SimulationResultInner {
                events: events.clone(),
                duration: t_now - t_start,
                player: self.player.clone(),
                opponent: self.opponent.clone(),
            }));
        }
        if self.player.health.current() <= 0 {
            return Some(SimulationResult::Defeat(SimulationResultInner {
                events: events.clone(),
                duration: t_now - t_start,
                player: self.player.clone(),
                opponent: self.opponent.clone(),
            }));
        }
        None
    }

    pub fn run_once_with_rng(&mut self, mut rng: StdRng) -> SimulationResult {
        let t_start = Instant::now();
        let mut events = Vec::with_capacity(*SIMULATION_TICK_COUNT);
        for _ in 0..*SIMULATION_TICK_COUNT {
            if let Some(result) = self.get_exit_condition(Instant::now(), t_start, &events) {
                return result;
            }

            let tick_events = self.tick();
            for event in &tick_events {
                self.apply_event(event, &mut rng)
                    .inspect_err(|error| {
                        self.dispatch_event(&DispatchableEvent::Error(format!("{error}")))
                    })
                    .ok();
            }
            events.extend(tick_events);
        }

        SimulationResult::Draw(
            SimulationDrawType::Timeout,
            SimulationResultInner {
                events,
                duration: Instant::now() - t_start,
                player: self.player.clone(),
                opponent: self.opponent.clone(),
            },
        )
    }

    pub fn create_rng() -> rand::rngs::StdRng {
        rand::rngs::StdRng::from_rng(&mut rand::rng())
    }

    pub fn run_once(&mut self) -> SimulationResult {
        let rng = rand::rngs::StdRng::from_rng(&mut rand::rng());
        self.run_once_with_rng(rng)
    }

    pub fn run(self, iterations: usize) -> Vec<SimulationResult> {
        let rng = rand::rngs::StdRng::from_rng(&mut rand::rng());
        self.run_with_rng(iterations, rng)
    }

    pub fn run_with_rng(
        mut self,
        iterations: usize,
        rng: rand::rngs::StdRng,
    ) -> Vec<SimulationResult> {
        let mut results: Vec<SimulationResult> = Vec::new();
        for _ in 0..iterations {
            tracing::info_span!("simulation_iteration")
                .in_scope(|| results.push(self.run_once_with_rng(rng.clone())));
        }
        results
    }
}
