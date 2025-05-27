use std::time::Instant;

use indexmap::IndexMap;
use models::v2_0_0::{
    CardDerivedProperty, DerivedValue, Enchantment, PlayerTarget, TargetCondition,
};
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};

use crate::{
    Card, CardModification, CardSummary, CombatEvent, DispatchableEvent, GlobalCardId,
    NUMBER_OF_BOARD_SPACES, Player, SIMULATION_TICK_COUNT, SimulationDrawType, SimulationResult,
    SimulationResultInner, SimulationTemplate, TaggedCombatEvent,
};

#[derive(Clone, Debug)]
pub struct Simulation {
    pub player: Player,
    pub opponent: Player,
    pub source: Option<String>,
    pub event_sender: Option<std::sync::mpsc::Sender<DispatchableEvent>>,
    pub stdout_enabled: bool,
    pub cards: IndexMap<GlobalCardId, Card>,
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
        let opponent_card_ids: Vec<GlobalCardId> = player_cards.keys().cloned().collect();
        let mut cards = player_cards;
        cards.extend(opponent_cards.into_iter());

        Ok(Self {
            cards,
            player: template.player.create_player(player_card_ids)?,
            opponent: template.opponent.create_player(opponent_card_ids)?,
            source: template.source,
            event_sender: None,
            stdout_enabled: false,
        })
    }
}

impl Simulation {
    // TODO Dont store them like this, keep them indexable at the position for performance during
    // lookups
    pub fn card_at_position<'a>(&self, cards: &'a Vec<Card>, position: u8) -> Option<&'a Card> {
        if let Some(c) = cards.iter().find(|c| c.position == position) {
            Some(c)
        } else {
            self.dispatch_event(&DispatchableEvent::Warning(format!(
                "attempt to get card at position {position} failed"
            )));
            None
        }
    }

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
        self.dispatch_log(format!("{condition}"));
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
            eprintln!("condition {condition:?} found the following: {cards:?}");
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

    pub fn with_stdout(mut self) -> Self {
        self.stdout_enabled = true;
        self
    }

    fn dispatch_log(&self, s: String) {
        let event = &DispatchableEvent::Log(s);
        self.dispatch_event(event)
    }

    fn dispatch_event(&self, event: &DispatchableEvent) {
        if let Some(ref tx) = self.event_sender {
            let _ = tx.send(event.clone());
        }
        if self.stdout_enabled {
            eprintln!("EVENT: {:?}", event);
        }
    }

    fn tick(&mut self) -> Vec<TaggedCombatEvent> {
        let mut events: Vec<TaggedCombatEvent> = Vec::new();
        self.player.tick();
        self.opponent.tick();
        for (_, card) in &mut self.cards {
            for e in card.tick() {
                events.push(TaggedCombatEvent(card.owner, e));
            }
        }
        events
    }

    fn apply_event(&mut self, event: &TaggedCombatEvent, rng: &mut StdRng) -> anyhow::Result<()> {
        self.dispatch_log(format!("{event:?}"));
        match event {
            TaggedCombatEvent(.., CombatEvent::Skip(reason)) => {
                self.dispatch_event(&DispatchableEvent::Warning(format!(
                    "event skipped: {reason:?}"
                )));
            }
            TaggedCombatEvent(.., CombatEvent::Raw(s)) => {
                self.dispatch_event(&DispatchableEvent::Warning(format!(
                    "raw event skipped: {s}"
                )));
            }
            TaggedCombatEvent(owner, CombatEvent::DealDamage(player_target, dmg, source_id)) => {
                if let Some(card) = self.cards.get(source_id) {
                    let r = rng.random::<f64>();
                    let did_crit = card.crit_chance.as_fraction() < r;
                    let dmg = if did_crit {
                        // TODO what about increased crit dmg
                        *dmg + *dmg
                    } else {
                        *dmg
                    };
                    match owner == player_target {
                        true => self.player.take_damage(dmg),
                        false => self.opponent.take_damage(dmg),
                    }
                } else {
                    // TODO else what?
                }
            }
            TaggedCombatEvent(owner, CombatEvent::ApplyBurn(player_target, burn, ..)) => {
                // TODO burn crit
                match owner == player_target {
                    true => self.player.burn(*burn),
                    false => self.opponent.burn(*burn),
                }
            }
            TaggedCombatEvent(owner, CombatEvent::ApplyPoison(player_target, poison, ..)) => {
                // TODO poison crit
                match owner == player_target {
                    true => self.player.poison(*poison),
                    false => self.opponent.poison(*poison),
                }
            }
            TaggedCombatEvent(
                owner,
                CombatEvent::ApplyShield(player_target, shield, source_id),
            ) => {
                // TODO shield crit
                let shield_value = match *shield {
                    DerivedValue::Constant(s) => s,
                    _ => self.derive_value(shield.clone(), source_id)? as u32,
                };
                let shield_target = match owner {
                    PlayerTarget::Player => player_target,
                    PlayerTarget::Opponent => &player_target.inverse(),
                };
                match shield_target {
                    PlayerTarget::Player => self.player.shield(shield_value),
                    PlayerTarget::Opponent => self.opponent.shield(shield_value),
                }
            }
            TaggedCombatEvent(.., CombatEvent::Freeze(target, duration, source_id)) => {
                let candidate_ids: Vec<GlobalCardId> = self
                    .get_cards_by_target(source_id, target.target_condition())
                    .into_iter()
                    .collect();
                let to_freeze = target.number_of_targets();
                self.dispatch_log(format!("Freeze {to_freeze:?}"));
                let mut eligible_ids: Vec<GlobalCardId> = candidate_ids
                    .iter()
                    .filter_map(|&id| match self.cards.get(&id) {
                        Some(card)
                            if card.modifications.iter().any(|m| {
                                matches!(m, CardModification::Enchanted(Enchantment::Radiant))
                            }) =>
                        {
                            None
                        }
                        _ => Some(id),
                    })
                    .collect();

                if eligible_ids.len() > to_freeze {
                    eligible_ids.shuffle(rng);
                }

                self.dispatch_log(format!("Freeze {eligible_ids:?} {candidate_ids:?}"));
                for id in eligible_ids.into_iter().take(to_freeze) {
                    if let Some(card_ref) = self.cards.get(&id) {
                        let summary = CardSummary::from(card_ref);
                        self.dispatch_event(&DispatchableEvent::CardFrozen(summary, *duration));
                    } else {
                        self.dispatch_event(&DispatchableEvent::Warning(format!(
                            "attempted to freeze card with id {id} which isn't on the board"
                        )))
                    }

                    if let Some(card_mut) = self.cards.get_mut(&id) {
                        card_mut.freeze(*duration);
                    }
                }
            }
        }
        Ok(())
    }

    // TODO probably wont scale long term with just u32
    pub fn derive_value(
        &self,
        value: DerivedValue<u32>,
        source_id: &GlobalCardId,
    ) -> anyhow::Result<f32> {
        match value {
            DerivedValue::Constant(..) => anyhow::bail!("constants do not need to be derived"),
            DerivedValue::FromCard(card_target, card_derived_property, modifier) => {
                let targets: Vec<&Card> = self
                    .get_cards_by_target(source_id, card_target.target_condition())
                    .iter()
                    .map(|id| self.cards.get(id))
                    .flatten()
                    .collect();
                match card_derived_property {
                    CardDerivedProperty::Value => Ok(modifier
                        * targets
                            .iter()
                            .map(|t| t.tier.scale_cost(t.inner.size.base_cost()) as f32)
                            .sum::<f32>()),
                }
            }
            DerivedValue::FromPlayer(..) => todo!(),
        }
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
                    source: self.source.clone(),
                    events: events.clone(),
                    duration: t_now - t_start,
                    player: self.player.clone(),
                    opponent: self.opponent.clone(),
                },
            ));
        }
        if self.opponent.health.current() <= 0 {
            return Some(SimulationResult::Victory(SimulationResultInner {
                source: self.source.clone(),
                events: events.clone(),
                duration: t_now - t_start,
                player: self.player.clone(),
                opponent: self.opponent.clone(),
            }));
        }
        if self.player.health.current() <= 0 {
            return Some(SimulationResult::Defeat(SimulationResultInner {
                source: self.source.clone(),
                events: events.clone(),
                duration: t_now - t_start,
                player: self.player.clone(),
                opponent: self.opponent.clone(),
            }));
        }
        None
    }

    pub fn run_once_with_rng(&mut self, mut rng: StdRng) -> SimulationResult {
        for (id, card) in &self.cards {
            self.dispatch_log(format!(
                "registered card {}<@{}> with id {}",
                card.inner.name, card.position, id
            ));
        }
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
                source: self.source.clone(),
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
        (0..iterations)
            .into_iter()
            .map(|_| self.run_once_with_rng(rng.clone()))
            .collect()
    }
}
