use crate::v2_0_0::{EffectValue, Percentage, Tag};

use super::{
    CardDerivedProperty, CardTarget, Condition, DerivedValue, Effect, EffectEvent, GlobalEvent,
    Modifier, PlayerTarget, TargetCondition,
};

pub fn parse_numeric<T: std::str::FromStr>(cooldown_str: &str) -> anyhow::Result<T>
where
    T::Err: std::fmt::Display,
{
    let cooldown = crate::v2_0_0::re::NUMERIC_REGEX
        .find(cooldown_str)
        .ok_or(anyhow::anyhow!("no cooldown value in tooltip"))?
        .as_str()
        .parse::<T>()
        .map_err(|e| anyhow::anyhow!("could not parse cooldown {e}"))?;
    Ok(cooldown)
}

static TODO: bool = true; //TODO SELLSFORGOLD Not used since theres no extra value data
#[derive(Debug, Clone, PartialEq)]
pub enum Tooltip {
    Conditional(Condition, Box<Tooltip>),
    When(EffectEvent),
    StaticModifier(Modifier),
    ConditionalModifier(Condition, Modifier),
    SellsForGold,
    Raw(String),
}

impl std::fmt::Display for Tooltip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tooltip::Conditional(i, j) => write!(f, "Tooltip::Conditional({i}, {j})"),
            Tooltip::When(i) => write!(f, "Tooltip::When({i})"),
            Tooltip::SellsForGold => write!(f, "Tooltip::SellsForGold"),
            Tooltip::StaticModifier(i) => write!(f, "Tooltip::StaticModifier({i})"),
            Tooltip::ConditionalModifier(i, j) => {
                write!(f, "Tooltip::ConditionalModifier({i}, {j})")
            }
            Tooltip::Raw(i) => {
                write!(f, "Tooltip::Raw({i:?}.to_string())")
            }
        }
    }
}

impl Tooltip {
    fn from_at_the_start(tooltip: &str) -> anyhow::Result<Tooltip> {
        if let Some(rest) = tooltip.strip_prefix("at the start of each day,") {
            Ok(Tooltip::When(EffectEvent::OnDayStart(
                Effect::from_tooltip_str(rest.trim()),
            )))
        } else if let Some(rest) = tooltip.strip_prefix("at the start of each fight,") {
            Ok(Tooltip::When(EffectEvent::OnFightStart(
                Effect::from_tooltip_str(rest.trim()),
            )))
        } else {
            anyhow::bail!("invalid 'at the start' variant: {tooltip}")
        }
    }

    fn from_first_time(tooltip: &str) -> anyhow::Result<Tooltip> {
        let effect_event = if let Some(rest) =
            tooltip.strip_prefix("the first time you fall below half health each fight, ")
        {
            EffectEvent::OnFirstTime(
                GlobalEvent::PlayerFallsBelowHpPercentage(50.0),
                Effect::from_tooltip_str(rest),
            )
        } else {
            anyhow::bail!("invalid first time event: '{tooltip}'");
        };
        Ok(Tooltip::When(effect_event))
    }

    fn from_when(tooltip: &str) -> anyhow::Result<Tooltip> {
        let effect_event = if let Some(rest) =
            tooltip.strip_prefix("when you use an adjacent item,")
        {
            EffectEvent::OnCardUsed(TargetCondition::Adjacent, Effect::from_tooltip_str(rest))
        } else if let Some(rest) = tooltip.strip_prefix("when you use an item,") {
            EffectEvent::OnCardUsed(
                TargetCondition::HasOwner(PlayerTarget::Player),
                Effect::from_tooltip_str(rest),
            )
        } else if let Some(rest) = tooltip.strip_prefix("when you sell this") {
            EffectEvent::OnCardSold(Effect::from_tooltip_str(rest))
        } else if let Some(rest) = tooltip.strip_prefix("when this is transformed,") {
            EffectEvent::OnCardTransformed(Effect::from_tooltip_str(rest))
        } else if let Some(rest) = tooltip.strip_prefix("when you use shield or heal,") {
            EffectEvent::OnCardUsed(
                TargetCondition::HasOwner(PlayerTarget::Player)
                    & (TargetCondition::HasTag(Tag::Heal) | TargetCondition::HasTag(Tag::Shield)),
                Effect::from_tooltip_str(rest),
            )
        } else if let Some(rest) = tooltip.strip_prefix("when you crit,") {
            EffectEvent::OnCrit(
                TargetCondition::HasOwner(PlayerTarget::Player),
                Effect::from_tooltip_str(rest),
            )
        } else if let Some(rest) = tooltip.strip_prefix("when your enemy uses an item,") {
            EffectEvent::OnCardUsed(
                TargetCondition::HasOwner(PlayerTarget::Opponent),
                Effect::from_tooltip_str(rest),
            )
        } else if let Some(rest) = tooltip.strip_prefix("when you win a fight against a hero,") {
            EffectEvent::OnWinVersusHero(Effect::from_tooltip_str(rest))
        } else if let Some(rest) = tooltip.strip_prefix("when you use a weapon,") {
            EffectEvent::OnCardUsed(
                TargetCondition::HasOwner(PlayerTarget::Player),
                Effect::from_tooltip_str(rest),
            )
        } else {
            anyhow::bail!("invalid conditional effect: '{tooltip}'")
        };
        Ok(Tooltip::When(effect_event))
    }

    pub fn from_or_raw<T: TryInto<Tooltip> + ToString>(value: T) -> Self {
        let copy = value.to_string();
        let tooltip = value.try_into().unwrap_or(Tooltip::Raw(copy));
        tooltip
    }

    pub fn from_or_raw_enchantment<T: ToString>(value: T) -> Self {
        let tooltip = Tooltip::from_enchantment(value.to_string());
        tooltip
    }

    fn from_enchantment(value: String) -> Tooltip {
        Tooltip::Raw(value)
    }

    fn from_str(value: &str) -> Tooltip {
        let value = value.to_lowercase();
        let value = value.as_str();
        if value.starts_with("cooldown") {
            return parse_numeric(value)
                .map(Modifier::Cooldown)
                .map(Tooltip::StaticModifier)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value.starts_with("ammo") {
            return parse_numeric(value)
                .map(Modifier::Ammo)
                .map(Tooltip::StaticModifier)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value.starts_with("multicast") {
            return parse_numeric(value)
                .map(Modifier::Multicast)
                .map(Tooltip::StaticModifier)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if let Some(rest) = value.strip_prefix("use") {
            TargetCondition::from_str(rest);
            return parse_numeric(value)
                .map(Modifier::Multicast)
                .map(Tooltip::StaticModifier)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value.starts_with("crit chance") {
            return parse_numeric(value)
                .map(Percentage::from_percentage_value)
                .map(Modifier::CritChance)
                .map(Tooltip::StaticModifier)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value.starts_with("slow all enemy items for") {
            return parse_numeric(value)
                .map(|v| {
                    Effect::Slow(
                        CardTarget(
                            usize::MAX,
                            TargetCondition::HasOwner(PlayerTarget::Opponent),
                        ),
                        v,
                    )
                })
                .map(EffectEvent::OnCooldown)
                .map(Tooltip::When)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value.starts_with("you take") && value.ends_with("% less damage.") {
            return parse_numeric(value)
                .map(EffectValue::Percentage)
                .map(Modifier::LessDamageTaken)
                .map(Tooltip::StaticModifier)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value.starts_with("you take no damage for") {
            return parse_numeric(value)
                .map(Effect::DamageImmunity)
                .map(EffectEvent::OnCooldown)
                .map(Tooltip::When)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value.starts_with("at the start of") {
            return Tooltip::from_at_the_start(value).unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value == "sells for gold." {
            return Tooltip::SellsForGold;
        }
        if value == "this deals double crit damage." {
            return Tooltip::StaticModifier(Modifier::DoubleCritDamage);
        }
        if value.starts_with("your other weapons gain") && value.ends_with("damage for the fight.")
        {
            return parse_numeric(value)
                .map(|v: u32| {
                    Effect::IncreaseDamage(
                        CardTarget(
                            usize::MAX,
                            TargetCondition::HasOwner(PlayerTarget::Player)
                                & TargetCondition::HasTag(Tag::Weapon),
                        ),
                        EffectValue::Flat(v),
                    )
                })
                .map(EffectEvent::OnCooldown)
                .map(Tooltip::When)
                .unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if let Some(rest) = value.strip_prefix("use ") {
            let effect = Effect::UseCard(CardTarget(
                1,
                TargetCondition::from_str(&rest) & TargetCondition::HasOwner(PlayerTarget::Player),
            ));
            return Tooltip::When(EffectEvent::OnCooldown(effect));
        }
        if value.starts_with("when") {
            return Tooltip::from_when(value).unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value.starts_with("the first time") {
            return Tooltip::from_first_time(value).unwrap_or(Tooltip::Raw(value.to_string()));
        }
        if value == "this has double damage." {
            return Tooltip::StaticModifier(Modifier::WeaponDamage(EffectValue::Percentage(100)));
        }
        if value == "this cannot be frozen, slowed or destroyed." {
            return Tooltip::StaticModifier(Modifier::Radiant);
        }
        if value == "+50% crit chance" {
            return Tooltip::StaticModifier(Modifier::CritChance(
                Percentage::from_percentage_value(50.0),
            ));
        }
        if value == "this has +1 multicast." {
            return Tooltip::StaticModifier(Modifier::Multicast(1));
        }
        if value == "shield equal to the value of the adjacent items." {
            let todo = true; //TODO: change to percentage
            return Tooltip::When(EffectEvent::OnCooldown(Effect::Shield(
                PlayerTarget::Player,
                DerivedValue::FromCard(
                    CardTarget(2, TargetCondition::Adjacent),
                    CardDerivedProperty::Value,
                    1.0,
                ),
            )));
        }
        if let Some(capture) = crate::v2_0_0::re::HASTE_N_FOR_M.captures(value) {
            if let (Some(n_str), Some(m_str)) = (capture.get(1), capture.get(2)) {
                match (
                    n_str.as_str().parse::<usize>(),
                    m_str.as_str().parse::<f64>(),
                ) {
                    (Ok(n), Ok(m)) => {
                        return Tooltip::When(EffectEvent::OnCooldown(Effect::Haste(
                            CardTarget(n, TargetCondition::HasOwner(PlayerTarget::Player)),
                            m,
                        )));
                    }
                    _ => {}
                }
            }
        }
        if let Some(capture) = crate::v2_0_0::re::SLOW_N_FOR_M.captures(value) {
            if let (Some(n_str), Some(m_str)) = (capture.get(1), capture.get(2)) {
                match (
                    n_str.as_str().parse::<usize>(),
                    m_str.as_str().parse::<f64>(),
                ) {
                    (Ok(n), Ok(m)) => {
                        return Tooltip::When(EffectEvent::OnCooldown(Effect::Slow(
                            CardTarget(n, TargetCondition::HasOwner(PlayerTarget::Opponent)),
                            m,
                        )));
                    }
                    _ => {}
                }
            }
        }
        if let Some(capture) = crate::v2_0_0::re::FREEZE_N_FOR_M.captures(value) {
            if let (Some(n_str), Some(m_str)) = (capture.get(1), capture.get(2)) {
                match (
                    n_str.as_str().parse::<usize>(),
                    m_str.as_str().parse::<f64>(),
                ) {
                    (Ok(n), Ok(m)) => {
                        return Tooltip::When(EffectEvent::OnCooldown(Effect::Freeze(
                            CardTarget(n, TargetCondition::HasOwner(PlayerTarget::Opponent)),
                            m,
                        )));
                    }
                    _ => {}
                }
            }
        }
        if let Some(capture) = crate::v2_0_0::re::FREEZE_N_FOR_M_OF_SIZE.captures(value) {
            if let (Some(n_str), Some(size_str), Some(m_str)) =
                (capture.get(1), capture.get(2), capture.get(3))
            {
                match (
                    n_str.as_str().parse::<usize>(),
                    m_str.as_str().parse::<f64>(),
                    size_str.as_str().try_into(),
                ) {
                    (Ok(n), Ok(m), Ok(size)) => {
                        return Tooltip::When(EffectEvent::OnCooldown(Effect::Freeze(
                            CardTarget(
                                n,
                                TargetCondition::HasOwner(PlayerTarget::Opponent)
                                    & TargetCondition::HasSize(size)
                                    & TargetCondition::HasCooldown,
                            ),
                            m,
                        )));
                    }
                    _ => {}
                }
            }
        }
        if let Some(capture) = crate::v2_0_0::re::STATIC_WEAPON_DAMAGE.captures(value) {
            if let Some(damage_str) = capture.get(1) {
                if let Ok(damage) = damage_str.as_str().parse::<u32>() {
                    return Tooltip::StaticModifier(Modifier::WeaponDamage(EffectValue::Flat(
                        damage,
                    )));
                }
            }
        }
        match EffectEvent::from_tooltip_str(value) {
            EffectEvent::Raw(..) => match Effect::from_tooltip_str(value) {
                Effect::Raw(..) => Tooltip::Raw(value.to_string()),
                e => Tooltip::When(EffectEvent::OnCooldown(e)),
            },
            e => Tooltip::When(e),
        }
    }
}

impl TryFrom<&str> for Tooltip {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let t = Tooltip::from_str(value);
        Ok(t)
    }
}
