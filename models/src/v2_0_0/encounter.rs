use super::{Enchantment, TieredCard, TieredSkill};

#[derive(Debug, Clone)]
pub enum EncounterDay {
    Event,
    Numeric(usize),
}

#[derive(Debug, Clone)]
pub struct EncounterCard {
    pub card: TieredCard,
    pub enchantment: Option<Enchantment>,
}

#[derive(Debug, Clone)]
pub struct Encounter {
    pub id: &'static str,
    pub name: &'static str,
    pub level: i64,
    pub health: usize,
    pub cards: Vec<EncounterCard>,
    pub skills: Vec<TieredSkill>,
    pub day: EncounterDay,
}
