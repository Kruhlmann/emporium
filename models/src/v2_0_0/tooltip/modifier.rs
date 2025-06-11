use crate::v2_0_0::Percentage;

use super::EffectValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    DoubleCritDamage,
    WeaponDamage(EffectValue<u32>),
    LessDamageTaken(EffectValue<u32>),
    CritChance(Percentage),
    Cooldown(f64),
    Ammo(u32),
    Multicast(u32),
    IncreasedValue(u32),
    Radiant,
}

impl std::fmt::Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modifier::DoubleCritDamage => write!(f, "Modifier::DoubleCritDamage"),
            Modifier::Radiant => write!(f, "Modifier::Radiant"),
            Modifier::Cooldown(i) => write!(f, "Modifier::Cooldown({i:.2})"),
            Modifier::Ammo(i) => write!(f, "Modifier::Ammo({i})"),
            Modifier::Multicast(i) => write!(f, "Modifier::Multicast({i})"),
            Modifier::CritChance(i) => write!(f, "Modifier::CritChance({i:.2})"),
            Modifier::LessDamageTaken(i) => write!(f, "Modifier::LessDamageTaken({i})"),
            Modifier::WeaponDamage(i) => write!(f, "Modifier::WeaponDamage({i})"),
            Modifier::IncreasedValue(i) => write!(f, "Modifier::IncreasedValue({i})"),
        }
    }
}
