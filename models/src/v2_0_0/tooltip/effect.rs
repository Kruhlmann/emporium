use std::str::FromStr;

use heck::ToTitleCase;
use regex::Regex;

use crate::v2_0_0::{Percentage, Tag, Tier};

use super::{
    CardTarget, Condition, EffectValue, ObtainedEffectItem, PlayerTarget, TargetCondition,
};

lazy_static::lazy_static! {
    pub static ref EFFECT_GET_ITEMS_REGEX: Regex = Regex::new(r"^get\s+(a|\d+)\s+([\p{L} ]+)\.?$").unwrap();
    pub static ref EFFECT_GET_TAG_CONDITIONAL_ITEMS_REGEX: Regex = Regex::new(r"^get a ([\p{L} ]+). if you have a ([\p{L} ]+), get a second ([\p{L} ]+)\.?").unwrap();
    pub static ref EFFECT_GET_TRIPLE_SINGULAR_ITEMS_REGEX: Regex = Regex::new(r"^get a ([\p{L} ]+), ([\p{L} ]+) and ([\p{L} ]+)\.?$").unwrap();
    pub static ref EFFECT_GAIN_PERMANENT_MAX_HP: Regex = Regex::new(r"^permanently gain (\d+) max health\.?$").unwrap();
    pub static ref EFFECT_SPEND_GOLD_FOR_EFFECT: Regex = Regex::new(r"^spend (\d+) gold to ([\p{L} ]+)\.?$").unwrap();
    pub static ref EFFECT_THIS_GAINS_MAX_AMMO: Regex = Regex::new(r"^this gains (\d+) max ammo\.?$").unwrap();
    pub static ref EFFECT_POISON_SELF: Regex = Regex::new(r"^poison yourself (\d+)\.?$").unwrap();
    pub static ref EFFECT_UPGRADE_RANDOM_PIGGLE: Regex = Regex::new(r"^upgrade a random piggle\.?$").unwrap();
    pub static ref EFFECT_GAIN_GOLD: Regex = Regex::new(r"^gain (\d+) gold\.?$").unwrap();
    pub static ref EFFECT_UPGRADE_LOWER_TIER_TAGGED: Regex = Regex::new(r"^upgrade a ([\p{L} ]+) of a lower tier\.?$").unwrap();
    pub static ref EFFECT_BURN_FROM_DAMAGE: Regex = Regex::new(r"burn equal to (\d+)% of this item's damage.").unwrap();
    pub static ref EFFECT_HEAL_FROM_DAMAGE: Regex = Regex::new(r"heal equal to (\d+)% of this item's damage.").unwrap();
    pub static ref EFFECT_HEAL_FROM_DAMAGE_FULL: Regex = Regex::new(r"heal equal to this item's damage.").unwrap();
    pub static ref EFFECT_SHIELD_FROM_DAMAGE: Regex = Regex::new(r"shield equal to (\d+)% of this item's damage.").unwrap();
    pub static ref EFFECT_SHIELD_FROM_DAMAGE_FULL: Regex = Regex::new(r"shield equal to this item's damage.").unwrap();
    pub static ref EFFECT_POISON_FROM_DAMAGE: Regex = Regex::new(r"poison equal to (\d+)% of this item's damage.").unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub enum CardDerivedProperty {
    Value,
    Damage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerDerivedProperty {
    MaximumHealth,
    CurrentHealth,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DerivedValue<T> {
    Constant(T),
    FromCard(CardTarget, CardDerivedProperty, f32),
    FromPlayer(CardTarget, PlayerDerivedProperty, f32),
}

impl<T: std::fmt::Display> std::fmt::Display for DerivedValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DerivedValue::Constant(i) => write!(f, "DerivedValue::Constant({i})"),
            DerivedValue::FromCard(i, j, k) => write!(
                f,
                "DerivedValue::FromCard({i}, CardDerivedProperty::{j:?}, {k:.2})"
            ),
            DerivedValue::FromPlayer(i, j, k) => write!(
                f,
                "DerivedValue::FromPlayer({i}, PlayerDerivedProperty::{j:?}, {k:.2})"
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Raw(String),
    Use(CardTarget),
    GainShield(CardTarget, EffectValue<f64>),
    CooldownReduction(CardTarget, EffectValue<f64>),
    IncreaseDamage(CardTarget, EffectValue<u32>),
    Freeze(CardTarget, f64),
    Slow(CardTarget, f64),
    Haste(CardTarget, f64),
    Reload(CardTarget, u32),
    DamageImmunity(f64),
    Charge(CardTarget, f64),
    Destroy(CardTarget),
    Burn(PlayerTarget, DerivedValue<u32>),
    Heal(PlayerTarget, DerivedValue<u32>),
    Shield(PlayerTarget, DerivedValue<u32>),
    Regen(PlayerTarget, DerivedValue<u32>),
    Poison(PlayerTarget, DerivedValue<u32>),
    GainGold(PlayerTarget, DerivedValue<u32>),
    DealDamage(PlayerTarget, DerivedValue<u32>),
    UseCard(CardTarget),
    Upgrade(Tier, CardTarget),
    PermanentMaxHealthIncrease(DerivedValue<u32>),
    IncreaseMaxAmmo(CardTarget, EffectValue<u32>),
    ObtainItem(Vec<ObtainedEffectItem>),
    SpendGoldForEffect(u32, Box<Effect>),
    GainXp(PlayerTarget, u32),
    ConditionalMatchItem(CardTarget, Box<Effect>),
    MultiEffect(Vec<Effect>),
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Use(i) => write!(f, "Effect::Use({i})"),
            Effect::GainShield(i, j) => write!(f, "Effect::GainShield({i}, {j})"),
            Effect::CooldownReduction(i, j) => write!(f, "Effect::CooldownReduction({i}, {j})"),
            Effect::Freeze(i, j) => write!(f, "Effect::Freeze({i}, {j:.2})"),
            Effect::Slow(i, j) => write!(f, "Effect::Slow({i}, {j:.2})"),
            Effect::DamageImmunity(i) => write!(f, "Effect::DamageImmunity({i:.2})"),
            Effect::Destroy(i) => write!(f, "Effect::Destroy({i})"),
            Effect::Burn(i, j) => write!(f, "Effect::Burn({i}, {j})"),
            Effect::Shield(i, j) => write!(f, "Effect::Shield({i}, {j})"),
            Effect::Regen(i, j) => write!(f, "Effect::Regen({i}, {j})"),
            Effect::DealDamage(i, j) => write!(f, "Effect::DealDamage({i}, {j})"),
            Effect::Poison(i, j) => write!(f, "Effect::Poison({i}, {j})"),
            Effect::Heal(i, j) => write!(f, "Effect::Heal({i}, {j})"),
            Effect::IncreaseDamage(i, j) => write!(f, "Effect::IncreaseDamage({i}, {j})"),
            Effect::Reload(i, j) => write!(f, "Effect::Reload({i}, {j})"),
            Effect::Haste(i, j) => write!(f, "Effect::Haste({i}, {j:.2})"),
            Effect::Charge(i, j) => write!(f, "Effect::Charge({i}, {j:.2})"),
            Effect::GainGold(i, j) => write!(f, "Effect::GainGold({i}, {j})"),
            Effect::UseCard(i) => write!(f, "Effect::UseCard({i})"),
            Effect::SpendGoldForEffect(i, j) => {
                write!(f, "Effect::SpendGoldForEffect({i}, Box::new({j}))")
            }
            Effect::ObtainItem(i) => write!(
                f,
                "Effect::ObtainItem(vec![{}])",
                i.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Effect::Raw(i) => write!(f, "Effect::Raw({i:?}.to_string())"),
            Effect::PermanentMaxHealthIncrease(i) => {
                write!(f, "Effect::PermanentMaxHealthIncrease({i})")
            }
            Effect::IncreaseMaxAmmo(i, j) => write!(f, "Effect::IncreaseMaxAmmo({i}, {j})"),
            Effect::Upgrade(i, j) => write!(f, "Effect::Upgrade(Tier::{i:?}, {j})"),
            Effect::GainXp(i, j) => write!(f, "Effect::GainXp({i}, {j})"),
            Effect::MultiEffect(effects) => {
                let effect_str = effects
                    .iter()
                    .map(|e| format!("{e}"))
                    .collect::<Vec<String>>()
                    .join(",");
                write!(f, "Effect::MultiEffect(vec![{effect_str}])")
            }
            Effect::ConditionalMatchItem(i, j) => {
                write!(f, "Effect::ConditionalMatchItem({i}, Box::new({j}))")
            }
        }
    }
}

impl Effect {
    pub fn from_tooltip_str(tooltip: &str) -> Effect {
        let tooltip = tooltip.trim();

        if tooltip == "use this." {
            return Effect::Use(CardTarget(1, TargetCondition::IsSelf));
        }

        if let Some(captures) = EFFECT_GET_ITEMS_REGEX.captures(tooltip) {
            if let (Some(count), Some(name)) = (captures.get(1), captures.get(2)) {
                let count_str = count.as_str();
                let count = if count_str == "a" {
                    1
                } else {
                    match count_str.parse::<u32>() {
                        Ok(c) => c,
                        Err(_) => return Effect::Raw(tooltip.to_string()),
                    }
                };
                let name = name.as_str().to_title_case();
                let obtained_item = ObtainedEffectItem::new(name, count);
                return Effect::ObtainItem(vec![obtained_item]);
            }
        }
        if let Some(captures) = EFFECT_GET_TRIPLE_SINGULAR_ITEMS_REGEX.captures(tooltip) {
            if let (Some(item_a), Some(item_b), Some(item_c)) =
                (captures.get(1), captures.get(2), captures.get(3))
            {
                return Effect::ObtainItem(vec![
                    ObtainedEffectItem::new(item_a.as_str(), 1),
                    ObtainedEffectItem::new(item_b.as_str(), 1),
                    ObtainedEffectItem::new(item_c.as_str(), 1),
                ]);
            }
        }
        if let Some(captures) = EFFECT_GET_TAG_CONDITIONAL_ITEMS_REGEX.captures(tooltip) {
            if let (Some(item_a), Some(tag_str), Some(item_b)) =
                (captures.get(1), captures.get(2), captures.get(3))
            {
                if let Ok(tag) = Tag::from_str(tag_str.as_str()) {
                    return Effect::ObtainItem(vec![
                        ObtainedEffectItem::new(item_a.as_str(), 1),
                        ObtainedEffectItem::new_conditional(
                            item_b.as_str(),
                            1,
                            Condition::HasCardOfTag(tag),
                        ),
                    ]);
                }
            }
        }
        if let Some(captures) = EFFECT_POISON_SELF.captures(tooltip) {
            if let Some(poison_str) = captures.get(1) {
                if let Ok(poison) = poison_str
                    .as_str()
                    .parse::<u32>()
                    .map(DerivedValue::Constant)
                {
                    return Effect::Poison(PlayerTarget::Player, poison);
                }
            }
        }
        if let Some(captures) = EFFECT_GAIN_PERMANENT_MAX_HP.captures(tooltip) {
            if let Some(hp_str) = captures.get(1) {
                if let Ok(hp) = hp_str.as_str().parse::<u32>().map(DerivedValue::Constant) {
                    return Effect::PermanentMaxHealthIncrease(hp);
                }
            }
        }
        if let Some(captures) = EFFECT_SPEND_GOLD_FOR_EFFECT.captures(tooltip) {
            if let (Some(gold_str), Some(effect_str)) = (captures.get(1), captures.get(2)) {
                if let Ok(gold) = gold_str.as_str().parse::<u32>() {
                    return Effect::SpendGoldForEffect(
                        gold,
                        Box::new(Effect::from_tooltip_str(effect_str.as_str())),
                    );
                }
            }
        }

        if let Some(captures) = EFFECT_POISON_FROM_DAMAGE.captures(tooltip) {
            if let Some(poison_str) = captures.get(1) {
                if let Ok(poison_pct) = poison_str
                    .as_str()
                    .parse::<f64>()
                    .map(Percentage::from_percentage_value)
                {
                    return Effect::Poison(
                        PlayerTarget::Opponent,
                        DerivedValue::FromCard(
                            CardTarget(1, TargetCondition::IsSelf),
                            CardDerivedProperty::Damage,
                            poison_pct.as_fraction() as f32,
                        ),
                    );
                }
            }
        }

        if let Some(captures) = EFFECT_SHIELD_FROM_DAMAGE.captures(tooltip) {
            if let Some(shield_str) = captures.get(1) {
                if let Ok(shield_pct) = shield_str
                    .as_str()
                    .parse::<f64>()
                    .map(Percentage::from_percentage_value)
                {
                    return Effect::Shield(
                        PlayerTarget::Player,
                        DerivedValue::FromCard(
                            CardTarget(1, TargetCondition::IsSelf),
                            CardDerivedProperty::Damage,
                            shield_pct.as_fraction() as f32,
                        ),
                    );
                }
            }
        }

        if let Some(..) = EFFECT_SHIELD_FROM_DAMAGE_FULL.captures(tooltip) {
            return Effect::Shield(
                PlayerTarget::Player,
                DerivedValue::FromCard(
                    CardTarget(1, TargetCondition::IsSelf),
                    CardDerivedProperty::Damage,
                    1.0,
                ),
            );
        }

        if let Some(captures) = EFFECT_HEAL_FROM_DAMAGE.captures(tooltip) {
            if let Some(heal_str) = captures.get(1) {
                if let Ok(heal_pct) = heal_str
                    .as_str()
                    .parse::<f64>()
                    .map(Percentage::from_percentage_value)
                {
                    return Effect::Heal(
                        PlayerTarget::Player,
                        DerivedValue::FromCard(
                            CardTarget(1, TargetCondition::IsSelf),
                            CardDerivedProperty::Damage,
                            heal_pct.as_fraction() as f32,
                        ),
                    );
                }
            }
        }

        if let Some(..) = EFFECT_HEAL_FROM_DAMAGE_FULL.captures(tooltip) {
            return Effect::Heal(
                PlayerTarget::Player,
                DerivedValue::FromCard(
                    CardTarget(1, TargetCondition::IsSelf),
                    CardDerivedProperty::Damage,
                    1.0,
                ),
            );
        }

        if let Some(captures) = EFFECT_BURN_FROM_DAMAGE.captures(tooltip) {
            if let Some(burn_str) = captures.get(1) {
                if let Ok(burn_pct) = burn_str
                    .as_str()
                    .parse::<f64>()
                    .map(Percentage::from_percentage_value)
                {
                    return Effect::Burn(
                        PlayerTarget::Opponent,
                        DerivedValue::FromCard(
                            CardTarget(1, TargetCondition::IsSelf),
                            CardDerivedProperty::Damage,
                            burn_pct.as_fraction() as f32,
                        ),
                    );
                }
            }
        }

        if let Some(captures) = EFFECT_GAIN_GOLD.captures(tooltip) {
            if let Some(gold_str) = captures.get(1) {
                if let Ok(gold) = gold_str.as_str().parse::<u32>().map(DerivedValue::Constant) {
                    return Effect::GainGold(PlayerTarget::Player, gold);
                }
            }
        }
        if let Some(captures) = EFFECT_THIS_GAINS_MAX_AMMO.captures(tooltip) {
            if let Some(amount_str) = captures.get(1) {
                if let Ok(amount) = amount_str.as_str().parse::<u32>() {
                    return Effect::IncreaseMaxAmmo(
                        CardTarget(1, TargetCondition::IsSelf),
                        EffectValue::Flat(amount),
                    );
                }
            }
        }
        if EFFECT_UPGRADE_RANDOM_PIGGLE.is_match(tooltip) {
            return Effect::Upgrade(
                Tier::Bronze,
                CardTarget(1, TargetCondition::NameIncludes("piggle".to_string())),
            );
        }
        if tooltip == "deal damage equal to double this item's value." {
            return Effect::DealDamage(
                PlayerTarget::Opponent,
                DerivedValue::FromCard(
                    CardTarget(1, TargetCondition::IsSelf),
                    CardDerivedProperty::Value,
                    2.0,
                ),
            );
        }
        if tooltip == "deal damage equal to this item's value." {
            return Effect::DealDamage(
                PlayerTarget::Opponent,
                DerivedValue::FromCard(
                    CardTarget(1, TargetCondition::IsSelf),
                    CardDerivedProperty::Value,
                    1.0,
                ),
            );
        }
        if tooltip == "destroy an item for the fight." {
            return Effect::Destroy(CardTarget(
                1,
                TargetCondition::HasOwner(PlayerTarget::Opponent),
            ));
        }
        if tooltip == "gain 1 xp. if you had wanted poster in play, gain 1 additional xp." {
            return Effect::MultiEffect(vec![
                Effect::GainXp(PlayerTarget::Player, 1),
                Effect::ConditionalMatchItem(
                    CardTarget(
                        1,
                        TargetCondition::NameIncludes("wanted poster".to_string()),
                    ),
                    Effect::GainXp(PlayerTarget::Player, 1).into(),
                ),
            ]);
        }
        Effect::Raw(tooltip.to_string())
    }

    pub fn from_iter<T: Iterator>(tooltip: &mut T) -> Effect
    where
        T::Item: std::fmt::Display,
    {
        let mut tooltip_str = String::new();
        while let Some(s) = tooltip.next() {
            tooltip_str += &format!("{s} ");
        }
        Effect::from_tooltip_str(&tooltip_str)
    }
}
