use super::{CardCombatEncounter, Hero, PackId, Tag, Tier, TieredValue, Tooltip};

#[derive(Debug, Clone)]
pub struct Skill {
    pub id: &'static str,
    pub name: &'static str,
    pub starting_tier: Tier,
    pub tiers: TieredValue<Vec<Tooltip>>,
    pub tags: Vec<Tag>,
    pub hidden_tags: Vec<Tag>,
    pub custom_tags: Vec<Tag>,
    pub heroes: Vec<Hero>,
    pub unified_tooltips: Vec<&'static str>,
    pub pack_id: PackId,
    pub combat_encounters: Vec<CardCombatEncounter>,
}

#[derive(Debug, Clone)]
pub struct TieredSkill {
    pub skill: Skill,
    pub tier: Tier,
}
