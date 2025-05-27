use std::str::FromStr;

use heck::ToTitleCase;
use regex::Regex;

use crate::v2_0_0::{Tag, Tier};

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum CardDerivedProperty {
    Value,
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
    Burn(PlayerTarget, u32),
    Heal(PlayerTarget, u32),
    Shield(PlayerTarget, DerivedValue<u32>),
    Poison(PlayerTarget, u32),
    GainGold(PlayerTarget, u32),
    DealDamage(PlayerTarget, u32),
    UseCard(CardTarget),
    Upgrade(CardTarget, Tier),
    PermanentMaxHealthIncrease(EffectValue<u32>),
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
            Effect::GainShield(i, j) => write!(f, "Effect::GainShield({i}, {j})"),
            Effect::CooldownReduction(i, j) => write!(f, "Effect::CooldownReduction({i}, {j})"),
            Effect::Freeze(i, j) => write!(f, "Effect::Freeze({i}, {j:.2})"),
            Effect::Slow(i, j) => write!(f, "Effect::Slow({i}, {j:.2})"),
            Effect::DamageImmunity(i) => write!(f, "Effect::DamageImmunity({i:.2})"),
            Effect::Destroy(i) => write!(f, "Effect::Destroy({i})"),
            Effect::Burn(i, j) => write!(f, "Effect::Burn({i}, {j})"),
            Effect::Shield(i, j) => write!(f, "Effect::Shield({i}, {j})"),
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
            Effect::Upgrade(i, j) => write!(f, "Effect::Upgrade({i}, Tier::{j:?})"),
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
        if let Some(captures) = EFFECT_GET_ITEMS_REGEX.captures(tooltip) {
            if let (Some(count), Some(name)) = (captures.get(1), captures.get(2)) {
                let count = match [count.as_str()] {
                    ["a"] => 1,
                    [count_str] if let Ok(count) = count_str.parse::<u32>() => count,
                    [..] => return Effect::Raw(tooltip.to_string()),
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
                if let Ok(poison) = poison_str.as_str().parse::<u32>() {
                    return Effect::Poison(PlayerTarget::Player, poison);
                }
            }
        }
        if let Some(captures) = EFFECT_GAIN_PERMANENT_MAX_HP.captures(tooltip) {
            if let Some(amount_str) = captures.get(1) {
                if let Ok(amount) = amount_str.as_str().parse::<u32>() {
                    return Effect::PermanentMaxHealthIncrease(EffectValue::Flat(amount));
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

        if let Some(captures) = EFFECT_GAIN_GOLD.captures(tooltip) {
            if let Some(gold_str) = captures.get(1) {
                if let Ok(gold) = gold_str.as_str().parse::<u32>() {
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
                CardTarget(1, TargetCondition::NameIncludes(" piggle".to_string())),
                Tier::Bronze,
            );
        }
        // if let Some(captures) = EFFECT_UPGRADE_LOWER_TIER_TAGGED.captures(tooltip) {
        //     if let Some(tag_str) = captures.get(1) {
        //         if let Ok(tag) = Tag::from_str(tag_str.as_str()) {
        //             let condition = TargetCondition::HasOwner(PlayerTarget::Player)
        //                 & C
        //             let condition = TargetConditionCondition::And(
        //                 TargetConditionCondition::IsOwn.into(),
        //                 TargetConditionCondition::And(
        //                     TargetConditionCondition::HasTag(tag).into(),
        //                     TargetConditionCondition::IsLowerTierThanSelf.into(),
        //                 )
        //                 .into(),
        //             );
        //             return Effect::Upgrade(Tier::Bronze, TargetCondition::Conditional(condition));
        //         }
        //     }
        // }
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
