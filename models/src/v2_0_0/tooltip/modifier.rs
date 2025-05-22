use super::EffectValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    DoubleCritDamage,
    WeaponDamage(EffectValue<u64>),
    LessDamageTaken(EffectValue<u64>),
    CritChance(u64),
    Cooldown(f64),
    Ammo(u64),
    Multicast(u64),
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
        }
    }
}
