use std::time::{Duration, Instant};

use models::v2_0_0::{
    CardTarget, Effect, EffectEvent, Enchantment, Modifier, PlayerTarget, Tier, Tooltip,
};
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::Deserialize;

lazy_static::lazy_static! {
    static ref NUMBER_OF_BOARD_SPACES: u8 = 10;
    static ref TICKS_PER_SECOND: usize = 60;
    static ref TICKRATE: f32 = 1000.0 / *TICKS_PER_SECOND as f32;
    static ref TICK_DURATION: Duration = Duration::from_millis(TICKRATE.round() as u64);
    static ref DURATION_BEFORE_SANDSTORM: Duration = Duration::from_secs(35);
    static ref MAX_FIGHT_DURATION: Duration = Duration::from_secs(300);
    static ref SIMULATION_TICK_COUNT: usize = {
        let fight_ms = MAX_FIGHT_DURATION.as_micros();
        let tick_ms  = TICK_DURATION.as_micros();
        (fight_ms / tick_ms) as usize
    };
}

#[derive(Clone, Debug)]
pub struct TaggedCombatEvent(pub PlayerTarget, pub CombatEvent);

#[derive(Clone, Debug)]
pub enum SkipReason {
    IsFrozen,
}

pub enum CardIndex {
    PlayerCard(usize),
    OpponentCard(usize),
}

#[derive(Clone, Debug)]
pub enum CombatEvent {
    Raw(String),
    Skip(SkipReason),
    DealDamage(PlayerTarget, i64),
    ApplyBurn(PlayerTarget, i64),
    ApplyPoison(PlayerTarget, i64),
    Freeze(CardTarget, GameTicks),
}

pub fn effect_to_combat_events(value: &Effect) -> Vec<CombatEvent> {
    match value {
        Effect::DealDamage(player_target, damage) => {
            vec![CombatEvent::DealDamage(
                *player_target,
                (*damage).try_into().unwrap_or(0),
            )]
        }
        Effect::Burn(player_target, burn) => {
            vec![CombatEvent::ApplyBurn(
                *player_target,
                (*burn).try_into().unwrap_or(0),
            )]
        }
        Effect::Poison(player_target, poison) => {
            vec![CombatEvent::ApplyPoison(
                *player_target,
                (*poison).try_into().unwrap_or(0),
            )]
        }
        Effect::Freeze(target, duration_seconds) => {
            let duration: GameTicks = Duration::from_secs_f64(*duration_seconds).into();
            vec![CombatEvent::Freeze(target.clone(), duration)]
        }
        _ => vec![CombatEvent::Raw(format!("{value}"))],
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum CardModification {
    Enchanted(models::v2_0_0::Enchantment),
}

#[derive(Clone, Debug, Deserialize)]
pub struct CardTemplate {
    pub name: String,
    pub tier: Tier,
    #[serde(default)]
    pub modifications: Vec<CardModification>,
}

#[derive(Clone, Debug)]
pub struct GameTicks(pub u128);

impl From<Duration> for GameTicks {
    fn from(value: Duration) -> Self {
        Self(value.as_millis() / TICK_DURATION.as_millis())
    }
}

#[derive(Clone, Debug)]
pub struct Card {
    pub inner: models::v2_0_0::Card,
    pub modifications: Vec<CardModification>,
    pub cooldown_effects: Vec<Effect>,
    pub cooldown: GameTicks,
    pub cooldown_counter: u128,
    pub freeze_ticks: GameTicks,
    pub slow_ticks: GameTicks,
}

impl Card {
    pub fn tick(&mut self) -> Vec<CombatEvent> {
        if self.freeze_ticks.0 > 0 {
            self.freeze_ticks.0 -= 1;
            eprintln!("freeze ticks now at {:?}", self.freeze_ticks);
            return vec![CombatEvent::Skip(SkipReason::IsFrozen)];
        }
        let mut events: Vec<CombatEvent> = Vec::new();
        if self.cooldown.0 > 0 {
            if self.cooldown_counter % self.cooldown.0 == 0 {
                for effect in &self.cooldown_effects {
                    let mut effect_events: Vec<CombatEvent> = effect_to_combat_events(effect);
                    events.append(&mut effect_events);
                }
            }
        }
        self.cooldown_counter += 1;
        events
    }
}

impl TryFrom<&CardTemplate> for Card {
    type Error = anyhow::Error;

    fn try_from(value: &CardTemplate) -> Result<Self, Self::Error> {
        let create_item: fn() -> models::v2_0_0::Card =
            *gamedata::v2_0_0::cards::CONSTRUCT_CARD_BY_NAME
                .get(value.name.as_str())
                .ok_or(anyhow::anyhow!("unknown card {:?}", &value.name))?;
        let inner = create_item();
        let tooltips = value.tier.select(inner.tiers.clone());
        if tooltips.len() == 0 {
            anyhow::bail!(
                "no tooltips on card {} of tier {:?}",
                value.name,
                value.tier
            );
        }
        let cooldown_effects: Vec<Effect> = tooltips
            .iter()
            .filter_map(|t| match t {
                Tooltip::When(EffectEvent::OnCooldown(e)) => Some(e.clone()),
                _ => None,
            })
            .inspect(|e| eprintln!("cooldown load: {e:?}"))
            .collect();
        let cooldown = tooltips
            .iter()
            .find_map(|t| match t {
                Tooltip::StaticModifier(Modifier::Cooldown(c)) => {
                    Some(Duration::from_millis((c * 1000.0).round() as u64))
                }
                _ => None,
            })
            .unwrap_or(Duration::from_secs(0));
        Ok(Card {
            inner,
            modifications: value.modifications.clone(),
            cooldown: cooldown.into(),
            cooldown_effects,
            cooldown_counter: 0,
            freeze_ticks: GameTicks(0),
            slow_ticks: GameTicks(0),
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PlayerHealth(i64, u64);

impl PlayerHealth {
    pub fn max(&self) -> u64 {
        self.1
    }

    pub fn current(&self) -> i64 {
        self.0
    }
}

impl std::ops::Add<i64> for PlayerHealth {
    type Output = Self;

    fn add(self, other: i64) -> Self::Output {
        Self(self.0 + other, self.1)
    }
}

impl std::ops::Sub<i64> for PlayerHealth {
    type Output = Self;

    fn sub(self, other: i64) -> Self::Output {
        Self(self.0 - other, self.1)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerTemplate {
    pub health: u64,
    #[serde(default, rename = "cards")]
    pub card_templates: Vec<CardTemplate>,
    #[serde(default, rename = "skills")]
    pub skill_templates: Vec<CardTemplate>,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub health: PlayerHealth,
    pub shield: i64,
    pub poison_stacks: i64,
    pub burn_stacks: i64,
    pub regeneration_stacks: i64,
    pub cards: Vec<Card>,
    pub template: PlayerTemplate,
    pub dot_counter: usize,
}

impl Player {
    pub fn burn(&mut self, amount: i64) {
        self.burn_stacks += amount
            .try_into()
            .inspect_err(|_| eprintln!("invalid burn value {amount}"))
            .unwrap_or(0);
    }

    pub fn poison(&mut self, amount: i64) {
        self.poison_stacks += amount
            .try_into()
            .inspect_err(|_| eprintln!("invalid poison value {amount}"))
            .unwrap_or(0);
    }

    // TODO: This is bad.. increase the tickrate and filter every other tick for cards.
    pub fn burn_tick(&mut self) {
        if self.burn_stacks > 0 {
            let modifier = if self.shield > 0 { 0.5 } else { 1.0 };
            self.health = self.health - (self.burn_stacks as f64 * modifier).round() as i64;
            self.burn_stacks -= 1;
        }
    }

    pub fn tick(&mut self) -> Vec<CombatEvent> {
        if self.dot_counter % *TICKS_PER_SECOND == 0 {
            // Half-ticks, Half-measure (burn)
            for _ in 0..2 {
                self.burn_tick();
            }
            self.health = self.health + self.regeneration_stacks - self.poison_stacks;
        }
        let mut events: Vec<CombatEvent> = Vec::new();
        for card in &mut self.cards {
            let mut card_events = &mut card.tick();
            events.append(&mut card_events)
        }
        self.dot_counter += 1;
        events
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cards = self
            .template
            .card_templates
            .iter()
            .map(|c| format!("{} ({:?})", c.name, c.tier))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "Player(❤️ {}/{}, 🔥:{}, 🧪:{}, 🌱:{}, 🛡️:{}) [{cards}]",
            self.health.current(),
            self.health.max(),
            self.burn_stacks,
            self.poison_stacks,
            self.regeneration_stacks,
            self.shield,
        )
    }
}

impl TryFrom<PlayerTemplate> for Player {
    type Error = anyhow::Error;

    fn try_from(value: PlayerTemplate) -> Result<Self, Self::Error> {
        let mut cards: Vec<Card> = Vec::new();
        for template in &value.card_templates {
            let card = template.try_into().inspect_err(|error| {
                eprintln!("unable to parse card template {template:?}: {error}")
            })?;
            cards.push(card);
        }
        let board_spaces_required: u8 = cards.iter().map(|c| c.inner.size.board_spaces()).sum();
        println!("board_spaces_required {board_spaces_required}");
        if board_spaces_required > *NUMBER_OF_BOARD_SPACES {
            anyhow::bail!("board too large ({board_spaces_required} spaces)");
        }

        Ok(Self {
            health: PlayerHealth(value.health.try_into()?, value.health),
            shield: 0,
            poison_stacks: 0,
            burn_stacks: 0,
            regeneration_stacks: 0,
            dot_counter: 0,
            cards,
            template: value,
        })
    }
}

#[derive(Debug)]
pub struct SimulationResultInner {
    pub source: Option<String>,
    pub events: Vec<TaggedCombatEvent>,
    pub duration: Duration,
    pub player: Player,
    pub opponent: Player,
}

#[derive(Debug, PartialEq)]
pub enum SimulationDrawType {
    Timeout,
    SimultaneousDefeat,
}

#[derive(Debug)]
pub enum SimulationResult {
    Victory(SimulationResultInner),
    Defeat(SimulationResultInner),
    Draw(SimulationDrawType, SimulationResultInner),
}

impl std::fmt::Display for SimulationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = match self {
            SimulationResult::Victory(r)
            | SimulationResult::Defeat(r)
            | SimulationResult::Draw(SimulationDrawType::Timeout, r)
            | SimulationResult::Draw(SimulationDrawType::SimultaneousDefeat, r) => r,
        };
        let source_str = inner
            .source
            .as_ref()
            .map(|s| format!(" [{s}] "))
            .unwrap_or("".to_string());
        let header = match self {
            SimulationResult::Victory(..) => {
                format!("[{:?}]{source_str} Victory", inner.duration)
            }
            SimulationResult::Defeat(..) => format!("[{:?}]{source_str} Defeat", inner.duration),
            SimulationResult::Draw(SimulationDrawType::Timeout, ..) => {
                format!("[{:?}]{source_str} Draw by timeout", inner.duration)
            }
            SimulationResult::Draw(SimulationDrawType::SimultaneousDefeat, ..) => {
                format!(
                    "[{:?}]{source_str} Draw by simultaneous defeat",
                    inner.duration,
                )
            }
        };
        write!(f, "{header}")
    }
}

#[derive(Debug, Deserialize)]
pub struct SimulationTemplate {
    pub player: PlayerTemplate,
    pub opponent: PlayerTemplate,
    #[serde(default)]
    pub verbose: bool,
    #[serde(skip, default)]
    pub source: Option<String>,
}

#[derive(Debug)]
pub struct Simulation {
    pub player: Player,
    pub opponent: Player,
    pub verbose: bool,
    pub source: Option<String>,
}

impl TryFrom<SimulationTemplate> for Simulation {
    type Error = anyhow::Error;

    fn try_from(value: SimulationTemplate) -> Result<Self, Self::Error> {
        Ok(Self {
            player: value.player.try_into()?,
            opponent: value.opponent.try_into()?,
            verbose: value.verbose,
            source: value.source,
        })
    }
}

impl Simulation {
    fn tick(&mut self) -> Vec<TaggedCombatEvent> {
        let mut events: Vec<TaggedCombatEvent> = Vec::new();
        for event in self.player.tick() {
            events.push(TaggedCombatEvent(PlayerTarget::Player, event));
        }
        for event in self.opponent.tick() {
            events.push(TaggedCombatEvent(PlayerTarget::Opponent, event));
        }
        events
    }

    fn apply_event(&mut self, event: &TaggedCombatEvent, rng: &mut StdRng) {
        match event {
            TaggedCombatEvent(.., CombatEvent::Skip(reason)) => {
                eprintln!("skipped event due to {reason:?}")
            }
            TaggedCombatEvent(.., CombatEvent::Raw(s)) => eprintln!("skipped raw event {s}"),
            TaggedCombatEvent(owner, CombatEvent::DealDamage(target, dmg)) => match owner == target
            {
                true => self.player.health = self.player.health - *dmg,
                false => self.opponent.health = self.opponent.health - *dmg,
            },
            TaggedCombatEvent(owner, CombatEvent::ApplyBurn(target, burn)) => match owner == target
            {
                true => self.player.burn(*burn),
                false => self.opponent.burn(*burn),
            },
            TaggedCombatEvent(owner, CombatEvent::ApplyPoison(target, poison)) => {
                match owner == target {
                    true => self.player.poison(*poison),
                    false => self.opponent.poison(*poison),
                }
            }
            TaggedCombatEvent(owner, CombatEvent::Freeze(card_target, duration)) => {
                match card_target {
                    CardTarget::NOpponent(n) => {
                        let cards = match owner {
                            PlayerTarget::Player => &mut self.opponent.cards,
                            PlayerTarget::Opponent => &mut self.player.cards,
                        };
                        let mut freezable_targets: Vec<&mut Card> = cards
                            .iter_mut()
                            .filter(|card| {
                                card.modifications
                                    .iter()
                                    .find(|m| {
                                        matches!(
                                            m,
                                            CardModification::Enchanted(Enchantment::Radiant)
                                        )
                                    })
                                    .is_none()
                            })
                            .collect();
                        let count = std::cmp::min(*n, freezable_targets.len());
                        freezable_targets.shuffle(rng);

                        for card in freezable_targets.into_iter().take(count) {
                            card.freeze_ticks = duration.clone();
                        }
                    }
                    _ => panic!("invalid freeze event {event:?}"),
                }
            }
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

    pub fn run(self) -> SimulationResult {
        let rng = rand::rngs::StdRng::from_rng(&mut rand::rng());
        self.run_with_rng(rng)
    }

    pub fn run_with_rng(mut self, mut rng: rand::rngs::StdRng) -> SimulationResult {
        let t_start = Instant::now();
        let mut events: Vec<TaggedCombatEvent> = Vec::new();

        for i in 0..*SIMULATION_TICK_COUNT {
            let t_now = Instant::now();

            if let Some(result) = self.get_exit_condition(t_now, t_start, &events) {
                return result;
            }

            let mut tick_events = self.tick();
            let mut log_line = String::new();
            for event in &tick_events {
                self.apply_event(&event, &mut rng);
                log_line.push_str(&format!("      {} -> {:?}\n", event.0, event.1));
            }
            events.append(&mut tick_events);
            eprint!(
                "\n[{i}]\n  {}\n  {}\n{}",
                self.player, self.opponent, log_line
            );
        }
        eprintln!(
            "reached iteration {} with no winner",
            *SIMULATION_TICK_COUNT
        );
        SimulationResult::Draw(
            SimulationDrawType::Timeout,
            SimulationResultInner {
                source: self.source,
                events,
                duration: Instant::now() - t_start,
                player: self.player,
                opponent: self.opponent,
            },
        )
    }
}
