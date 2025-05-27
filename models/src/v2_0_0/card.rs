use std::sync::Arc;

use super::{
    CardCombatEncounter, CardEnchantment, Hero, PackId, Size, Tag, Tier, TieredValue, Tooltip,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: &'static str,
    pub name: &'static str,
    pub starting_tier: Tier,
    pub tiers: TieredValue<Arc<[Tooltip]>>,
    pub tags: Arc<[Tag]>,
    pub hidden_tags: Arc<[Tag]>,
    pub custom_tags: Arc<[Tag]>,
    pub size: Size,
    pub heroes: Arc<[Hero]>,
    pub enchantments: Arc<[CardEnchantment]>,
    pub unified_tooltips: Arc<[&'static str]>,
    pub pack_id: PackId,
    pub combat_encounters: Arc<[CardCombatEncounter]>,
}

#[derive(Debug, Clone)]
pub struct TieredCard {
    pub card: Card,
    pub tier: Tier,
}

impl Card {
    pub fn min_tier(&self) -> Tier {
        if !self.tiers.bronze.is_empty() {
            return Tier::Bronze;
        }
        if !self.tiers.silver.is_empty() {
            return Tier::Silver;
        }
        if !self.tiers.gold.is_empty() {
            return Tier::Gold;
        }
        if !self.tiers.diamond.is_empty() {
            return Tier::Diamond;
        }
        if !self.tiers.legendary.is_empty() {
            return Tier::Legendary;
        }
        panic!("no tiers");
    }

    pub fn max_tier(&self) -> Tier {
        if !self.tiers.legendary.is_empty() {
            return Tier::Legendary;
        }
        if !self.tiers.diamond.is_empty() {
            return Tier::Diamond;
        }
        if !self.tiers.gold.is_empty() {
            return Tier::Gold;
        }
        if !self.tiers.silver.is_empty() {
            return Tier::Silver;
        }
        if !self.tiers.bronze.is_empty() {
            return Tier::Bronze;
        }
        panic!("no tiers");
    }

    pub fn available_tiers(&self) -> Vec<Tier> {
        let mut tier_range: Vec<Tier> = Vec::with_capacity(5);
        if !self.tiers.bronze.is_empty() {
            tier_range.push(Tier::Bronze);
        }
        if !self.tiers.silver.is_empty() {
            tier_range.push(Tier::Silver);
        }
        if !self.tiers.gold.is_empty() {
            tier_range.push(Tier::Gold);
        }
        if !self.tiers.diamond.is_empty() {
            tier_range.push(Tier::Diamond);
        }
        if !self.tiers.legendary.is_empty() {
            tier_range.push(Tier::Legendary);
        }
        tier_range
    }
}
