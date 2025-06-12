use super::{CardTarget, DerivedValue, Effect, GlobalEvent, PlayerTarget, TargetCondition};

#[derive(Debug, Clone, PartialEq)]
pub enum EffectEvent {
    OnCooldown(Effect),
    OnCardTransformed(Effect),
    OnDayStart(Effect),
    OnWinVersusHero(Effect),
    OnFightStart(Effect),
    OnCardSold(Effect),
    OnCardUsed(TargetCondition, Effect),
    OnCrit(TargetCondition, Effect),
    OnFirstTime(GlobalEvent, Effect),
    Raw(String),
}

impl std::fmt::Display for EffectEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectEvent::OnCooldown(i) => write!(f, "EffectEvent::OnCooldown({i})"),
            EffectEvent::OnCardSold(i) => write!(f, "EffectEvent::OnCardSold({i})"),
            EffectEvent::OnCardTransformed(i) => write!(f, "EffectEvent::OnCardTransformed({i})"),
            EffectEvent::OnDayStart(i) => write!(f, "EffectEvent::OnDayStart({i})"),
            EffectEvent::OnWinVersusHero(i) => write!(f, "EffectEvent::OnWinVersusHero({i})"),
            EffectEvent::OnFightStart(i) => write!(f, "EffectEvent::OnFightStart({i})"),
            EffectEvent::OnCardUsed(i, j) => write!(f, "EffectEvent::OnCardUsed({i}, {j})"),
            EffectEvent::OnCrit(i, j) => write!(f, "EffectEvent::OnCrit({i}, {j})"),
            EffectEvent::OnFirstTime(i, j) => write!(f, "EffectEvent::OnFirstTime({i}, {j})"),
            EffectEvent::Raw(i) => write!(f, "EffectEvent::Raw({i:?}.to_string())"),
        }
    }
}

impl EffectEvent {
    pub fn from_tooltip_str(tooltip: &str) -> EffectEvent {
        let tooltip = tooltip.trim();
        if let Some(captures) = crate::v2_0_0::re::EFFECT_BURN.captures(tooltip) {
            if let Some(burn_str) = captures.get(1) {
                if let Ok(burn) = burn_str.as_str().parse::<u32>().map(DerivedValue::Constant) {
                    return EffectEvent::OnCooldown(Effect::Burn(PlayerTarget::Opponent, burn));
                }
            }
        }
        if let Some(captures) = crate::v2_0_0::re::EFFECT_POISON.captures(tooltip) {
            if let Some(poison_str) = captures.get(1) {
                if let Ok(poison) = poison_str
                    .as_str()
                    .parse::<u32>()
                    .map(DerivedValue::Constant)
                {
                    return EffectEvent::OnCooldown(Effect::Poison(PlayerTarget::Opponent, poison));
                }
            }
        }
        if let Some(captures) = crate::v2_0_0::re::EFFECT_REGEN.captures(tooltip) {
            if let Some(regen_str) = captures.get(1) {
                if let Ok(regen) = regen_str.as_str().parse::<u32>() {
                    return EffectEvent::OnCooldown(Effect::Regen(
                        PlayerTarget::Player,
                        DerivedValue::Constant(regen),
                    ));
                }
            }
        }
        if let Some(captures) = crate::v2_0_0::re::EFFECT_SHIELD.captures(tooltip) {
            if let Some(shield_str) = captures.get(1) {
                if let Ok(shield) = shield_str.as_str().parse::<u32>() {
                    return EffectEvent::OnCooldown(Effect::Shield(
                        PlayerTarget::Player,
                        DerivedValue::Constant(shield),
                    ));
                }
            }
        }
        if let Some(captures) = crate::v2_0_0::re::EFFECT_HEAL.captures(tooltip) {
            if let Some(heal_str) = captures.get(1) {
                if let Ok(heal) = heal_str.as_str().parse::<u32>().map(DerivedValue::Constant) {
                    return EffectEvent::OnCooldown(Effect::Heal(PlayerTarget::Player, heal));
                }
            }
        }

        if let Some(captures) = crate::v2_0_0::re::EFFECT_DEAL_DAMAGE.captures(tooltip) {
            if let Some(damage_str) = captures.get(1) {
                if let Ok(damage) = damage_str
                    .as_str()
                    .parse::<u32>()
                    .map(DerivedValue::Constant)
                {
                    return EffectEvent::OnCooldown(Effect::DealDamage(
                        PlayerTarget::Opponent,
                        damage,
                    ));
                }
            }
        }

        if tooltip == "use all your other items." {
            return EffectEvent::OnCooldown(Effect::Use(CardTarget(
                usize::MAX,
                !TargetCondition::IsSelf & TargetCondition::HasOwner(PlayerTarget::Player),
            )));
        }

        EffectEvent::Raw(tooltip.to_string())
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
