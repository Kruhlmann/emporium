use super::{
    CardCombatEncounter, CardEnchantment, Hero, PackId, Size, Tag, Tier, TieredValue, Tooltip,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: &'static str,
    pub name: &'static str,
    pub starting_tier: Tier,
    pub tiers: TieredValue<Vec<Tooltip>>,
    pub tags: Vec<Tag>,
    pub hidden_tags: Vec<Tag>,
    pub custom_tags: Vec<Tag>,
    pub size: Size,
    pub heroes: Vec<Hero>,
    pub enchantments: Vec<CardEnchantment>,
    pub unified_tooltips: Vec<&'static str>,
    pub pack_id: PackId,
    pub combat_encounters: Vec<CardCombatEncounter>,
}

#[derive(Debug, Clone)]
pub struct TieredCard {
    pub card: Card,
    pub tier: Tier,
}
